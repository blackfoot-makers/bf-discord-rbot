use chrono::prelude::*;
use log::{debug, error, info};
use serenity::{http, model::id::ChannelId};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::Arc;
use std::{thread, time};

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

fn query(client: &reqwest::blocking::Client, api_token: &String) -> Result<Querry, Box<dyn Error>> {
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

    let mut ticket_trigered: Vec<String> = Vec::new();

    let client = reqwest::blocking::Client::new();
    loop {
        threads_check();
        match query(&client, &api_token) {
            Ok(result_parsed) => {
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
            }
            Err(err) => {
                error!("Error querying airtable: {}", err);
            }
        }
        info!("ticket_trigered : {:?}", ticket_trigered);
        thread::sleep(time::Duration::from_secs(120));
    }
}
