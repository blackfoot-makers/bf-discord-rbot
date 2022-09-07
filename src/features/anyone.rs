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

fn get_random_user(mut users: Vec<Member>) -> Member {
  let user_role = RoleId::from(USER_ROLE);
  users.retain(|user| user.roles.contains(&user_role));
  let random_index = rand::thread_rng().gen_range(0..users.len());
  users[random_index].clone()
}

#[command]
pub async fn anyone(params: CallBackParams) -> CallbackReturn {
  let http = &params.context.http;
  let channel_id = params.message.channel_id;
  let guild = match parse::get_guild(channel_id, params.context, None).await {
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
  let content = match params.args.len() {
    1 => format!("{} is the chosen one", random_user.mention()),
    _ => format!("{} {}", random_user.mention(), params.args[1]),
  };
  params
    .message
    .channel_id
    .send_message(http, |m| m.content(format!("{}", content)))
    .await
    .unwrap();
  Ok(Some(String::from(":ok:")))
}
