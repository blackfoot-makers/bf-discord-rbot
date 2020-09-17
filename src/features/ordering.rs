use crate::core::{
  commands::{CallBackParams, CallbackReturn},
  parse,
  process::{CACHE, HTTP_STATIC},
};
use serenity::{
  model::{
    channel::{ChannelType, GuildChannel},
    id::{ChannelId, GuildId},
  },
  prelude::*,
};
use std::collections::HashMap;
use std::sync::Arc;

pub fn ordering_channel_command(params: CallBackParams) -> CallbackReturn {
  let category: u64 = if params.args.len() == 2 {
    match parse::discord_str_to_id(params.args[1], Some(parse::DiscordIds::Channel)) {
      Ok((id, _)) => id,
      Err(error) => return Ok(Some(error)),
    }
  } else {
    0
  };
  let gid = params.message.guild_id.unwrap();
  let (ordering, func) = match guild_chanels_ordering(gid, category) {
    Some(res) => res,
    None => return Ok(Some(String::from("Channels are already ordered"))),
  };
  crate::core::validation::validate_command(&ordering, params.message, params.context, func);
  Ok(None)
}

pub fn move_channels(chanid: u64, position: u64) {
  let cache_lock = CACHE.write().clone();

  let cache = cache_lock.read();
  match cache.guild_channel(ChannelId(chanid)) {
    Some(channel) => {
      let mut channel = channel.write();
      println!("{}>{}", channel.name(), channel.position);
      let http = HTTP_STATIC.write().clone().unwrap();
      if let Err(why) = channel.edit(&http, |chan| chan.position(position)) {
        println!("Unable to edit channel {}:\n{}", channel.name, why);
      }
    }
    None => println!("Channel {} not found", chanid),
  }
}

fn ordering_channels_type(
  channels: &mut HashMap<ChannelId, Arc<RwLock<GuildChannel>>>,
  chantype: ChannelType,
  category: u64,
) -> (String, Vec<ChannelId>) {
  let mut channels: Vec<_> = channels
    .iter()
    .filter(|(_, chan)| {
      let chan = chan.read();
      chan.kind == chantype
        && match chan.category_id {
          Some(chan) => chan == category,
          None => category == 0,
        }
    })
    .collect();
  channels.sort_by(|chan, chan2| chan.1.read().name.cmp(&chan2.1.read().name));
  let mut display = String::new();
  let mut ordered_channels: Vec<ChannelId> = Vec::new();
  for (index, (_, chan)) in channels.iter().enumerate() {
    let channel = chan.read();
    if channel.position != index as i64 {
      display.push_str(&*format!(
        "[{}] {} => {}\n",
        channel.name(),
        channel.position,
        index
      ));
    }
    ordered_channels.push(channel.id);
  }
  (display, ordered_channels)
}

fn ordering_channels_type_apply(new_order: Vec<ChannelId>) {
  let cache_lock = CACHE.write();
  let cache = cache_lock.read();
  for (index, channelid) in new_order.iter().enumerate() {
    let channel = cache.guild_channel(channelid).unwrap();
    let mut channel_mut = channel.write();
    if channel_mut.position != index as i64 {
      let http = HTTP_STATIC.write().clone().unwrap();
      if let Err(why) = channel_mut.edit(&http, |chan| chan.position(index as u64)) {
        // TODO: Should tell the user about it
        println!("Unable to edit channel {}:\n{}", channel_mut.name, why);
      }
    }
  }
}

pub fn guild_chanels_ordering(
  gid: GuildId,
  category: u64,
) -> Option<(String, Box<dyn FnOnce() + Send + Sync>)> {
  let cache_lock = CACHE.write();
  let cache = cache_lock.read();
  let ordering = match cache.guild(gid) {
    Some(guild) => {
      let channels = &mut guild.write().channels;
      (
        ordering_channels_type(channels, ChannelType::Text, category),
        ordering_channels_type(channels, ChannelType::Voice, category),
      )
    }
    None => panic!("Guild not found"),
  };
  let texts_chans = ordering.0;
  let voices_chans = ordering.1;
  if texts_chans.0.is_empty() && voices_chans.0.is_empty() {
    return None;
  }
  let preview_reply = format!(
    "Order prevision:\nTexts:\n{}\nVoices:\n{}",
    texts_chans.0, voices_chans.0
  );
  let func = move || {
    ordering_channels_type_apply(texts_chans.1);
    ordering_channels_type_apply(voices_chans.1);
  };

  Some((preview_reply, Box::new(func)))
}
