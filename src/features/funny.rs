use std::time::SystemTime;

use crate::core::commands::{CallBackParams, CallbackReturn};
use crate::database;
use chrono::{DateTime, Utc};
use database::{NewStorage, StorageDataType};
use procedural_macros::command;
use serde_json::{from_str, Value};
use serenity::prelude::*;

lazy_static! {
  pub static ref ATTACKED: RwLock<String> = RwLock::new(String::new());
}

#[command]
pub async fn attack_lauch(params: CallBackParams) -> CallbackReturn {
  ATTACKED.write().await.clear();

  let tag = format!("<@{}", &params.args[1][3..]);
  ATTACKED.write().await.push_str(&tag);
  Ok(Some(format!("Prepare yourself {} !", params.args[1])))
}

pub async fn mom_change_cmdless(user: &str, timestamp: DateTime<Utc>) -> Option<String> {
  let mut db_instance = database::INSTANCE.write().unwrap();
  let time: SystemTime = SystemTime::from(timestamp);
  let storage_found = db_instance.find_storage_type(StorageDataType::Mom).cloned();
  if let Some(storage) = storage_found {
    db_instance.storage_update(storage.id, user);
  } else {
    db_instance.storage_add(NewStorage {
      data: user,
      datatype: StorageDataType::Mom as i64,
      dataid: None,
      date: Some(time),
    });
  }
  Some(format!("It's your momas turn yourself {} !", user))
}

#[command]
pub async fn mom_change(params: CallBackParams) -> CallbackReturn {
  Ok(mom_change_cmdless(&params.args[1], *params.message.timestamp).await)
}

pub async fn which_mom_cmdless() -> Option<String> {
  let db_instance = database::INSTANCE.write().unwrap();
  let currentmom = db_instance.find_storage_type(StorageDataType::Mom);
  if let Some(mom) = currentmom {
    Some(format!("It's currently {} mom's", mom.data))
  } else {
    Some(String::from("Nobody is in trouble for now..."))
  }
}

#[command]
pub async fn which_mom(_: CallBackParams) -> CallbackReturn {
  Ok(which_mom_cmdless().await)
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
