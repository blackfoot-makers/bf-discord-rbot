use crate::core::{
  commands::{CallBackParams, CallbackReturn},
  parse,
  validation::{self, ValidationCallback},
};
use futures::FutureExt;
use procedural_macros::command;
use serenity::{
  model::{
    channel::{ChannelType, GuildChannel},
    id::{ChannelId, GuildId},
  },
  prelude::*,
};

use super::archivage::filter_guild_channel;

#[command]
pub async fn ordering_channel_command(params: CallBackParams) -> CallbackReturn {
  let category: u64 = if params.args.len() == 2 {
    match parse::discord_str_to_id(&params.args[1], Some(parse::DiscordIds::Channel)) {
      Ok((id, _)) => id,
      Err(error) => return Ok(Some(error)),
    }
  } else {
    0
  };
  let gid = params.message.guild_id.unwrap();
  let (ordering, func) = match guild_chanels_ordering(gid, category, params.context).await {
    Some(res) => res,
    None => return Ok(Some(String::from("Channels are already ordered"))),
  };
  validation::validate_command(&ordering, params.message, params.context, func).await;
  Ok(None)
}

fn ordering_channels_type(
  channels: &[GuildChannel],
  chantype: ChannelType,
  category: u64,
) -> (String, Vec<ChannelId>) {
  let mut channels: Vec<_> = channels
    .iter()
    .filter(|chan| {
      chan.kind == chantype
        && match chan.parent_id {
          Some(chan) => chan == category,
          None => category == 0,
        }
    })
    .collect();
  channels.sort_by(|chan, chan2| chan.name.cmp(&chan2.name));
  let mut display = String::new();
  let mut ordered_channels: Vec<ChannelId> = Vec::new();
  for (index, channel) in channels.iter().enumerate() {
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

async fn ordering_channels_type_apply(new_order: Vec<ChannelId>, context: &Context) {
  let cache = &context.cache;
  for (index, channelid) in new_order.iter().enumerate() {
    let mut channel = cache.guild_channel(channelid).unwrap();
    if channel.position != index as i64 {
      if let Err(why) = channel
        .edit(&context.http, |chan| chan.position(index as u64))
        .await
      {
        // TODO: Should tell the user about it
        println!("Unable to edit channel {}:\n{}", channel.name, why);
      }
    }
  }
}

pub async fn guild_chanels_ordering<'fut>(
  gid: GuildId,
  category: u64,
  context: &Context,
) -> Option<(String, ValidationCallback)> {
  let cache = &context.cache;
  let ordering = match cache.guild(gid) {
    Some(guild) => {
      let channels = filter_guild_channel(guild.channels);
      (
        ordering_channels_type(&channels, ChannelType::Text, category),
        ordering_channels_type(&channels, ChannelType::Voice, category),
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
  let context_clone = context.clone();
  let func = || {
    async move {
      ordering_channels_type_apply(texts_chans.1, &context_clone).await;
      ordering_channels_type_apply(voices_chans.1, &context_clone).await;
    }
    .boxed()
  };

  Some((preview_reply, Box::new(func)))
}
