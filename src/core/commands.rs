//! Handle the connection with discord and it's events.
use futures::executor::block_on;
use reqwest;
use serde_json::{from_str, Value};
use serenity::{model::id::ChannelId, prelude::*};
use std::collections::HashMap;
use std::process;
use std::str::FromStr;

use crate::database::{Role, INSTANCE};
use crate::features::{calendar::google_calendar, notify::Event};

/// Struct that old Traits Implementations to Handle the different events send by discord.
pub struct Command {
  pub exec: fn(&Vec<&str>) -> String,
  pub argument_min: usize,
  pub argument_max: usize,
  pub channel: Option<ChannelId>,
  pub usage: String,
  pub permission: Role,
}

const INTRODUCE: &str = "Hello, i am a BOT. i was designed to peek over you conversations and make very weird comments. i don't have any purpose yet, but you can ask me about the weather";
const MOM_RFC: &str = "```\
- It must be an insult or a degrading comment
- To be validated the phrase incrimating and changing the mom being targeted has to be writed up in the #confidentiel channel vote for\
- The insult toward a mom must be dirrect\
- The mom is reseted after 1 week, and can also be reseted by insulting someone else mom with another computer that was left unlocked or by buing  pastries\
```";

lazy_static! {
  pub static ref ATTACKED: RwLock<String> = RwLock::new(String::new());
  pub static ref MOM: RwLock<String> = RwLock::new(String::new());
  pub static ref TAG_MSG_LIST: HashMap<&'static str, &'static str> = hashmap![
    "ping" => "pong",
    "introduce your self" => INTRODUCE,
    "introduce" => INTRODUCE,
    "mom rules" => MOM_RFC,
    "mom rfc" => MOM_RFC,
    "weather" => "The fuck do i know !",
    "what is today weather ?" => "The fuck do i know !",
    "what is today weather" => "The fuck do i know !",
    "bad" => "😢",
    "Bonjour !" => "Bonsoir !",
    "Bonjour" => "Bonsoir !",
    "🖕" => "🖕"
  ];
  pub static ref CONTAIN_MSG_LIST: HashMap<&'static str, &'static str> = hashmap![
    "keke" => "https://media.giphy.com/media/26ufju9mygxXmfjos/giphy.gif",
    "kéké" => "https://media.giphy.com/media/26ufju9mygxXmfjos/giphy.gif",
    "bad bot" => "😎",
    "hello there" => "https://i.kym-cdn.com/photos/images/newsfeed/001/475/420/c62.gif",
    "ok boomer" => "Ok millennial"
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
      exec: |_| -> String { process::exit(0x0100) },
      argument_min: 0,
      argument_max: 0,
      channel: None,
      usage: String::from("@BOT quit"),
      permission: Role::Admin,
    },
    "send_message" =>
    Command {
      exec: manual_send_message,
      argument_min: 2,
      argument_max: 2,
      channel: None,
      usage: String::from("@BOT send_message #channelid @who"),
      permission: Role::Admin,
    },
    "users" =>
    Command {
      exec: |_| -> String { format!("{:?}", INSTANCE.write().unwrap().users) },
      argument_min: 0,
      argument_max: 0,
      channel: None,
      usage: String::from("@BOT users"),
      permission: Role::Admin,
    },
    "promote" =>
    Command {
      exec: promote_user,
      argument_min: 2,
      argument_max: 2,
      channel: None,
      usage: String::from("@BOT promote @user role"),
      permission: Role::Admin,
    },
    "reminder" =>
    Command {
      exec: Event::add_reminder,
      argument_min: 4,
      argument_max: 5,
      channel: None,
      usage: String::from("@BOT reminder NAME DATE(MONTH-DAY:HOURS:MINUTES) MESSAGE CHANNEL REPEAT(delay in minutes)"),
      permission: Role::User,
    },
    "countdown" =>
    Command {
      exec: Event::add_countdown,
      argument_min: 6,
      argument_max: 6,
      channel: None,
      usage: String::from("@BOT countdown NAME START_DATE(MONTH-DAY:HOURS) END_DATE(MONTH-DAY:HOURS) DELAY_OF_REPETITION(minutes) MESSAGE CHANNEL"),
      permission: Role::User,
    },
    "attack" =>
    Command {
      exec: attack_lauch,
      argument_min: 1,
      argument_max: 1,
      channel: None,
      usage: String::from("@BOT attack @user"),
      permission: Role::User,
    },
    "mom change" =>
    Command {
      exec: mom_change,
      argument_min: 1,
      argument_max: 1,
      channel: None,
      usage: String::from("@BOT momchange @user"),
      permission: Role::User,
    },
    "mom" =>
    Command {
      exec: witch_mom,
      argument_min: 0,
      argument_max: 0,
      channel: None,
      usage: String::from("@BOT mom"),
      permission: Role::User,
    },
    "check" =>
    Command {
      exec: google_calendar,
      argument_min: 2,
      argument_max: 2,
      channel: None ,
      usage: String::from("Usage : @Bot check date(daily/monthly) type(calendar/codex)"),
      permission: Role::User,
    },
    "cat" =>
    Command {
      exec: get_cat_pic,
      argument_min: 0,
      argument_max: 0,
      channel: None,
      usage: String::from("@BOT cat"),
      permission: Role::Guest,
    },
    "help" =>
    Command {
      exec: print_help,
      argument_min: 0,
      argument_max: 0,
      channel: None,
      usage: String::from("@BOT help"),
      permission: Role::Guest,
    }
  ];
}

fn print_help(_args: &Vec<&str>) -> String {
  let mut result = String::from("Available commands: \nNAME => USAGE | PERMISSION\n");
  for (key, command) in COMMANDS_LIST.iter() {
    result.push_str(&*format!(
      "{} => Usage: {} | {{{}}}\n",
      key, command.usage, command.permission
    ))
  }
  result
}

fn promote_user(args: &Vec<&str>) -> String {
  let mut db_instance = INSTANCE.write().unwrap();

  let role = match Role::from_str(args[2]) {
    Err(_) => return String::from("Role not found"),
    Ok(role) => role,
  };

  let userid = args[1];
  let userid = userid[3..userid.len() - 1].parse::<u64>().unwrap();
  db_instance.user_role_update(&userid, &role)
}

fn get_cat_pic(_args: &Vec<&str>) -> String {
  let response = block_on(reqwest::get(
    "https://api.thecatapi.com/v1/images/search?size=full",
  ))
  .unwrap();
  let text = block_on(response.text()).unwrap();

  let v: Value = from_str(&text).unwrap();

  let url = v[0]["url"].clone();
  let result = &mut url.to_string();
  result.pop();
  String::from(&result[1..])
}

fn manual_send_message(args: &Vec<&str>) -> String {
  let http = super::process::HTTP_STATIC.read().clone().unwrap();

  let chan_id = args[1].parse::<u64>().unwrap();
  ChannelId(chan_id)
    .send_message(http, |m| m.content(args[2]))
    .unwrap();

  String::new()
}

fn attack_lauch(args: &Vec<&str>) -> String {
  ATTACKED.write().clear();
  ATTACKED.write().push_str(args[1]);
  format!("Prepare yourself {} !", args[1])
}

fn mom_change(args: &Vec<&str>) -> String {
  MOM.write().clear();
  MOM.write().push_str(args[1]);
  format!("It's your momas turn yourself {} !", args[1])
}

fn witch_mom(_args: &Vec<&str>) -> String {
  format!("It's currently {} mom's", MOM.read())
}
