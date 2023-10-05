use procedural_macros::command;
use serenity::model::prelude::GuildId;

use crate::{
  constants::discordids::GUILD_ID,
  core::{
    commands::{CallBackParams, CallbackReturn},
    parse::emoji_str_convert,
  },
};

#[command]
pub async fn add(params: CallBackParams<'_>) -> CallbackReturn {
  if let Some((is_animated, emoji_name, emoji_id)) = emoji_str_convert(&params.args[1]) {
    let extension = if is_animated { "gif" } else { "png" };
    let url = format!(
      "https://cdn.discordapp.com/emojis/{}.{}?size=128",
      emoji_id, extension
    );

    let client = reqwest::Client::builder().build()?;
    let response = client.get(url).send().await.unwrap();
    let response_body = response.bytes().await.unwrap();
    let base64_img = format!(
      "data:image/{};base64,{}",
      extension,
      base64::encode(response_body)
    );

    let guild = params.message.guild_id.unwrap_or(GuildId(GUILD_ID));
    guild
      .create_emoji(&params.context.http, emoji_name, &base64_img)
      .await?;
    Ok(Some(String::from(":ok:")))
  } else {
    Ok(Some(String::from("I am not able to get this emoji")))
  }
}

#[command]
pub async fn emoji_steal(params: CallBackParams<'_>) -> CallbackReturn {
  match &params.message.message_reference {
    None => Ok(Some(String::from("This message is not a reply"))),
    Some(message) => {
      let message_content = params
        .context
        .http
        .get_message(message.channel_id.0, message.message_id.unwrap().0)
        .await
        .expect("Expected a recent message to be in the cache");

      if let Some((is_animated, emoji_name, emoji_id)) = emoji_str_convert(&message_content.content)
      {
        let extension = if is_animated { "gif" } else { "png" };
        let url = format!(
          "https://cdn.discordapp.com/emojis/{}.{}?size=128",
          emoji_id, extension
        );

        let client = reqwest::Client::builder().build()?;
        let response = client.get(url).send().await.unwrap();
        let response_body = response.bytes().await.unwrap();
        let base64_img = format!(
          "data:image/{};base64,{}",
          extension,
          base64::encode(response_body)
        );

        let guild = params.message.guild_id.unwrap_or(GuildId(GUILD_ID));
        guild
          .create_emoji(&params.context.http, emoji_name, &base64_img)
          .await?;
        Ok(Some(String::from(":ok:")))
      } else {
        Ok(Some(String::from("I am not able to get this emoji")))
      }
    }
  }
}
