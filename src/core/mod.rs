//! The base of the program containing the abstractions for files and connection to discord.

pub mod commands;
pub mod eventhandler;
// pub mod files;
// pub mod api;
pub mod parse;
pub mod permissions;
pub mod process;
pub mod slash_command;
pub mod validation;

/// Spawn thread to run core functions.
pub fn run() {
  eventhandler::bot_connect();
}
