use log::error;
use serenity::{
  model::{channel::Channel, guild::Guild},
  prelude::*,
};
use std::sync::Arc;

const BLACKFOOT_ID: u64 = 464779118857420811;
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
      None => Ok(context.cache.read().guild(BLACKFOOT_ID).unwrap()),
    },
    Channel::Guild(guildchan) => Ok(guildchan.read().guild(&context.cache).unwrap()),
    _ => Err(String::from("This doesn't work in this channel")),
  }
}
