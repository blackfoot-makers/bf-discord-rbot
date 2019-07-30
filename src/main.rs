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

extern crate chrono;
// extern crate imap;
// extern crate native_tls;
// extern crate reqwest;
extern crate hyper;
extern crate log;
extern crate rand;
extern crate rifling;
extern crate serde;
extern crate serde_json;
extern crate serenity;
extern crate time;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

mod core;
mod features;

use core::files;
use features::notify;
use hyper::rt::{run, Future};
use hyper::{Error, Server};
use rifling::{Constructor, Delivery, Hook};
use std::io;
use std::sync::{Arc, RwLock};
/// Store the credential for the mail account, and discord token.
#[derive(Serialize, Deserialize)]
struct Credentials {
    pub email: String,
    pub password: String,
    pub domain: String,
    pub token: String,
}

impl Credentials {
    pub fn new() -> Credentials {
        Credentials {
            email: String::new(),
            password: String::new(),
            domain: String::new(),
            token: String::new(),
        }
    }
}

// Represent the file for mails config
// static ref MAIL_INFO_FILE: Arc<RwLock<files::FileReader<Info>>> = Arc::new(RwLock::new(
//     files::build(String::from("mail_info.json"), Info::new())
// ));
lazy_static! {
    /// Represent the file that store the credential
    static ref CREDENTIALS_FILE: Arc<files::FileReader<Credentials>> = Arc::new(files::build(
        String::from("credentials.json"),
        Credentials::new()
    ));
    /// Is the last mail text, stored to be use on call back
    static ref MAIL_LOCK: Arc<RwLock<String>> =
        Arc::new(RwLock::new(String::from("No email stored")));

    static ref NOTIFY_EVENT_FILE: Arc<RwLock<files::FileReader<Vec<notify::Event>>>> = Arc::new(RwLock::new(
        files::build(String::from("events.json"), Vec::new())
    ));
}

/// We run the core and we loop on a basic cmd.
fn main() {
    let mut cons = Constructor::new();
    let hook = Hook::new("*", Some(String::from("secret")), |delivery: &Delivery| {
        println!("Received delivery: {:?}", delivery)
    });
    cons.register(hook);
    let addr = "0.0.0.0:4567".parse().unwrap();
    let server = Server::bind(&addr)
        .serve(cons)
        .map_err(|e: Error| println!("Error: {:?}", e));
    run(server);
    // let join_handle = core::run();
    // loop {
    //     let mut input = String::new();
    //     match io::stdin().read_line(&mut input) {
    //         Ok(n) => {
    //             if n == 0 {
    //                 join_handle.join().unwrap();
    //                 break;
    //             }
    //             input.pop();
    //             if input == "quit" {
    //                 break;
    //             } else if input.starts_with("msg") {
    //                 let split: Vec<&str> = input.split(' ').collect();
    //                 let _chan = split[1].parse::<u64>().unwrap();
    //             //FIXME let _ = ChannelId(chan).send_message(|m| m.content(split[2]));
    //             } else {
    //                 println!("Invalid input [{}]", input);
    //             }
    //         }
    //         Err(error) => println!("error: {}", error),
    //     }
    // }
}
