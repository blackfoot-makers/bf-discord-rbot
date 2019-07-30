//! Features for the bot

// Discontinued
// pub mod mail;
// pub mod monitor;
// pub mod slackimport;
pub mod notify;
pub mod githooks;

use serenity::http;
use std::sync::Arc;
use std::thread;


/// Spawn a Thread for [`notify`] and [`githooks`] to run in background
///
/// [`notify`]: notify/index.html
/// [`githooks`]: githooks/index.html
pub fn run(http: Arc<http::raw::Http>) {
  println!("Running featrues");
  // thread::spawn(move || mail::check_mail());
  // thread::spawn(|| monitor::error_code_check());
  thread::spawn(|| notify::check_events(http));
  // thread::spawn(|| githooks::check_hooks(http));
}
