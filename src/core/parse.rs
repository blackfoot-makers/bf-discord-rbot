use crate::constants::discordids;
use log::error;
use serenity::{
  model::{channel::Channel, guild::Guild},
  prelude::*,
};
use std::sync::Arc;

pub fn get_main_guild(context: &Context) -> Arc<RwLock<Guild>> {
  context
    .cache
    .read()
    .guild(discordids::GUILD_ID)
    .expect("Unable to find main guild")
}

pub fn get_guild(
  channel: Channel,
  context: &Context,
  gid: Option<&str>,
) -> Result<Arc<RwLock<Guild>>, String> {
  match channel {
    Channel::Private(_) => match gid {
      Some(gid) => {
        let id = match gid.parse::<u64>() {
          Ok(id) => id,
          Err(parse_error) => {
            error!("{}", parse_error);
            return Err(String::from("Invalid guild id"));
          }
        };
        match context.cache.read().guild(id) {
          Some(guild) => Ok(guild),
          None => Err(format!("Guild: {} not found", gid)),
        }
      }
      None => Ok(get_main_guild(context)),
    },
    Channel::Guild(guildchan) => Ok(guildchan.read().guild(&context.cache).unwrap()),
    _ => Err(String::from("This doesn't work in this channel")),
  }
}

pub fn discord_str_to_id(id: &str) -> Result<u64, &str> {
  let size = id.len();
  if size < 18 {
    return Err("Unable to parse, text isn't an disocrd ID");
  }

  let result = if size == 18 {
    id.parse::<u64>().expect("Unable to parse Id, not numeric")
  } else {
    id[size - 19..size - 1]
      .parse::<u64>()
      .expect("Unable to parse Id, badly formated")
  };
  Ok(result)
}
