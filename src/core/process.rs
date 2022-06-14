//! Handle the connection with discord and it's events.
use super::commands::{
  CallBackParams, COMMANDS_LIST, CONTAIN_MSG_LIST, CONTAIN_REACTION_LIST, TAG_MSG_LIST,
};
use super::permissions;
use crate::core::parse::split_message_args;
use crate::database;
use crate::features::funny::ATTACKED;
use log::{debug, error};
use serenity::model::event::MessageUpdateEvent;
use serenity::{
  model::channel::Message,
  model::id::{ChannelId, UserId},
  prelude::*,
};
use std::process::exit;
use std::time::SystemTime;

pub async fn getbotid(ctx: &Context) -> UserId {
  ctx.cache.current_user_id()
}

pub async fn process_message(ctx: Context, message: Message) {
  if is_user_blocked(&ctx, &message).await {
    return;
  };
  personal_attack(&ctx, &message).await;
  annoy_channel(&ctx, &message).await;
  filter_outannoying_messages(&ctx, &message).await;

  //Check if i am tagged in the message else do the reactions
  // check for @me first so it's considered a command
  let botid = getbotid(&ctx).await.0;
  if message.content.starts_with(&*format!("<@!{}>", botid))
    || message.content.starts_with(&*format!("<@{}>", botid))
  {
    if attacked(&ctx, &message).await {
      return;
    }
    let line = message.content.clone();
    let mut message_split = split_message_args(&line);

    // Check if there is only the tag : "@bot"
    if message_split.len() == 1 {
      message
        .channel_id
        .say(&ctx.http, "What do you need ?")
        .await
        .unwrap();
      return;
    }
    // Removing tag
    message_split.remove(0);

    // will go through commands.rs definitions to try and execute the request
    if !process_tag_msg(&message_split, &message, &ctx).await
      && !process_command(&message_split, &message, &ctx).await
    {
      message
        .channel_id
        .say(&ctx.http, "How about a proper request ?")
        .await
        .unwrap();
    }
  } else {
    process_contains(&message, &ctx).await;
  }
  trigger_inchannel(&message, &ctx).await;
}

async fn allowed_channel(
  command_channel: Option<ChannelId>,
  message_channel: ChannelId,
  ctx: &Context,
) -> bool {
  match command_channel {
    Some(ref chan) => {
      if chan != &message_channel {
        message_channel
          .say(
            &ctx.http,
            format!(
              "I am not allowed to issue this command in this channel ! Use {} instead.",
              chan.mention()
            ),
          )
          .await
          .unwrap();
        return false;
      }
      true
    }
    None => true,
  }
}

pub async fn process_command(message_split: &[String], message: &Message, ctx: &Context) -> bool {
  for (key, command) in COMMANDS_LIST.iter() {
    if *key == message_split[0] {
      if !allowed_channel(command.channel, message.channel_id, ctx).await {
        return true;
      };
      let (allowed, role) = permissions::is_user_allowed(ctx, command.permission, message).await;
      if !allowed {
        message
          .channel_id
          .send_message(&ctx.http, |m| {
            m.content(format!("You({}) are not allowed to run this command", role))
          })
          .await
          .unwrap();
        return true;
      }
      // We remove default arguments: author and command name from the total
      let arguments_length = message_split.len() - 1;
      let result =
        if arguments_length >= command.argument_min && arguments_length <= command.argument_max {
          let params = CallBackParams {
            args: message_split,
            message,
            context: ctx,
          };
          (command.exec)(params).await
        } else {
          let why = if arguments_length >= command.argument_min {
            "Too many arguments"
          } else {
            "No enough arguments"
          };
          Ok(Some(format!("{}\nUsage: {}", why, command.usage)))
        };

      match result {
        Ok(Some(reply)) => {
          if reply == ":ok:" {
            message.react(&ctx.http, '✅').await.unwrap();
          } else {
            message.reply(&ctx.http, reply).await.unwrap();
          }
        }
        Ok(None) => {}
        Err(err) => {
          message
            .reply(&ctx.http, "Bipboop this is broken <@173013989180178432>")
            .await
            .unwrap();
          error!("Command Error: {} => {}", key, err);
        }
      }
      return true;
    }
  }
  false
}

