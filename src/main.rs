mod data;
mod server;
mod session;

use std::{env, error::Error, fs, process};

use session::SessionOptions;
use tokio::net::UnixListener;

const DEFAULT_SOCKET: &str = "/tmp/greetd.sock";
const DEFAULT_USERNAME: &str = "user";
const DEFAULT_PASSWORD: &str = "password";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
  let mut args = getopts::Options::new();
  args.optflag("h", "help", "show this usage information");
  args.optopt("s", "socket", "path to the UNIX socket to create", "PATH");
  args.optopt("u", "user", "username and password to accept", "USERNAME:PASSWORD");
  args.optflag("m", "mfa", "enable second-factor authentication");

  #[cfg(feature = "fingerprint")]
  args.optflag("f", "fingerprint", "enable fingerprint scan");

  let opts = match args.parse(env::args().collect::<Vec<String>>()) {
    Ok(matches) => matches,
    Err(err) => {
      eprintln!("{err}");
      usage(args);
      process::exit(1)
    }
  };

  if opts.opt_present("help") {
    usage(args);
    process::exit(0);
  }

  let socket = match opts.opt_str("socket") {
    Some(socket) => socket,
    None => DEFAULT_SOCKET.to_string(),
  };

  let (username, password) = match opts.opt_str("user") {
    Some(spec) => match spec.split_once(':') {
      Some((username, password)) => (username.to_string(), password.to_string()),
      None => {
        eprintln!("invalid format for user option, should be USERNAME:PASSWORD");
        usage(args);
        process::exit(1);
      }
    },

    None => (DEFAULT_USERNAME.to_string(), DEFAULT_PASSWORD.to_string()),
  };

  let session_opts = SessionOptions {
    username: &username,
    password: &password,
    mfa: opts.opt_present("mfa"),
    #[cfg(feature = "fingerprint")]
    fingerprint: opts.opt_present("fingerprint"),
  };

  let _ = fs::remove_file(&socket);
  let listener = UnixListener::bind(&socket).unwrap();

  loop {
    if let Ok((stream, _)) = listener.accept().await {
      server::handle(stream, &session_opts).await;
    }
  }
}

fn usage(opts: getopts::Options) {
  eprint!("{}", opts.usage("Usage: dummygreeter [OPTIONS]"));
}
