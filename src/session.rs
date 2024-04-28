use greetd_ipc::{AuthMessageType, ErrorType, Response};

#[derive(Default)]
pub struct SessionOptions<'a> {
  pub username: &'a str,
  pub password: &'a str,
  pub mfa: bool,
  #[cfg(feature = "fingerprint")]
  pub fingerprint: bool,
}

#[derive(Debug, PartialEq)]
pub enum State {
  Username,
  Password,
  Mfa,
  #[cfg(feature = "fingerprint")]
  Fingerprint,
  Done,
}

#[derive(Debug)]
pub struct Context {
  pub state: State,
  pub username: bool,
  pub password: bool,
  pub mfa: bool,
}

impl Default for Context {
  fn default() -> Self {
    Context {
      state: State::Username,
      username: false,
      password: false,
      mfa: false,
    }
  }
}

impl Context {
  pub fn next(&mut self, opts: &SessionOptions<'_>) -> Response {
    if let Some(error) = self.check() {
      return error;
    }

    match self.state {
      State::Username => self.password_prompt(),

      #[cfg(not(feature = "fingerprint"))]
      State::Password => match opts.mfa {
        true => self.mfa_prompt(),
        _ => Response::Success,
      },

      #[cfg(feature = "fingerprint")]
      State::Password => match (opts.mfa, opts.fingerprint) {
        (true, _) => self.mfa_prompt(),
        #[cfg(feature = "fingerprint")]
        (false, true) => self.fingerprint_prompt(),
        _ => Response::Success,
      },

      #[cfg(not(feature = "fingerprint"))]
      State::Mfa => self.success(),

      #[cfg(feature = "fingerprint")]
      State::Mfa => match opts.fingerprint {
        true => self.fingerprint_prompt(),
        false => self.success(),
      },

      #[cfg(feature = "fingerprint")]
      State::Fingerprint => self.success(),

      State::Done => Response::Success,
    }
  }

  fn check(&self) -> Option<Response> {
    let ok = match self.state {
      State::Username => true,
      State::Password => self.username && self.password,
      State::Mfa => self.mfa,
      _ => true,
    };

    if ok {
      return None;
    }

    Some(Response::Error {
      error_type: ErrorType::AuthError,
      description: "Invalid credentials".to_string(),
    })
  }

  fn password_prompt(&mut self) -> Response {
    self.state = State::Password;

    Response::AuthMessage {
      auth_message_type: AuthMessageType::Secret,
      auth_message: "Password:".to_string(),
    }
  }

  fn mfa_prompt(&mut self) -> Response {
    self.state = State::Mfa;

    Response::AuthMessage {
      auth_message_type: AuthMessageType::Visible,
      auth_message: "7 + 2 =".to_string(),
    }
  }

  #[cfg(feature = "fingerprint")]
  fn fingerprint_prompt(&mut self) -> Response {
    self.state = State::Fingerprint;

    Response::AuthMessage {
      auth_message_type: AuthMessageType::Info,
      auth_message: "Scan your fingerprint...".to_string(),
    }
  }

  fn success(&mut self) -> Response {
    self.state = State::Done;

    Response::Success
  }
}
