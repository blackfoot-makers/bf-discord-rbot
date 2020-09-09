use crate::constants::discordids;
use log::error;
use serenity::{
  model::{channel::Channel, guild::Guild},
  prelude::*,
};
use std::sync::Arc;
use strum_macros::Display;

#[derive(PartialEq, Debug, Display)]
pub enum DiscordIds {
  Message,
  Channel,
  Role,
  User,
}

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
  gid: Option<&&str>,
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

pub fn discord_str_to_id(
  id: &str,
  exepected_type: Option<DiscordIds>,
) -> Result<(u64, DiscordIds), String> {
  let size = id.len();
  const SIZEBIGINT: usize = 18;
  if size < SIZEBIGINT {
    return Err(String::from("Unable to parse, text isn't an disocrd ID"));
  }

  if size == SIZEBIGINT {
    let parsedid = id.parse::<u64>().expect("Unable to parse Id, not numeric");
    Ok((parsedid, DiscordIds::Channel))
  } else {
    let parsedid = id[size - (SIZEBIGINT + 1)..size - 1]
      .parse::<u64>()
      .expect("Unable to parse Id, badly formated");
    let identifier = &id[0..size - (SIZEBIGINT + 1)];
    let discordtype: DiscordIds = match identifier {
      "<@" | "<@!" => DiscordIds::User,
      "<#" => DiscordIds::Channel,
      "<@&" => DiscordIds::Role,
      _ => DiscordIds::Channel,
      // Channel can't be pinged so no identifier sadly
      // _ => return Err(&*format!("Incored type for discordid: {}", identifier)),
    };
    if let Some(expected) = exepected_type {
      if expected != discordtype {
        let msg = format!(
          "Mismatched type, expected: {}, got: {}",
          expected, discordtype
        );
        return Err(msg);
      }
    }
    Ok((parsedid, discordtype))
  }
}
