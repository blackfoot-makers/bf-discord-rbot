use crate::core::commands::{CallBackParams, CallbackReturn};
use serde_json::{from_str, Value};
use serenity::prelude::*;

lazy_static! {
  pub static ref ATTACKED: RwLock<String> = RwLock::new(String::new());
  pub static ref MOM: RwLock<String> = RwLock::new(String::new());
}

pub fn attack_lauch(params: CallBackParams) -> CallbackReturn {
  ATTACKED.write().clear();

  let tag = format!("<@{}", &params.args[1][3..]);
  ATTACKED.write().push_str(&*tag);
  Ok(Some(format!("Prepare yourself {} !", params.args[1])))
}

pub fn mom_change(params: CallBackParams) -> CallbackReturn {
  MOM.write().clear();
  MOM.write().push_str(params.args[1]);
  Ok(Some(format!(
    "It's your momas turn yourself {} !",
    params.args[1]
  )))
}

pub fn witch_mom(_: CallBackParams) -> CallbackReturn {
  Ok(Some(format!("It's currently {} mom's", MOM.read())))
}

pub fn get_cat_pic(_: CallBackParams) -> CallbackReturn {
  let response =
    reqwest::blocking::get("https://api.thecatapi.com/v1/images/search?size=full").unwrap();
  let text = response.text().unwrap();

  let v: Value = from_str(&text).unwrap();

  let url = v[0]["url"].clone();
  let result = &mut url.to_string();
  result.pop();
  Ok(Some(String::from(&result[1..])))
}
