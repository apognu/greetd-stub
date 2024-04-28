mod data;
mod server;
mod session;

use std::{fs, path::Path};

use tokio::net::UnixListener;

pub use crate::session::SessionOptions;

pub async fn start<P>(socket: P, opts: &SessionOptions<'static>)
where
  P: AsRef<Path>,
{
  let _ = fs::remove_file(&socket);
  let listener = UnixListener::bind(socket).unwrap();

  loop {
    if let Ok((stream, _)) = listener.accept().await {
      server::handle(stream, opts).await;
    }
  }
}
