use procedural_macros::command;

use crate::core::{
  commands::{CallBackParams, CallbackReturn},
  parse::emoji_str_convert,
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

    let guild = params
      .message
      .guild_id
      .expect("Guildid wasn't found in the message");
    guild
      .create_emoji(&params.context.http, emoji_name, &base64_img)
      .await?;
    Ok(Some(String::from(":ok:")))
  } else {
    Ok(Some(String::from("I am not able to get this emoji")))
  }
}
