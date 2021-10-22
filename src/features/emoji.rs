use procedural_macros::command;

use crate::core::{
  commands::{CallBackParams, CallbackReturn},
  parse::emoji_str_convert,
};

#[command]
pub async fn add(params: CallBackParams<'_>) -> CallbackReturn {
  if let Some((emoji_name, emoji_id)) = emoji_str_convert(&params.args[0]) {
    // emoji_id
    let url = format!(
      "https://cdn.discordapp.com/emojis/{}.png?size=128",
      emoji_id
    );
    let response = reqwest::get(url).await.unwrap();
    let response_body = response.text().await.unwrap();
    let base64_img = base64::encode(response_body);

    params
      .message
      .guild(&params.context.cache)
      .await
      .unwrap()
      .create_emoji(&params.context.http, &*emoji_name, &*base64_img)
      .await
      .unwrap();
    Ok(Some(String::from(":ok:")))
  } else {
    Ok(Some(String::from("I am not able to get this emoji")))
  }
}
