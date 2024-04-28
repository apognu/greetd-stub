# Stub server for greetd

This software can be used, either as a binary or a library, to spin up a more-or-less greetd-compatible server, to be used to develop against, or for automated testing.

It will ask for a user, password, and optionally a MFA-kind question and a fingerprint scan.

## As a binary

```
$ greetd-stub -s /tmp/greetd-stub.sock --user apognu:mypassword --mfa --fingerprint
```

You can then direct you greeter to the provided socket (which defaults to `/tmp/greetd-stub.sock`) to make it work.

## As a library

This software can also be used in-process in order to spin us a greetd server from within your test environment:

```rust
use libgreetd_stub::SessionOptions;

#[tokio::main]
async fn mytest() {
  let opts = SessionOptions {
    username: "apognu",
    password: "mypassword",
    mfa: false,
    fingerprint: false,
  };

  let server = tokio::task::spawn(async move {
    libgreetd_stub::start("/tmp/greetd-stub.sock", opts).await;
  });

  // Awaiting `server` will spin up the stub, you can now run your integration tests.
}
```
