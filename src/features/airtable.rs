use chrono::prelude::*;
use log::{debug, info};
use reqwest;
use serenity::{http, model::id::ChannelId};
use std::collections::HashMap;
use std::sync::Arc;
use std::{thread, time};

const API_TOKEN: &str = "keycdRFRdaBnZPvH8";
const TICKET_SECONDS: i64 = 300;
// const TESTBOT_CHAN: ChannelId = ChannelId(555206410619584519);
const AIRBNB_CHAN: ChannelId = ChannelId(501406998085238784);

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

fn seconds_since_now(date_param: &str) -> Result<i64, chrono::ParseError> {
    let date = Local.datetime_from_str(date_param, "%Y-%m-%dT%H:%M:%S.000Z")?;
    let duration_chrono = Local::now() - date;
    Ok(duration_chrono.num_seconds())
}

pub fn check_airtable<F>(http: Arc<http::Http>, threads_check: F)
where
    F: for<'a> Fn(),
{
    let mut ticket_trigered: Vec<String> = Vec::new();

    loop {
        threads_check();

        let client = reqwest::blocking::Client::new();
        let mut request = client.request(
      reqwest::Method::GET,
      "https://api.airtable.com/v0/appA8HEheXt1LwX6t/Actions?fields%5B%5D=Requete&filterByFormula=%7BState%7D%20%3D%20%27%27",
    );
        request = request.bearer_auth(API_TOKEN);
        let text = request.send().unwrap().text();
        let result = text.unwrap();
        let result_parsed: Querry = serde_json::from_str(&result).unwrap();
        for record in result_parsed.records {
            if record.fields.contains_key("Requete") {
                let seconds = seconds_since_now(&record.created_time).unwrap();
                debug!(
                    "Secs: {} | [{}]{}",
                    seconds, record.id, record.fields["Requete"]
                );
                if seconds < TICKET_SECONDS && !ticket_trigered.contains(&record.id) {
                    ticket_trigered.push(record.id);
                    AIRBNB_CHAN
                        .say(&http, format!("New ticket: {}", record.fields["Requete"]))
                        .unwrap();
                }
            }
        }
        info!("ticket_trigered : {:?}", ticket_trigered);
        thread::sleep(time::Duration::from_secs(120));
    }
}
