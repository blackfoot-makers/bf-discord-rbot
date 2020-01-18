//! discord-db is a Rust Discord BOT.
//!
//! To simply runs this bot fill the credentials.json file at the root of your directory with your informations
//!
//! # Credential.json
//! ```json
//! {
//!   "email": "your@email.io",
//!   "password": "password",
//!   "domain": "ssl0.ovh.net",
//!   "token": "YOURDISCORDTOKEN"
//! }
//! ```
//!
//! And run `cargo run`
//!
//! This bot is compose of 2 modules:
//!
//!  *  [Core][core docs] Wich is the active connection with discord and manage the events.
//!
//!  *  [Features][features docs] The features that the bot do.
//!
//!
//! [core docs]: core/index.html
//! [features docs]: features/index.html

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;

mod core;
mod database;
mod features;

use dotenv::dotenv;
use std::io;

/// We run the core and we loop on a basic cmd.
fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    let join_handle = core::run();

    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                // If the is no stdin just wait for core::run
                if n == 0 {
                    join_handle.join().unwrap();
                    break;
                }

                input.pop();
                if input == "quit" {
                    break;
                // } else if input == "deploy" {
                //     let http = core::process::HTTP_STATIC.read().clone().unwrap();
                //     features::docker::deploy_test(
                //         String::from("GreeFine"),
                //         String::from("CI-Preview-Exemple"),
                //         String::from("master"),
                //         http,
                //     );
                } else {
                    println!("Invalid input [{}]", input);
                }
            }
            Err(error) => println!("error: {}", error),
        }
    }
}
