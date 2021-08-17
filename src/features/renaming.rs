use crate::core::{
  commands::{CallBackParams, CallbackReturn},
  parse,
};
use log::error;
use procedural_macros::command;
use serenity::model::id::UserId;

#[command]
pub async fn rename(params: CallBackParams) -> CallbackReturn {
  let cache = &params.context.cache;
  let http = &params.context.http;
  let channel = params.message.channel(&cache).await.unwrap();
  let guild = match parse::get_guild(channel, params.context, params.args.get(3)).await {
    Ok(guild) => guild,
    Err(error) => return Ok(Some(error)),
  };
  let (targeted_user_id, _) =
    parse::discord_str_to_id(&params.args[1], Some(parse::DiscordIds::User))?;
  let member = guild.member(http, UserId(targeted_user_id)).await;
  match member {
    Ok(member) => {
      member
        .edit(http, |member| member.nickname(&params.args[2]))
        .await?;
      Ok(Some(String::from("Rename done :)")))
    }
    Err(error) => {
      error!("Rename: member not found: {}", error);
      Ok(Some(format!(
        "User {} not found in guild: {}",
        targeted_user_id, guild.name,
      )))
    }
  }
}
