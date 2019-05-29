//! Small example program that runs a login using a username and password and prints out the
//! resulting response from shotgun.
//!
//! Set the `SG_SERVER` environment variable to the url for your shotgun server, eg:
//!
//! ```text
//! export SG_SERVER=https://shotgun.example.com
//! ```
//!
//! `shotgun_rs` also looks at the `CA_BUNDLE` environment variable for when you need a custom CA
//! loaded to access your shotgun server, for example:
//!
//! ```text
//! export CA_BUNDLE=/etc/ssl/my-ca-certs.crt
//! ```
//!
//! Usage:
//!
//! ```text
//! $ cargo run --example login -- <username>
//! ```

extern crate shotgun_rs;
use shotgun_rs::TokenResponse;
use std::env;
use tokio::prelude::*;

fn main() {
    // Get a username from argv.
    let username = env::args()
        .skip(1)
        .take(1)
        .next()
        .expect("Please specify a user to login as");

    // Prompt the user for a password.
    let password = rpassword::read_password_from_tty(Some("Password: ")).unwrap();

    // Prepare a future pipeline. This api is async (read: lazy) so everything chained to the
    // initial call doesn't
    // happen until we "give" the future to the runtime afterwards.
    let fut = {
        // Important to create the shotgun client and have it drop before the work is passed off
        // to the runtime (and so we build the client and futures inside a block).
        let sg = shotgun_rs::Shotgun::new(
            env::var("SG_SERVER").expect("SG_SERVER is required var."),
            None,
            None,
        )
        .expect("SG Client");

        sg.authenticate_user(&username, &password)
            // The receiver can use a type hint here to tell the client how to deserialize the
            // response from the server.
            //
            // The library provides a struct, `AccessTokenResponse`, which fits the shape of a
            // successful auth attempt, but you can of course provide your own deserialization
            // target if you only need a subset of the fields.
            // For this simple case, we could even use a plain `serde_json::Value`.
            .and_then(|resp: TokenResponse| {
                // So long as the body parses as an object and does not contain "errors" the auth
                // check probably succeeded.
                println!("\nLogin Succeeded:\n{:?}", resp);
                Ok(())
            })
            .map_err(|e| {
                // If the future ended in Err, then something else went bad.
                // Could be an I/O problem, or the server couldn't respond, time out, malformed or
                // otherwise non-json response...
                // ... or the creds could have been bad.
                eprintln!("\nSomething bad happend:\n{}", e);
            })
    };

    // Execute the future pipeline, blocking until it completes.
    tokio::run(fut);
}
