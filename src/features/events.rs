use crate::{
  core::commands::{CallBackParams, CallbackReturn},
  database::{NewEvent, INSTANCE},
};
use chrono::{prelude::*, Duration};
use procedural_macros::command;
use regex::Regex;
use serenity::{http, model::id::ChannelId};
use std::sync::Arc;
use std::{thread, time};

lazy_static! {
  static ref TIME_INPUT_REGEX: Regex =
    Regex::new(r#"^([0-9]{1,3})(m(inutes?)?|h(ours?)?|d(ay?s)?)$"#)
      .expect("unable to create regex");
}

#[command]
pub async fn remind_me(params: CallBackParams) -> CallbackReturn {
  let input_date = &params.args[1];

  if let Some(captures) = TIME_INPUT_REGEX.captures(input_date) {
    let number = captures
      .get(1)
      .unwrap()
      .as_str()
      .parse::<u16>()
      .expect("unable to parse number value from regex");

    let mut duration: Option<Duration> = None;
    for i in [3, 5, 7] {
      if captures.get(i).is_some() {
        match i {
          3 => {
            duration = Some(Duration::minutes(number.into()));
          }
          5 => {
            duration = Some(Duration::hours(number.into()));
          }
          7 => {
            duration = Some(Duration::days(number.into()));
          }
          _ => panic!("captures matches missing case"),
        }
        break;
      }
    }
    if duration.is_none() {
      return Ok(Some("missing time denominator".to_string()));
    }
    let content = &params.args[2];
    let now = Utc::now().naive_local();
    let mut db_instance = INSTANCE.write().unwrap();
    db_instance.event_add(NewEvent {
      author: params.message.author.id.0 as i64,
      channel: params.message.channel_id.0 as i64,
      content,
      trigger_date: now + duration.unwrap(),
    });
    Ok(Some(":ok:".to_string()))
  } else {
    Ok(Some("the time parameter is invalid".to_string()))
  }
}

const SLEEP_TIME_SECS: u64 = 120;
/// Every X minutes check if an event should be sent
pub async fn check_events_loop(http: Arc<http::Http>) {
  loop {
    let events = {
      let db_instance = INSTANCE.read().unwrap();
      db_instance.events.clone()
    };
    let now = Utc::now().naive_utc();
    for event in events {
      let time_since_trigger = now - event.trigger_date;
      if time_since_trigger > Duration::seconds(0)
        && time_since_trigger < Duration::seconds(SLEEP_TIME_SECS as i64 + 5)
      {
        ChannelId(event.channel as u64)
          .say(http.clone(), event.content)
          .await
          .expect("unable to send event");
      }
    }

    thread::sleep(time::Duration::from_secs(SLEEP_TIME_SECS))
  }
}
