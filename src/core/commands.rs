//! Handle the connection with discord and it's events.
use serenity::{model::id::ChannelId, prelude::*};
use std::collections::HashMap;
use std::process;

use features::notify::Event;

/// Struct that old Traits Implementations to Handle the different events send by discord.
pub struct Command {
  pub exec: fn(&Vec<&str>) -> String,
  pub argument_min: usize,
  pub argument_max: usize,
  pub channel: Option<ChannelId>,
  pub usage: String,
}

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

lazy_static! {
  pub static ref ATTACKED: RwLock<String> = RwLock::new(String::new());
  pub static ref TAG_MSG_LIST: HashMap<&'static str, &'static str> = hashmap![
    "ping" => "pong",
    "bad" => "😢",
    "Bonjour !" => "Bonsoir !",
    "Bonjour" => "Bonsoir !",
    "🖕" => "🖕"
  ];
  pub static ref CONTAIN_MSG_LIST: HashMap<&'static str, &'static str> = hashmap![
    "keke" => "https://media.giphy.com/media/26ufju9mygxXmfjos/giphy.gif",
    "kéké" => "https://media.giphy.com/media/26ufju9mygxXmfjos/giphy.gif",
    "bad bot" => "😎"
  ];
  pub static ref CONTAIN_REACTION_LIST: HashMap<&'static str, &'static str> = hashmap![
    "👊" => "👊",
    "licorne" => "🦄",
    "leslie" => "🦄",
    "max" => "🍌",
    "retard" => "⌚",
    "pm" => "🐱"
  ];
  pub static ref COMMANDS_LIST: HashMap<&'static str, Command> = hashmap![
    "quit" =>
    Command {
      exec: quit,
      argument_min: 0,
      argument_max: 0,
      channel: None,
      usage: String::from("Usage : @BOT quit"),
    },
    "reminder" =>
    Command {
      exec: Event::add_reminder,
      argument_min: 4,
      argument_max: 5,
      channel: None	,
      usage: String::from("Usage : @BOT reminder NAME DATE(MONTH-DAY:HOURS:MINUTES) MESSAGE CHANNEL REPEAT(delay in minutes)"),
    },
    "countdown" =>
    Command {
      exec: Event::add_countdown,
      argument_min: 6,
      argument_max: 6,
      channel: None	,
      usage: String::from("Usage : @BOT countdown NAME START_DATE(MONTH-DAY:HOURS) END_DATE(MONTH-DAY:HOURS) DELAY_OF_REPETITION(minutes) MESSAGE CHANNEL"),
    },
    "attack" =>
    Command {
      exec: attack_lauch,
      argument_min: 1,
      argument_max: 1,
      channel: None	,
      usage: String::from("Usage : @BOT countdown NAME START_DATE(MONTH-DAY:HOURS) END_DATE(MONTH-DAY:HOURS) DELAY_OF_REPETITION(minutes) MESSAGE CHANNEL"),
    }
  ];
}

fn attack_lauch(args: &Vec<&str>) -> String {
  ATTACKED.write().clear();
  ATTACKED.write().push_str(args[1]);
  format!("Prepare yourself {} !", args[1])
}

fn quit(_args: &Vec<&str>) -> String {
  println!("Quitting.");
  process::exit(0x0100);
}