pub async fn process_tag_msg(message_split: &[String], message: &Message, ctx: &Context) -> bool {
  for (key, reaction) in TAG_MSG_LIST.iter() {
    if *key == message_split[0] {
      message.channel_id.say(&ctx.http, reaction).await.unwrap();
      return true;
    }
  }
  false
}

pub async fn process_contains(message: &Message, ctx: &Context) {
  for (key, text) in CONTAIN_MSG_LIST.iter() {
    if message.content.contains(key) {
      message.channel_id.say(&ctx.http, *text).await.unwrap();
    }
  }

  for (key, reaction) in CONTAIN_REACTION_LIST.iter() {
    if message.content.contains(key) {
      message.react(ctx, *reaction).await.unwrap();
    }
  }
}

const CATS: [char; 12] = [
  '😺', '😸', '😹', '😻', '😼', '😽', '🙀', '😿', '😾', '🐈', '🐁', '🐭',
];
const KEYS: [char; 8] = ['🔑', '🗝', '🔏', '🔐', '🔒', '🔓', '🖱', '👓'];
use crate::constants::discordids::{
  ANNOYED_CHAN_CYBERGOD, ANNOYED_CHAN_HERDINGCHATTE, ANNOYED_CHAN_TESTBOT,
};
/// Anoying other channels
pub async fn annoy_channel(ctx: &Context, message: &Message) {
  if message.channel_id == ChannelId(ANNOYED_CHAN_HERDINGCHATTE) {
    let random_active = rand::random::<usize>() % 10;
    if random_active == 0 {
      let random_icon = rand::random::<usize>() % CATS.len();
      message.react(ctx, CATS[random_icon]).await.unwrap();
    }
  }
  if message.channel_id == ChannelId(ANNOYED_CHAN_CYBERGOD) {
    let random_active = rand::random::<usize>() % 10;
    if random_active == 0 {
      let random_icon = rand::random::<usize>() % KEYS.len();
      message.react(ctx, KEYS[random_icon]).await.unwrap();
    }
  }
  if message.channel_id == ChannelId(ANNOYED_CHAN_TESTBOT) {
    let random_active = rand::random::<usize>() % 10;
    if random_active == 0 {
      let random_icon = rand::random::<usize>() % KEYS.len();
      message.react(ctx, KEYS[random_icon]).await.unwrap();
    }
  }
}

const FILTERED: [&str; 1] = ["🔥"];
const PM: UserId = UserId(365228504817729539);
pub async fn filter_outannoying_messages(ctx: &Context, message: &Message) {
  if message.author.id != PM {
    return;
  }
  for annoying in FILTERED.iter() {
    if message.content.replace(annoying, "").trim().is_empty() {
      println!("Has been filtered !");
      let _ = message.delete(ctx).await;
    }
  }
}

pub async fn personal_attack(ctx: &Context, message: &Message) {
  if message.author.name == *ATTACKED.read().await {
    const ANNOYING: [char; 11] = [
      '🐧', '💩', '🍌', '💣', '👾', '🐔', '📛', '🔥', '‼', '⚡', '⚠',
    ];
    let random1 = rand::random::<usize>() % ANNOYING.len();
    let random2 = rand::random::<usize>() % ANNOYING.len();
    message.react(ctx, ANNOYING[random1]).await.unwrap();
    message.react(ctx, ANNOYING[random2]).await.unwrap();
  }
}

