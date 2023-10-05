//! Handle the connection with discord and it's events.
use std::{collections::HashMap, error::Error, fmt::Write, process, str::FromStr};

use super::{parse, slash_command};
use crate::features::anyone::anyone;
use crate::features::calendar::check_calendar;
use crate::features::{
  archivage, emoji, funny, invite_action, ordering, project_manager, renaming,
};
use crate::{
  database::{NewStorage, Role, StorageDataType, INSTANCE},
  features::events,
};
use procedural_macros::command;
use serenity::{futures::future::BoxFuture, FutureExt};
use serenity::{
  model::channel::Message,
  model::{gateway::Activity, id::ChannelId},
  prelude::*,
};

pub struct CallBackParams<'a> {
  pub args: &'a [String],
  pub message: &'a Message,
  pub context: &'a Context,
}
pub type CallbackReturn<'fut> =
  BoxFuture<'fut, Result<Option<String>, Box<dyn Error + Send + Sync>>>;
type Callback = fn(CallBackParams) -> CallbackReturn;

/// Struct that old Traits Implementations to Handle the different events send by discord.
pub struct Command {
  pub exec: Callback,
  pub argument_min: usize,
  pub argument_max: usize,
  pub channel: Option<ChannelId>,
  pub usage: &'static str,
  pub permission: Role,
}

