use crate::core::commands::{CallBackParams, CallbackReturn};
use chrono::{NaiveDate, NaiveDateTime, Utc};
use log::error;
use procedural_macros::command;
use reqwest::Client;
use std::{env, fmt::Display};

lazy_static! {
  static ref CRA_SERVER: String = env::var("CRA_SERVER").expect("CRA_SERVER WAS NOT FOUND");
}

const HOUR_PER_DAY: i64 = 7;

/// This structure is used to represent a warning in the database and query it
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Warnings {
  pub name: String,
  pub warning_date: NaiveDateTime,
  pub event_name: Option<String>,
  pub warning_reason: i32,
}

/// This enum is used to represent the type of warning
#[derive(Debug)]
pub enum WarnReason {
  /// The user is being warned because he doesn't have [HOUR_PER_DAY] hours in each days
  NotEnoughtHour = 0,
  /// The user is being warned because he have and event between two month
  BetweenTwoMonth = 1,
  /// The user is being warned because he have an event that is on a holiday and normal day at the same time
  BetweenHoliday = 2,
}

impl Display for WarnReason {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      WarnReason::NotEnoughtHour => {
        write!(f, "There is not {} hours in this day", HOUR_PER_DAY)
      }
      WarnReason::BetweenTwoMonth => write!(f, "An event is between two month"),
      WarnReason::BetweenHoliday => write!(f, "An event is on a holiday and normal day"),
    }
  }
}

impl From<i32> for WarnReason {
  fn from(value: i32) -> Self {
    match value {
      0 => WarnReason::NotEnoughtHour,
      1 => WarnReason::BetweenTwoMonth,
      2 => WarnReason::BetweenHoliday,
      _ => {
        error!("Can not convert {} to a WarnReason", value);
        WarnReason::NotEnoughtHour
      }
    }
  }
}

#[command]
pub async fn check_calendar(params: CallBackParams) -> CallbackReturn {
  let client = Client::new();
  let date = if params.args.len() > 1 {
    match NaiveDate::parse_from_str(&format!("01/{}", params.args[1]), "%m/%Y/%d") {
      Ok(date) => date,
      Err(_) => return Ok(Some(String::from("Invalid date format"))),
    }
  } else {
    Utc::now().naive_local().date()
  };
  let warnings = match client
    .get(format!("{}/warnings/", *CRA_SERVER))
    .query(&[
      ("date", date.to_string()),
      ("discord_id", params.message.author.id.to_string()),
    ])
    .send()
    .await
  {
    Ok(response) => match response.json::<Vec<Warnings>>().await {
      Ok(warnings) => warnings,
      Err(error) => {
        error!("An error occured while deserializing warnings: {}", error);
        return Ok(Some(String::from(
          "An error occured while deserializing warnings",
        )));
      }
    },
    Err(e) => {
      error!("Error while getting warnings: {}", e);
      return Ok(Some(String::from(
        "Error while getting warnings from the cra api",
      )));
    }
  };
  if warnings.is_empty() {
    return Ok(Some(String::from(
      "No warnings found timoun will be happy !!",
    )));
  }
  let mut messages = vec![format!("You have {} warnings:", warnings.len())];
  warnings.iter().for_each(|warning| {
    messages.push(format!(
      "`{}` {} for {}",
      warning.warning_date.date().format("%d/%m/%Y"),
      warning.event_name.clone().unwrap_or_default(),
      WarnReason::from(warning.warning_reason),
    ));
  });
  Ok(Some(messages.join("\n")))
}
