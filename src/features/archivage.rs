use crate::core::{
  commands::{CallBackParams, CallbackReturn},
  parse,
  validation::{self, ValidationCallback},
};
use chrono::prelude::*;
use futures::FutureExt;
use log::error;
use procedural_macros::command;
use serenity::{
  model::{
    channel::{ChannelType, GuildChannel},
    id::{ChannelId, GuildId},
  },
  prelude::*,
};
use std::collections::HashMap;

#[command]
pub async fn archive_channels_command(params: CallBackParams) -> CallbackReturn {
  let category: u64 = if params.args.len() == 2 {
    match parse::discord_str_to_id(params.args[1], Some(parse::DiscordIds::Channel)) {
      Ok((id, _)) => id,
      Err(error) => return Ok(Some(error)),
    }
  } else {
    0
  };
  let gid = params.message.guild_id.unwrap();
  let (archivage, func) = match guild_chanels_archivage(gid, category, params.context).await {
    Some(res) => res,
    None => return Ok(Some(String::from("Nothing to do"))),
  };

  validation::validate_command(&archivage, params.message, params.context, func).await;
  Ok(None)
}

// TODO: This is only working for 1 server as channel is static
use crate::constants::discordids::ARCHIVE_CATEGORY;
pub async fn move_channels_to_archive(chanids: Vec<u64>, context: &Context) {
  let cache = &context.cache;
  for chanid in chanids {
    match cache.guild_channel(ChannelId(chanid)).await {
      Some(mut channel) => {
        if let Err(why) = channel
          .edit(&context.http, |chan| {
            chan.category(ChannelId(ARCHIVE_CATEGORY))
          })
          .await
        {
          // TODO: Should tell the user about it
          error!("Unable to edit channel {}:\n{}", channel.name, why);
        }
      }
      None => error!("Channel {} not found", chanid),
    }
  }
}

async fn check_channels_activity(
  channels: &mut HashMap<ChannelId, GuildChannel>,
  category: u64,
  context: &Context,
) -> (String, Vec<u64>) {
  let channels: Vec<_> = channels
    .iter()
    .filter(|(_, chan)| {
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
    match channel
      .messages(&context.http, |retriever| retriever.limit(1))
      .await
    {
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

pub async fn guild_chanels_archivage<'fut>(
  gid: GuildId,
  category: u64,
  context: &Context,
) -> Option<(String, ValidationCallback)> {
  let cache = context.cache.clone();
  let unactive_channels = match cache.guild(gid).await {
    Some(mut guild) => {
      let channels = &mut guild.channels;
      check_channels_activity(channels, category, context).await
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
  let context_clone = context.clone();
  let func = || {
    async move {
      move_channels_to_archive(unactive_channels.1, &context_clone).await;
    }
    .boxed()
  };

  Some((preview_reply, Box::new(func)))
}
