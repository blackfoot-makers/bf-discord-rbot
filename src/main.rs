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
#[macro_use]
pub mod macros;

mod core;
mod database;
mod features;

use dotenv::dotenv;
use std::io::{self, Write};
use std::str::FromStr;

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
                } else if input == "users" {
                    let db_instance = database::INSTANCE.write().unwrap();
                    println!("Users: {:?}", db_instance.users);
                } else if input == "messages" {
                    let db_instance = database::INSTANCE.write().unwrap();
                    println!("messages: {:?}", db_instance.messages);
                // } else if input == "deploy" {
                //     let http = core::process::HTTP_STATIC.read().clone().unwrap();
                //     features::docker::deploy_test(
                //         String::from("GreeFine"),
                //         String::from("CI-Preview-Exemple"),
                //         String::from("master"),
                //         http,
                //     );
                } else if input == "channels" {
                    features::ordering::guild_chanels(serenity::model::id::GuildId(
                        339372728366923776,
                    ));
                } else if input == "chan" {
                    let mut channel = String::new();
                    let mut position = String::new();
                    print!("channel?(id) >");
                    io::stdout().flush().unwrap();
                    io::stdin().read_line(&mut channel).unwrap();
                    channel.pop();
                    let chanid = channel.parse::<u64>().unwrap();

                    print!("position?(number) >");
                    io::stdout().flush().unwrap();
                    io::stdin().read_line(&mut position).unwrap();
                    position.pop();
                    let positionnum = position.parse::<u64>().unwrap();

                    features::ordering::move_channels(chanid, positionnum);
                } else if input == "promote" {
                    let mut who = String::new();
                    let mut rolestring = String::new();
                    print!("Who?(id) >");
                    io::stdout().flush().unwrap();
                    io::stdin().read_line(&mut who).unwrap();
                    who.pop();
                    let userid = who.parse::<u64>().unwrap();

                    print!("role?(string) >");
                    io::stdout().flush().unwrap();
                    io::stdin().read_line(&mut rolestring).unwrap();
                    rolestring.pop();
                    let role = match database::Role::from_str(&*rolestring) {
                        Err(_) => return println!("Role not found"),
                        Ok(role) => role,
                    };

                    let mut db_instance = database::INSTANCE.write().unwrap();
                    println!("Promoting =>{}", db_instance.user_role_update(userid, role));
                } else {
                    println!("Invalid input [{}]", input);
                }
            }
            Err(error) => println!("error: {}", error),
        }
    }
}