const INTRODUCE: &str = "Hello, i am a BOT. i was designed to peek over you conversations and make very weird comments. i don't have any purpose yet, but you can ask me about the weather";
const MOM_RFC: &str = "```\
- It must be an insult or a degrading comment
- To be validated the phrase incrimating and changing the mom being targeted has to be writed up in the #confidentiel channel vote for
- The insult toward a mom must be dirrect
- The mom is reseted after 1 week, and can also be reseted by insulting someone else mom with another computer that was left unlocked or by buying pastries
```";

lazy_static! {
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
    "ok boomer" => "Ok millennial",
    "fedora" => "https://camo.githubusercontent.com/98c193cbace1f9ce312fdf8e1e54da111ca6fc1481a460fe7a4be75be4cc4caf/68747470733a2f2f63646e2e646973636f72646170702e636f6d2f6174746163686d656e74732f3537333533313630333730343437393734352f3930363632353736323936393431393832362f494d475f32303231313130365f3230323634332e6a7067"
  ];
  pub static ref CONTAIN_REACTION_LIST: HashMap<&'static str, char> = hashmap![
    "👊" => '👊',
    "licorne" => '🦄',
    "leslie" => '🦄',
    "max" => '🍌',
    "retard" => '⌚',
    "pm" => '🐱'
  ];
  pub static ref COMMANDS_LIST: HashMap<&'static str, Command> = hashmap![
    "quit" =>
    Command {
      exec: |_| -> CallbackReturn { process::exit(0x0100) },
      argument_min: 0,
      argument_max: 0,
      channel: None,
      usage: "@BOT quit",
      permission: Role::Admin,
    },
    "send_message" =>
    Command {
      exec: manual_send_message,
      argument_min: 2,
      argument_max: 2,
      channel: None,
      usage: "@BOT send_message <#channelid> <@who>",
      permission: Role::Admin,
    },
    "users" =>
    Command {
      exec: |_| -> CallbackReturn { async move { Ok(Some(format!("{:?}", INSTANCE.write().unwrap().users))) }.boxed() },
      argument_min: 0,
      argument_max: 0,
      channel: None,
      usage: "@BOT users",
      permission: Role::Admin,
    },
    "slash-command-set" =>
    Command {
      exec: slash_command::set,
      argument_min: 0,
      argument_max: 0,
      channel: None,
      usage: "@BOT quit",
      permission: Role::Admin,
    },
    "promote" =>
    Command {
      exec: promote_user,
      argument_min: 2,
      argument_max: 2,
      channel: None,
      usage: "@BOT promote <@user> <role>",
      permission: Role::Admin,
    },
    "set-activity" =>
    Command {
      exec: set_activity,
      argument_min: 1,
      argument_max: 1,
      channel: None,
      usage: "@BOT set-activity <ACTIVITY_NAME>",
      permission: Role::User,
    },
    "edit" =>
    Command {
      exec: modify_message,
      argument_min: 2,
      argument_max: 3,
      channel: None,
      usage: "@BOT edit [<#channel>] <message_id> \"<new content>\"",
      permission: Role::User,
    },
    "create-project" =>
    Command {
      exec: project_manager::create,
      argument_min: 1,
      argument_max: 7,
      channel: None,
      usage: "@BOT create-project <name> [codex=<codex>, client=<client>, lead=<Lead>, deadline=<Deadline>, description=<Brief projet>, contexte=<Contexte>]",
      permission: Role::User,
    },
    "add-project" =>
    Command {
      exec: project_manager::add,
      argument_min: 2,
      argument_max: 8,
      channel: None,
      usage: "@BOT add-project <#channel_id> <name> [codex=<codex>, client=<client>, lead=<Lead>, deadline=<Deadline>, description=<Brief projet>, contexte=<Contexte>]",
      permission: Role::User,
    },
    "delete-project" =>
    Command {
      exec: project_manager::delete,
      argument_min: 1,
      argument_max: 1,
      channel: None,
      usage: "@BOT delete-project <name>",
      permission: Role::User,
    },
    "add" =>
    Command {
      exec: project_manager::add_user,
      argument_min: 1,
      argument_max: 1,
      channel: None,
      usage: "@BOT add <@user>",
      permission: Role::User,
    },
    "remove" =>
    Command {
      exec: project_manager::remove_user,
      argument_min: 1,
      argument_max: 1,
      channel: None,
      usage: "@BOT remove <@user>",
      permission: Role::User,
    },
    "project-clear-user" =>
    Command {
    exec: project_manager::remove_user_from_all,
      argument_min: 1,
      argument_max: 1,
      channel: None,
      usage: "@BOT project-clear-user <User>",
      permission: Role::Admin,
    },
    "invite" =>
    Command {
      exec: invite_action::create,
      argument_min: 2,
      argument_max: 3,
      channel: None,
      usage: "@BOT invite [<#invitecode>] <role AND OR channel>",
      permission: Role::User,
    },
    "archivage" =>
    Command {
      exec: archivage::archive_channels_command,
      argument_min: 0,
      argument_max: 1,
      channel: None,
      usage: "@BOT archivage [<category>]",
      permission: Role::Admin,
    },
    "ordering" =>
    Command {
      exec: ordering::ordering_channel_command,
      argument_min: 0,
      argument_max: 1,
      channel: None,
      usage: "@BOT ordering [<category>]",
      permission: Role::Admin,
    },
    "remindme" =>
    Command {
      exec: events::remind_me,
      argument_min: 2,
      argument_max: 2,
      channel: None,
      usage: "@BOT remindme <WHEN ex: 1minute,1m,10h,5days> <CONTENT>",
      permission: Role::User,
    },
    "attack" =>
    Command {
      exec: funny::attack_lauch,
      argument_min: 1,
      argument_max: 1,
      channel: None,
      usage: "@BOT attack <@user>",
      permission: Role::User,
    },
    "mom-change" =>
    Command {
      exec: funny::mom_change,
      argument_min: 1,
      argument_max: 1,
      channel: None,
      usage: "@BOT momchange <@user>",
      permission: Role::User,
    },
    "mom" =>
    Command {
      exec: funny::which_mom,
      argument_min: 0,
      argument_max: 0,
      channel: None,
      usage: "@BOT mom",
      permission: Role::User,
    },
    "cat" =>
    Command {
      exec: funny::get_cat_pic,
      argument_min: 0,
      argument_max: 0,
      channel: None,
      usage: "@BOT cat",
      permission: Role::Guest,
    },
    "rename" =>
    Command {
      exec: renaming::rename,
      argument_min: 2,
      argument_max: 3,
      channel: None,
      usage: "@BOT rename <@user> <new nickname> [<guild>]",
      permission: Role::User,
    },
    "emoji-add" =>
    Command {
      exec: emoji::add,
      argument_min: 1,
      argument_max: 1,
      channel: None,
      usage: "@BOT emoji-add <custom emoji>",
      permission: Role::User,
    },
    "block" =>
    Command {
      exec: block_user,
      argument_min: 1,
      argument_max: 1,
      channel: None,
      usage: "@BOT block <user>",
      permission: Role::Admin,
    },
    "help" =>
    Command {
      exec: print_help,
      argument_min: 0,
      argument_max: 0,
      channel: None,
      usage: "@BOT help",
      permission: Role::Guest,
    },
    "anyone" =>
    Command {
      exec: anyone,
      argument_min: 0,
      argument_max: 1,
      channel: None,
      usage: "@BOT anyone <message>",
      permission: Role::User,
    },
    "check-calendar" =>
    Command {
      exec: check_calendar,
      argument_min: 0,
      argument_max: 1,
      channel: None,
      usage: "@BOT check_calendar <date = MM/AAAA>",
      permission: Role::User,
    },
    "emoji-steal" =>
    Command {
      exec: emoji::emoji_steal,
      argument_min: 0,
      argument_max: 0,
      channel: None,
      usage: "@BOT emoji-steal (expected to be used as a reply to a message containing an emoji)",
      permission: Role::User,
    }
  ];
}

#[command]
async fn block_user(params: CallBackParams) -> CallbackReturn {
  let mut db_instance = INSTANCE.write().unwrap();

  let user_id = match parse::discord_str_to_id(&params.args[1], Some(parse::DiscordIds::User)) {
    Ok((userid, _)) => userid,
    Err(error) => return Ok(Some(error)),
  };

  db_instance.storage_add(NewStorage {
    date: None,
    dataid: Some(user_id as i64),
    datatype: StorageDataType::Blocked as i64,
    data: "",
  });
  Ok(Some(String::from(":ok:")))
}

#[derive(Queryable, Debug, Clone)]
pub struct Storage {
  pub id: i32,
  pub datatype: i64,
  pub dataid: Option<i64>,
  pub data: String,
  pub date: Option<std::time::SystemTime>,
}

#[command]
async fn print_help(_: CallBackParams) -> CallbackReturn {
  let mut result =
    String::from("Available commands: \nNAME => USAGE (<Args> [Optionals])| PERMISSION\n");
  let mut commands_name: Vec<&&str> = COMMANDS_LIST.iter().map(|c| c.0).collect();
  commands_name.sort();

  for name in commands_name {
    let command = &COMMANDS_LIST[name];
    writeln!(
      result,
      "{} => Usage: {} | {{{}}}",
      name, command.usage, command.permission
    )
    .expect("unable to append string");
  }
  Ok(Some(result))
}

#[command]
async fn promote_user(params: CallBackParams) -> CallbackReturn {
  let mut db_instance = INSTANCE.write().unwrap();

  let role = match Role::from_str(&params.args[2]) {
    Err(_) => return Ok(Some(String::from("Role not found"))),
    Ok(role) => role,
  };

  match parse::discord_str_to_id(&params.args[1], Some(parse::DiscordIds::User)) {
    Ok((userid, _)) => Ok(Some(db_instance.user_role_update(userid, role))),
    Err(error) => Ok(Some(error)),
  }
}

#[command]
async fn set_activity(params: CallBackParams) -> CallbackReturn {
  params
    .context
    .set_activity(Activity::playing(&params.args[1]))
    .await;
  let myname = &params.context.cache.current_user().name;
  Ok(Some(format!("{} is now {} !", myname, params.args[1])))
}

#[command]
async fn manual_send_message(params: CallBackParams) -> CallbackReturn {
  match parse::discord_str_to_id(&params.args[1], Some(parse::DiscordIds::Channel)) {
    Ok((chan_id, _)) => {
      ChannelId(chan_id)
        .send_message(&params.context.http, |m| m.content(&params.args[2]))
        .await
        .unwrap();
      Ok(Some(String::from(":ok:")))
    }
    Err(error) => Ok(Some(error)),
  }
}

#[command]
async fn modify_message(params: CallBackParams) -> CallbackReturn {
  let ((channel_id, _), (message_id, _)) = if params.args.len() == 4 {
    (
      parse::discord_str_to_id(&params.args[1], Some(parse::DiscordIds::Channel))?,
      parse::discord_str_to_id(&params.args[2], Some(parse::DiscordIds::Message))?,
    )
  } else {
    (
      (params.message.channel_id.0, parse::DiscordIds::Channel),
      parse::discord_str_to_id(&params.args[1], Some(parse::DiscordIds::Message))?,
    )
  };
  let mut message = ChannelId(channel_id)
    .message(&params.context.http, message_id)
    .await?;
  if message.is_own(&params.context.cache) {
    message
      .edit(&params.context.http, |message| {
        message.content(params.args.last().unwrap())
      })
      .await?;
    Ok(Some(String::from(":ok:")))
  } else {
    Ok(Some(String::from("I can only modify my own messages")))
  }
}
