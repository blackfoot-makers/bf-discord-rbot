use crate::core::{
  commands::{CallBackParams, CallbackReturn},
  parse::{discord_str_to_id, DiscordIds},
  permissions::member_channel_read,
};
use crate::database::INSTANCE;
use log::warn;
use serenity::{
  model::{
    guild::Member,
    id::{ChannelId, GuildId, RoleId},
  },
  prelude::*,
};
use std::error;

pub fn on_new_member_check(ctx: Context, guild_id: &GuildId, member: &mut Member) {
  let invites = guild_id.invites(&ctx.http).unwrap();
  let mut db_instance = INSTANCE.write().unwrap();
  let mut single_used_invite = None;
  for invite in invites {
    let (invitediff, dbinvite) = db_instance
      .invite_update(invite.code.clone(), invite.uses as i32)
      .expect(&*format!("Unable to update invite: {} =>", &invite.code));
    if invitediff > 0 {
      if single_used_invite.is_some() || invitediff > 1 {
        return warn!("One or more invite used at a time, couldn't check for action");
      } else {
        single_used_invite = Some(dbinvite.clone());
      };
    }
  }
  if let Some(invite) = single_used_invite {
    if let Some(role) = invite.actionrole {
      member.add_role(&ctx.http, RoleId(role as u64)).unwrap();
    }
    if let Some(channel) = invite.actionchannel {
      let overwrite = member_channel_read(member.user_id());
      ChannelId(channel as u64)
        .create_permission(&ctx.http, &overwrite)
        .unwrap();
    }
  } else {
    warn!(
      "Wut no code used when joining the server?, member: {}",
      member.display_name()
    )
  }
}

fn parseCreateArgument(
  argument: &str,
) -> Result<Result<(u64, DiscordIds), String>, Box<dyn error::Error>> {
  let param1 = discord_str_to_id(argument, None)?;
  match param1.1 {
    DiscordIds::Role => Ok(Ok(param1)),
    DiscordIds::Channel => Ok(Ok(param1)),
    _ => Ok(Err(String::from(
      "Id provided should be a Channel or a Role",
    ))),
  }
}

pub fn create(params: CallBackParams) -> CallbackReturn {
  let role = None;
  let channel = None;
  Ok(Some(String::from("Done")))
}
