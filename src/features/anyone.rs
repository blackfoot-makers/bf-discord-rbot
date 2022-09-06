use crate::{
  constants::discordids::USER_ROLE,
  core::{
    commands::{CallBackParams, CallbackReturn},
    parse,
  },
};
use log::error;
use procedural_macros::command;
use rand::Rng;
use serenity::{
  model::prelude::{Member, RoleId},
  prelude::Mentionable,
};

fn get_random_user(users: Vec<Member>) -> Member {
  let user_role = RoleId::from(USER_ROLE);
  let random_index = rand::thread_rng().gen_range(0..users.len());
  if !users[random_index].roles.contains(&user_role) {
    get_random_user(users)
  } else {
    users[random_index].clone()
  }
}

#[command]
pub async fn anyone(params: CallBackParams) -> CallbackReturn {
  let http = &params.context.http;
  let channel_id = params.message.channel_id;
  let guild = match parse::get_guild(channel_id, params.context, params.args.get(3)).await {
    Ok(guild) => guild,
    Err(error) => return Ok(Some(error)),
  };
  let users = match guild.members(http, None, None).await {
    Ok(users) => users,
    Err(error) => {
      error!("Anyone: member not found: {}", error);
      return Ok(Some(format!(
        "Unable to get users of the guild: {}",
        guild.name(&params.context.cache).unwrap(),
      )));
    }
  };
  let random_user = get_random_user(users);
  params
    .message
    .channel_id
    .send_message(http, |m| {
      m.content(format!("{} {}", random_user.mention(), params.args[1]))
    })
    .await
    .unwrap();
  Ok(Some(String::from(":ok:")))
}
