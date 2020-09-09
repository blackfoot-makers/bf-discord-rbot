use crate::core::{
  commands::{CallBackParams, CallbackReturn},
  parse,
  process::{CACHE, HTTP_STATIC},
};
use chrono::prelude::*;
use log::error;
use serenity::{
  model::{
    channel::{ChannelType, GuildChannel},
    id::{ChannelId, GuildId},
  },
  prelude::*,
};
use std::{collections::HashMap, sync::Arc};

pub fn archive_channels_command(params: CallBackParams) -> CallbackReturn {
  let category: u64 = if params.args.len() == 3 {
    match parse::discord_str_to_id(params.args[1], Some(parse::DiscordIds::Channel)) {
      Ok((id, _)) => id,
      Err(error) => return Ok(Some(error)),
    }
  } else {
    0
  };
  let gid = params.message.guild_id.unwrap();
  let (archivage, func) = match guild_chanels_archivage(gid, category) {
    Some(res) => res,
    None => return Ok(Some(String::from("Nothing to do"))),
  };
  crate::core::validation::validate_command(&archivage, params.message, params.context, func);
  Ok(None)
}

// TODO: This is only working for 1 server as channel is static
use crate::constants::discordids::ARCHIVE_CATEGORY;
pub fn move_channels_to_archive(chanids: Vec<u64>) {
  let cache_lock = CACHE.write().clone();

  let cache = cache_lock.read();
  for chanid in chanids {
    match cache.guild_channel(ChannelId(chanid)) {
      Some(channel) => {
        let mut channel = channel.write();
        let http = HTTP_STATIC.write().clone().unwrap();
        if let Err(why) = channel.edit(&http, |chan| chan.category(ChannelId(ARCHIVE_CATEGORY))) {
          // TODO: Should tell the user about it
          error!("Unable to edit channel {}:\n{}", channel.name, why);
        }
      }
      None => error!("Channel {} not found", chanid),
    }
  }
}

fn check_channels_activity(
  channels: &mut HashMap<ChannelId, Arc<RwLock<GuildChannel>>>,
  category: u64,
) -> (String, Vec<u64>) {
  let channels: Vec<_> = channels
    .iter()
    .filter(|(_, chan)| {
      let chan = chan.read();
      chan.kind == ChannelType::Text
        && match chan.category_id {
          Some(chan) => chan == category,
          None => category == 0,
        }
    })
    .collect();
  let mut display = String::new();
  let mut unactive_channels: Vec<u64> = Vec::new();
  for (_, channel) in channels.iter() {
    let channel = channel.read();
    let http = HTTP_STATIC.write().clone().unwrap();
    match channel.messages(http, |retriever| retriever.limit(1)) {
      Ok(messages) => {
        if let Some(message) = messages.first() {
          let now = Local::now();
          if now.num_days_from_ce() - message.timestamp.num_days_from_ce() > 30 {
            display.push_str(&*format!(
              "[{}] last message: {}\n",
              channel.name(),
              message.timestamp
            ));
            unactive_channels.push(channel.id.0);
          }
        }
      }
      // TODO: Should tell the user about it
      Err(_) => error!("Unable to read message of {}", channel.id),
    };
  }
  (display, unactive_channels)
}

pub fn guild_chanels_archivage(
  gid: GuildId,
  category: u64,
) -> Option<(String, Box<dyn FnOnce() + Send + Sync>)> {
  let cache_lock = CACHE.write();
  let cache = cache_lock.read();
  let unactive_channels = match cache.guild(gid) {
    Some(guild) => {
      let channels = &mut guild.write().channels;
      check_channels_activity(channels, category)
    }
    None => {
      error!("Guild not found");
      return None;
    }
  };
  if unactive_channels.1.is_empty() {
    return None;
  }
  let preview_reply = format!(
    "Unactive channels to move to archives:\n{}",
    unactive_channels.0
  );
  let func = move || {
    move_channels_to_archive(unactive_channels.1);
  };

  Some((preview_reply, Box::new(func)))
}