pub async fn attacked(ctx: &Context, message: &Message) -> bool {
  const ANNOYING_MESSAGE: [&str; 6] = [
    "Ah oui mais y'a JPO",
    "Vous pourriez faire ça vous meme s'il vous plaît ? Je suis occupé",
    "Avant, Faut laver les vitres les gars",
    "Ah mais vous faites quoi ?",
    "Non mais tu as vu le jeu qui est sorti ?",
    "Je bosse sur un projet super innovant en ce moment, j'ai pas le temps",
  ];

  if message.author.name == *ATTACKED.read().await {
    let random = rand::random::<usize>() % 6;
    message
      .channel_id
      .say(&ctx.http, ANNOYING_MESSAGE[random])
      .await
      .unwrap();
    return true;
  }
  false
}

pub async fn is_user_blocked(_: &Context, message: &Message) -> bool {
  let db_instance = database::INSTANCE.read().unwrap();
  let blocked_users = db_instance.filter_storage_type(database::StorageDataType::Blocked);
  let user_id = message.author.id.0;
  return blocked_users
    .iter()
    .any(|x| x.dataid.unwrap() == user_id as i64);
}

impl From<&Message> for database::Message {
  fn from(val: &Message) -> Self {
    let author_id = *val.author.id.as_u64() as i64;
    let time: SystemTime = SystemTime::from(*val.timestamp);

    database::Message {
      id: *val.id.as_u64() as i64,
      author: author_id,
      content: val.content.clone(),
      channel: *val.channel_id.as_u64() as i64,
      date: Some(time),
    }
  }
}

impl From<&MessageUpdateEvent> for database::Message {
  fn from(val: &MessageUpdateEvent) -> Self {
    let author_id = if let Some(author) = &val.author {
      author.id.0 as i64
    } else {
      0
    };
    let time = if let Some(timestamp) = val.timestamp {
      SystemTime::from(*timestamp)
    } else {
      SystemTime::now()
    };

    database::Message {
      id: *val.id.as_u64() as i64,
      author: author_id,
      content: val.content.as_ref().unwrap_or(&String::new()).clone(),
      channel: *val.channel_id.as_u64() as i64,
      date: Some(time),
    }
  }
}

pub fn database_update(message: database::Message, is_edit: bool) {
  let mut db_instance = match database::INSTANCE.write() {
    Ok(instance) => instance,
    Err(poison_error) => {
      error!("Unable to unlock RWLock for db instance, {}", poison_error);
      exit(1)
    }
  };

  if is_edit
    && db_instance
      .messages
      .iter()
      .any(|db_message| message.id == db_message.id)
  {
    db_instance.message_edit_add(database::NewMessageEdit {
      id: None,
      parrent_message_id: message.id,
      author: message.author,
      channel: message.channel,
      content: message.content,
      date: message.date,
    });
  } else {
    if !db_instance
      .users
      .iter()
      .any(|e| e.discordid == message.author)
    {
      db_instance.user_add(message.author, &*database::Role::Guest.to_string());
    }
    db_instance.message_add(message);
  }
}

// TODO: This is only working for 1 server as channel is static
use crate::constants::discordids::{ARCHIVE_CATEGORY, PROJECT_CATEGORY};
pub async fn archive_activity(ctx: &Context, message: &Message) {
  match message.channel(&ctx.http).await {
    Ok(channel) => {
      let channelid = channel.id().0;
      match channel.guild() {
        Some(mut channel) => {
          if let Some(category) = channel.parent_id {
            if category == ARCHIVE_CATEGORY {
              channel
                .edit(&ctx.http, |edit| edit.category(ChannelId(PROJECT_CATEGORY)))
                .await
                .expect(&*format!(
                  "Unable to edit channel:{} to unarchive",
                  channel.id
                ));
            }
          }
        }
        None => debug!("Channel {} isn't in a guild", channelid),
      };
    }
    Err(_) => error!("Channel not found in cache {}", message.channel_id),
  };
}

pub async fn trigger_inchannel(_: &Message, _: &Context) {}
