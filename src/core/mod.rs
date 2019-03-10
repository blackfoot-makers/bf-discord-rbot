//! The base of the program containing the abstractions for files and connection to discord.

pub mod connection;
pub mod files;

use std::thread;

/// Spawn thread to run core functions.
pub fn run() -> thread::JoinHandle<()> {
    thread::spawn(|| connection::bot_connect())
}
