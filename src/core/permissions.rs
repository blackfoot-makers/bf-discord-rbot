use crate::constants::discordids;
use crate::database;
use serenity::{
  model::{
    channel::{Channel, Message},
    id::RoleId,
  },
  prelude::*,
};
use std::str::FromStr;

pub fn is_user_allowed(
  context: &Context,
  expected: database::Role,
  message: &Message,
) -> (bool, database::Role) {
  let mut dbrole;
  {
    let db_instance = database::INSTANCE.read().unwrap();
    let user: &database::User = db_instance
      .user_search(*message.author.id.as_u64())
      .unwrap();

    dbrole = match database::Role::from_str(&*user.role) {
      Err(e) => {
        println!("Error {}", e);
        return (false, database::Role::Guest);
      }
      Ok(role) => role,
    };
  }
  // Only checking/updating for user or guests
  if dbrole <= database::Role::User {
    if let Channel::Guild(guildchan) = message.channel(&context.cache).unwrap() {
      let has_discord_role = message
        .author
        .has_role(
          &context.http,
          guildchan.read().guild_id,
          RoleId(discordids::USER_ROLE),
        )
        .unwrap();
      if let Some(newrole) = if has_discord_role && dbrole != database::Role::User {
        Some(database::Role::User)
      } else if !has_discord_role && dbrole != database::Role::Guest {
        Some(database::Role::Guest)
      } else {
        None
      } {
        let mut db_instance = database::INSTANCE.write().unwrap();
        db_instance.user_role_update(*message.author.id.as_u64(), newrole);
        dbrole = newrole;
      }
    }
  }
  (dbrole >= expected, dbrole)
}
