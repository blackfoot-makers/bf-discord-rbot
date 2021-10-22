use procedural_macros::command;
use reqwest::Request;

use crate::core::{
  commands::{CallBackParams, CallbackReturn},
  parse::emoji_str_convert,
};

#[command]
pub async fn add(params: CallBackParams<'_>) -> CallbackReturn {
  if let Some((emoji_name, emoji_id)) = emoji_str_convert(&params.args[1]) {
    let url = format!(
      "https://cdn.discordapp.com/emojis/{}.png?size=128",
      emoji_id
    );

    let client = reqwest::Client::builder().build()?;
    let response = client
      .get(url)
      .header("user-agent", "curl/7.68.0")
      .send()
      .await
      .unwrap();
    let headers = format!("{:#?}", response.headers());
    let response_body = response.bytes().await.unwrap();
    let base64_img = format!("data:image/png;base64,{}", base64::encode(response_body));
    println!(
      "emoji_name: {}, emoji_id: {}, response: {:#?}\n\nbase64_img: {}",
      emoji_name, emoji_id, headers, base64_img
    );

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
