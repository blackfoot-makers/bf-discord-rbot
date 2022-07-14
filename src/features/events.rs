use crate::{
  core::commands::{CallBackParams, CallbackReturn},
  database::INSTANCE,
};
use chrono::{prelude::*, Duration};
use serenity::{http, model::id::ChannelId};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::{thread, time};

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
      let time_since_trigger = now - event.triger_date;
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
