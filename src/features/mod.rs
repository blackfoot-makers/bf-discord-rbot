//! Features for the bot

pub mod mail;
pub mod monitor;
pub mod notify;
pub mod slackimport;

use std::thread;

/// Spawn a Thread for [`mail`] and [`monitor`] to run in background
///
/// [`mail`]: mail/index.html
/// [`monitor`]: monitor/index.html
pub fn run() {
    println!("Running featrues");
    thread::spawn(move || mail::check_mail());
    // thread::spawn(|| monitor::error_code_check());
    thread::spawn(|| notify::check_events());
}
