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

extern crate bollard;
extern crate chrono;
extern crate failure;
extern crate futures;
extern crate hyper;
extern crate log;
extern crate pretty_env_logger;
extern crate rand;
extern crate reqwest;
extern crate rifling;
extern crate serde;
extern crate serde_json;
extern crate serenity;
extern crate time;
extern crate tokio;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

mod core;
mod features;

use std::io;

/// We run the core and we loop on a basic cmd.
fn main() {
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
                } else {
                    println!("Invalid input [{}]", input);
                }
            }
            Err(error) => println!("error: {}", error),
        }
    }
}
