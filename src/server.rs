#![allow(dead_code)]

use greetd_ipc::{codec::TokioCodec, ErrorType, Request, Response};
use tokio::net::UnixStream;

#[cfg(feature = "fingerprint")]
use libfprint_rs::{FpContext, FpPrint};

use crate::session::{Context, SessionOptions, State};

pub async fn handle(mut stream: UnixStream, opts: &SessionOptions) {
  let mut context = Context::default();

  loop {
    let request = match Request::read_from(&mut stream).await {
      Ok(request) => request,
      Err(greetd_ipc::codec::Error::Eof) => return,
      Err(_) => return,
    };

    tracing::debug!("received request {request:?}");

    let response = match request {
      Request::CreateSession { username } => {
        context.username = username == opts.username;
        context.next(opts)
      }

      Request::PostAuthMessageResponse { response: Some(input) } => match context.state {
        State::Password => {
          context.password = input == opts.password;
          context.next(opts)
        }

        State::Mfa => {
          context.mfa = input == "9";
          context.next(opts)
        }

        _ => context.next(opts),
      },

      #[cfg(feature = "fingerprint")]
      Request::PostAuthMessageResponse { response: None } if context.state == State::Fingerprint => {
        let fp = FpContext::new();
        let devices = fp.devices();
        let device = devices.first().unwrap();
        device.open_sync(None).unwrap();

        let print = FpPrint::deserialize(crate::data::PRINT).unwrap();
        let _ = device.verify_sync(&print, None, None, None::<()>, None);

        device.close_sync(None).unwrap();

        context.next(opts)
      }

      Request::StartSession { cmd, env } => {
        tracing::info!("session successfully started: {cmd:?}");
        tracing::info!("session environment: {:?}", env);

        Response::Success
      }

      Request::CancelSession => {
        return;
      }

      _ => Response::Error {
        error_type: ErrorType::AuthError,
        description: "Communication error".to_string(),
      },
    };

    tracing::debug!("sending response {response:?}");

    let _ = response.write_to(&mut stream).await;
  }
}
