use crate::constants::discordids::AITABLE_NOTIFY_CHAN;
use crate::database;
use chrono::DateTime;
use log::error;
use serenity::{http, model::id::ChannelId};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::Arc;
use std::{thread, time};

#[derive(Serialize, Deserialize, Debug)]
struct Record {
  id: String,
  fields: HashMap<String, String>,
  #[serde(rename(deserialize = "createdTime"))]
  created_time: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Querry {
  records: Vec<Record>,
  // offset: String,
}

fn database_record_add(record: &Record) -> bool {
  let mut db_instance = database::INSTANCE.write().unwrap();
  let time = match DateTime::parse_from_rfc3339(&*record.created_time) {
    // 2020-06-16T10:13:43.000Z
    Ok(time) => Some(time::SystemTime::from(time)),
    Err(error) => {
      error!(
        "Unable to parse time: {} for airtable row {}\n{}",
        record.id, record.created_time, error
      );
      None
    }
  };
  db_instance.airtable_row_add(&record.id, time, &record.fields["Requete"])
}

fn query(client: &reqwest::blocking::Client, api_token: &str) -> Result<Querry, Box<dyn Error>> {
  let mut request = client.request(reqwest::Method::GET,
        "https://api.airtable.com/v0/appA8HEheXt1LwX6t/Actions?fields%5B%5D=Requete&filterByFormula=%7BState%7D%20%3D%20%27%27");
  request = request.bearer_auth(api_token);
  let text = request.send()?.text();
  let result = text?;
  Ok(serde_json::from_str(&result)?)
}

pub fn check_airtable<F>(http: Arc<http::Http>, threads_check: F)
where
  F: for<'a> Fn(),
{
  let api_token: String = match env::var("AIRTABLE_TOKEN") {
    Ok(token) => token,
    Err(_) => {
      error!("Airtable token wasn't set, skiping feature");
      return;
    }
  };
  {
    let mut db_instance = database::INSTANCE.write().unwrap();
    db_instance.airtable_load();
  }

  let client = reqwest::blocking::Client::new();
  loop {
    threads_check();
    match query(&client, &api_token) {
      Ok(result_parsed) => {
        for record in result_parsed.records {
          if record.fields.contains_key("Requete") {
            let inserted = database_record_add(&record);
            if inserted {
              ChannelId(AITABLE_NOTIFY_CHAN)
                .say(&http, format!("New ticket: {}", record.fields["Requete"]))
                .expect(&*format!(
                  "Unable to send new message ticket: {}",
                  record.fields["Requete"]
                ));
            }
          }
        }
      }
      Err(err) => {
        error!("Error querying airtable: {}", err);
      }
    }
    thread::sleep(time::Duration::from_secs(120));
  }
}
