//! The base of the program containing the abstractions for files and connection to discord.

pub mod commands;
pub mod files;
pub mod process;

use std::thread;

/// Spawn thread to run core functions.
pub fn run() -> thread::JoinHandle<()> {
  thread::spawn(|| process::bot_connect())
}
