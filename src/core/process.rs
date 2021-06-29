//! Handle the connection with discord and it's events.
use super::commands::{
  CallBackParams, COMMANDS_LIST, CONTAIN_MSG_LIST, CONTAIN_REACTION_LIST, TAG_MSG_LIST,
};
use super::permissions;
use crate::database;
use crate::features::funny::ATTACKED;
use log::{debug, error};
use regex::Regex;
use serenity::{
  model::channel::Message,
  model::id::{ChannelId, UserId},
  prelude::*,
};
use std::time::SystemTime;

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

pub async fn process_command(message_split: &[&str], message: &Message, ctx: &Context) -> bool {
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
        Ok(option) => {
          if let Some(reply) = option {
            if reply == ":ok:" {
              message.react(&ctx.http, '✅').await.unwrap();
            } else {
              message.reply(&ctx.http, reply).await.unwrap();
            }
          }
        }
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

pub async fn process_tag_msg(message_split: &[&str], message: &Message, ctx: &Context) -> bool {
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

#[test]
fn test_split_args() {
  assert_eq!(vec!["test=\"test\""], split_args("test=\"test\""));
  assert_eq!(vec!["test=test"], split_args("test=test"));
  assert_eq!(
    vec!["test=\"test jambon\""],
    split_args("test=\"test jambon\"")
  );
  assert_eq!(vec!["test=\"test \""], split_args("test=\"test jambon\""));

  assert_eq!(
    vec![
      r#"test="test jambon""#,
      r#"dd"#,
      r#""testos=1""#,
      r#"ddd"#,
      r#"d"#,
      r#"dd"#,
      r#"" d d d ""#
    ],
    split_args(r#"test="test jambon" dd "testos=1" ddd d dd " d d d " "#)
  );
}

// FIXME: this works well but doesn't take escaping into account.
pub fn split_args(input: &str) -> Vec<&str> {
  let regex_split = Regex::new(r#"([^"\s]*"[^"\n]*"[^"\s]*)|([^\s]+)"#).unwrap();
  regex_split.find_iter(input).map(|m| m.as_str()).collect()
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
pub fn filter_outannoying_messages(ctx: &Context, message: &Message) {
  if message.author.id != PM {
    return;
  }
  for annoying in FILTERED.iter() {
    if message.content.replace(annoying, "").trim().is_empty() {
      println!("Has been filtered !");
      let _ = message.delete(ctx);
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

pub fn database_update(message: &Message) {
  let mut db_instance = database::INSTANCE.write().unwrap();
  let author_id = *message.author.id.as_u64() as i64;
  if !db_instance.users.iter().any(|e| e.discordid == author_id) {
    db_instance.user_add(author_id, &*database::Role::Guest.to_string());
  }
  let time: SystemTime = SystemTime::from(message.timestamp);
  db_instance.message_add(database::Message {
    id: *message.id.as_u64() as i64,
    author: author_id,
    content: message.content.clone(),
    channel: *message.channel_id.as_u64() as i64,
    date: Some(time),
  });
}

// TODO: This is only working for 1 server as channel is static
use crate::constants::discordids::{ARCHIVE_CATEGORY, PROJECT_CATEGORY};
pub async fn archive_activity(ctx: &Context, message: &Message) {
  match message.channel(&ctx.cache).await {
    Some(channel) => {
      let channelid = channel.id().0;
      match channel.guild() {
        Some(mut channel) => {
          if let Some(category) = channel.category_id {
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
    None => error!("Channel not found in cache {}", message.channel_id),
  };
}

pub async fn trigger_inchannel(_: &Message, _: &Context) {}
