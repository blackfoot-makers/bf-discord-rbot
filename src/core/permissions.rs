use crate::constants::discordids;
use crate::database;
use serenity::{
  model::{
    channel::{Channel, Message, PermissionOverwrite, PermissionOverwriteType},
    id::{RoleId, UserId},
    Permissions,
  },
  prelude::*,
};

use std::str::FromStr;

pub async fn is_user_allowed(
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
    if let Channel::Guild(guildchan) = message.channel(&context.cache).await.unwrap() {
      let has_discord_role = message
        .author
        .has_role(
          &context.http,
          guildchan.guild_id,
          RoleId(discordids::USER_ROLE),
        )
        .await
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

pub fn member_channel_read(user: UserId) -> PermissionOverwrite {
  let allow = Permissions::READ_MESSAGES;
  let deny = Permissions::empty();
  PermissionOverwrite {
    deny,
    allow,
    kind: PermissionOverwriteType::Member(user),
  }
}
