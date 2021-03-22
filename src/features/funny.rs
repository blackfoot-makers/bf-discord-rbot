use crate::core::commands::{CallBackParams, CallbackReturn};
use procedural_macros::command;
use serde_json::{from_str, Value};
use serenity::prelude::*;

lazy_static! {
  pub static ref ATTACKED: RwLock<String> = RwLock::new(String::new());
  pub static ref MOM: RwLock<String> = RwLock::new(String::new());
}

#[command]
pub async fn attack_lauch(params: CallBackParams) -> CallbackReturn {
  ATTACKED.write().await.clear();

  let tag = format!("<@{}", &params.args[1][3..]);
  ATTACKED.write().await.push_str(&*tag);
  Ok(Some(format!("Prepare yourself {} !", params.args[1])))
}

#[command]
pub async fn mom_change(params: CallBackParams) -> CallbackReturn {
  MOM.write().await.clear();
  MOM.write().await.push_str(params.args[1]);
  Ok(Some(format!(
    "It's your momas turn yourself {} !",
    params.args[1]
  )))
}

#[command]
pub async fn witch_mom(_: CallBackParams) -> CallbackReturn {
  Ok(Some(format!("It's currently {} mom's", MOM.read().await)))
}

#[command]
pub async fn get_cat_pic(_: CallBackParams) -> CallbackReturn {
  let response =
    reqwest::blocking::get("https://api.thecatapi.com/v1/images/search?size=full").unwrap();
  let text = response.text().unwrap();

  let v: Value = from_str(&text).unwrap();

  let url = v[0]["url"].clone();
  let result = &mut url.to_string();
  result.pop();
  Ok(Some(String::from(&result[1..])))
}
