//! Features for the bot

// Discontinued
// pub mod mail;
// pub mod monitor;
// pub mod slackimport;
// pub mod gitcommands;
// pub mod docker;
// pub mod calendar;

pub mod airtable;
// pub mod event;
pub mod archivage;
pub mod frontline;
pub mod funny;
pub mod invite_action;
pub mod mecleanup;
pub mod ordering;
pub mod project_manager;
pub mod renaming;
pub mod threadcontrol;

use serenity::{http, prelude::TypeMapKey};
use std::sync::Arc;
use std::thread;
use threadcontrol::ThreadControl;

pub struct Features {
  pub thread_control: ThreadControl,
  pub running: bool,
}

impl TypeMapKey for Features {
  type Value = Features;
}

impl Features {
  pub fn new() -> Self {
    Features {
      running: false,
      thread_control: ThreadControl::new(),
    }
  }

  /// Spawn a Thread per feature to run in background
  pub fn run(&mut self, http: &Arc<http::Http>) {
    println!("Running featrues");
    // let http_clone = http.clone();
    // let tc_clone = self.thread_control.clone();
    // thread::spawn(move || event::check_events(http_clone, || ThreadControl::check(&tc_clone)));
    let http_clone = http.clone();
    let tc_clone = self.thread_control.clone();
    thread::spawn(|| airtable::check(http_clone, move || ThreadControl::check(&tc_clone)));
    let http_clone = http.clone();
    let tc_clone = self.thread_control.clone();
    thread::spawn(|| frontline::check(http_clone, move || ThreadControl::check(&tc_clone)));
  }
}
