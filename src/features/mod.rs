//! Features for the bot

// Discontinued
// pub mod mail;
// pub mod monitor;
// pub mod slackimport;
// pub mod githooks;
// pub mod docker;

pub mod airtable;
pub mod calendar;
pub mod notify;

use serenity::http;
use std::sync::Arc;
use std::thread;

/// Spawn a Thread for [`notify`] and [`githooks`] to run in background
///
/// [`notify`]: notify/index.html
/// [`githooks`]: githooks/index.html
pub fn run(http: &Arc<http::Http>) {
  println!("Running featrues");

  let http_for_events = http.clone();
  let http_for_airtable = http.clone();

  // let http_for_githooks = http.clone();
  thread::spawn(move || notify::check_events(http_for_events));
  // thread::spawn(move || githooks::init(http_for_githooks));
  thread::spawn(move || calendar::unfeed_calendar());
  thread::spawn(move || airtable::check_airtable(http_for_airtable));
}
