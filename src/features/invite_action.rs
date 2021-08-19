use crate::core::{
  commands::{CallBackParams, CallbackReturn},
  parse::{discord_str_to_id, DiscordIds},
  permissions::member_channel_read,
};
use crate::database::INSTANCE;
use log::warn;
use procedural_macros::command;
use serenity::{
  model::{
    guild::Member,
    id::{ChannelId, GuildId, RoleId},
  },
  prelude::*,
};

pub async fn on_new_member_check(ctx: Context, guild_id: &GuildId, member: &mut Member) {
  let invites = guild_id.invites(&ctx.http).await.unwrap();
  let mut single_used_invite = None;
  {
    let mut db_instance = INSTANCE.write().unwrap();
    for invite in invites {
      let (invitediff, dbinvite) = db_instance
        .invite_update(invite.code.clone(), Some(invite.uses as i32), None, None)
        .expect(&*format!("Unable to update invite: {} =>", &invite.code));
      if invitediff > 0 {
        if single_used_invite.is_some() || invitediff > 1 {
          return warn!("One or more invite used at a time, couldn't check for action");
        } else {
          single_used_invite = Some(dbinvite.clone());
        };
      }
    }
  }

  println!("DEBUG invite found: {:#?}", single_used_invite);
  if let Some(invite) = single_used_invite {
    if let Some(role) = invite.actionrole {
      member
        .add_role(&ctx.http, RoleId(role as u64))
        .await
        .unwrap();
    }
    if let Some(channel) = invite.actionchannel {
      let overwrite = member_channel_read(member.user.id);
      ChannelId(channel as u64)
        .create_permission(&ctx.http, &overwrite)
        .await
        .unwrap();
    }
  } else {
    warn!(
      "Wut no code used when joining the server?, member: {}",
      member.display_name()
    )
  }
}

fn parse_create_argument(
  argument: &str,
  role: &mut Option<i64>,
  channel: &mut Option<i64>,
) -> Result<(), String> {
  let param1 = discord_str_to_id(argument, None)?;
  let result = match param1.1 {
    DiscordIds::Role => param1,
    DiscordIds::Channel => param1,
    _ => return Err(String::from("Id provided should be a Channel or a Role")),
  };

  match result.1 {
    DiscordIds::Role => {
      if role.is_some() {
        return Err(String::from("Role was already specified"));
      }
      *role = Some(result.0 as i64);
    }
    DiscordIds::Channel => {
      if channel.is_some() {
        return Err(String::from("Channel was already specified"));
      }
      *channel = Some(result.0 as i64)
    }
    _ => unreachable!(),
  }
  Ok(())
}

#[command]
pub async fn create(params: CallBackParams) -> CallbackReturn {
  let mut role = None;
  let mut channel = None;
  if let Err(err) = parse_create_argument(&params.args[2], &mut role, &mut channel) {
    return Ok(Some(err));
  }
  if params.args.len() == 4 {
    if let Err(err) = parse_create_argument(&params.args[3], &mut role, &mut channel) {
      return Ok(Some(err));
    }
  }
  {
    let mut db_instance = INSTANCE.write().unwrap();
    let code = &params.args[1].replace("https://discord.gg/", "");
    if code.len() != 8 {
      return Ok(Some(format!("Invite code: {}, isn't valid", code)));
    }

    db_instance
      .invite_update(code.clone(), None, channel, role)
      .expect(&*format!("Unable to update invite: {} =>", code));
  }
  Ok(Some(String::from("Done")))
}
