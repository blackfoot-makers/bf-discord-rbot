use crate::core::{
  commands::{CallBackParams, CallbackReturn},
  parse,
};
use log::error;
use procedural_macros::command;
use serenity::model::id::UserId;

#[command]
pub async fn rename(params: CallBackParams) -> CallbackReturn {
  let http = &params.context.http;
  let channel_id = params.message.channel_id;
  let guild = match parse::get_guild(channel_id, params.context, params.args.get(3)).await {
    Ok(guild) => guild,
    Err(error) => return Ok(Some(error)),
  };
  let (targeted_user_id, _) =
    parse::discord_str_to_id(&params.args[1], Some(parse::DiscordIds::User))?;
  let member = guild.member(http, UserId(targeted_user_id)).await;
  match member {
    Ok(member) => {
      if params.args[2].len() > 32 {
        return Ok(Some("The new nickname is too long.".to_string()));
      }
      member
        .edit(http, |member| member.nickname(&params.args[2]))
        .await?;
      Ok(Some(String::from(":ok:")))
    }
    Err(error) => {
      error!("Rename: member not found: {}", error);
      Ok(Some(format!(
        "User {} not found in guild: {}",
        targeted_user_id,
        guild.name(&params.context.cache).unwrap(),
      )))
    }
  }
}
