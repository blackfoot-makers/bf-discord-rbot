//! The base of the program containing the abstractions for files and connection to discord.

pub mod commands;
pub mod eventhandler;
pub mod files;
pub mod parse;
pub mod permissions;
pub mod process;
pub mod validation;

use std::thread;

/// Spawn thread to run core functions.
pub fn run() -> thread::JoinHandle<()> {
  thread::spawn(eventhandler::bot_connect)
}
