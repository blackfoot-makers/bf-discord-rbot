use crate::core::process::HTTP_STATIC;
use futures::executor::block_on;
use reqwest;
use reqwest::Error;
use serenity::model::id::ChannelId;
use std::collections::HashMap;

use job_scheduler::{JobScheduler, Job};
use std::time::Duration;

const CDC_CRA : ChannelId = ChannelId(651436625909252129);

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

lazy_static! {
    static ref HASHLIST: HashMap<&'static str, &'static str> =  hashmap![
        "daily calendar" => "http://108.128.12.212:5000/unfeedCalendar/daily",
        "monthly calendar" => "http://108.128.12.212:5000/unfeedCalendar/monthly",
        "daily codex" => "http://108.128.12.212:5000/invalidCodex/daily",
        "monthly codex" => "http://108.128.12.212:5000/invalidCodex/monthly"
    ];

    static ref RESPONSELIST: HashMap<&'static str, &'static str> = hashmap![
        "daily calendar" => "n'ayant pas remplis leur agenda de la journée sont: ",
        "monthly calendar" => "dont le nombre de jours ouvrés par mois est inférieur à la normale sont: ",
        "daily codex" => "ayant mal remplis leur codex de projet aujourd'hui sont: ",
        "monthly codex" => "ayant mal remplis leur codex projet sur les différents mois sont: "
    ];
}

async fn get_unfeed_calendar(name: &str) -> Result<Vec<String>, Error> {
    let url : &str = HASHLIST.get(&name).unwrap();
    let response = reqwest::blocking::get(url)?;
    let users: Vec<String> = response.json()?;
    
    Ok(users)
}

fn format_bot_response(name: &str, values: &Vec<String>) -> String {
    let response_text : &str =  RESPONSELIST.get(&name).unwrap();
    let message = "Les personnes ".to_string() ;
    
    format!("{}{}{}", message.to_string(), response_text.to_string(), values.join(" , "))
}

fn on_cron(name: &str) -> () {
    let http = HTTP_STATIC.read().clone().unwrap();
    let unfeeds = block_on(get_unfeed_calendar(name)).unwrap();

    CDC_CRA.send_message(http, |m| m.content(format_bot_response(name, &unfeeds))).unwrap();
    println!("{:?}", unfeeds.join(" "));
}

pub fn google_calendar(args: &Vec<&str>) -> String {
    let name = format!("{} {}", args[1], args[2]);
    
    if &name != "daily calendar" && &name != "monthly calendar" && &name != "daily codex" && &name != "monthly codex" {
        println!("Invalid argument: {:?}", &name);
    } else {
        on_cron(&name);
    }
    "Ok".to_string()
}

pub fn unfeed_calendar() -> () {
    let mut sched = JobScheduler::new();

    sched.add(Job::new("* 0 18 * * Mon,Tue,Wed,Thu,Fri *".parse().unwrap(), || {
        on_cron("daily calendar");
    }));

    loop {
        sched.tick();
        std::thread::sleep(Duration::from_secs(30))
    }
}
