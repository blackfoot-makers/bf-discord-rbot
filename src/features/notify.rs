use chrono::prelude::*;
use serenity::model::id::ChannelId;
use std::str::FromStr;
use std::{thread, time};
use NOTIFY_EVENT_FILE;

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    name: String,
    duration: time::Duration,
    started: time::SystemTime,
    message: String,
    channel: ChannelId,
    repeat: bool,
}

fn get_chan_id(chan_param: &str ) -> Result<ChannelId, serenity::model::misc::ChannelIdParseError> {
    let chan = &chan_param[2..chan_param.len() - 1];
    let chan_id  = ChannelId::from_str(chan)?;
    Ok(chan_id)
}


impl Event {
    pub fn add(params: &Vec<&str>) -> String {
        let now_year = Local::now().year();
        let date = Local.datetime_from_str(&format!("{}-{}:00", now_year, params[2]), "%Y-%m-%d:%H:%M:%S");
        match date {
            Ok(_date) => (),
            Err(e) => println!("{} =|= {}", params[2], e),
        };
        let duration_chrono = date.unwrap() - Local::now();
        let duration_time = time::Duration::new(duration_chrono.num_seconds() as u64, 0);
        let chan_id = get_chan_id(params[4]);
        let new_event = Event {
            name: String::from(params[1]),
            duration: duration_time,
            started: time::SystemTime::now(),
            message: String::from(params[3]),
            channel: chan_id.unwrap(),
            repeat: params.len() == 6 && params[5] == "repeat",
        };
        let mut file = NOTIFY_EVENT_FILE.write().unwrap();
        file.stored.push(new_event);
        file.write_stored().unwrap();
        "Ok".to_string()
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Event) -> bool {
        self.name == other.name
    }
}

trait EventVec {
    fn remove_elem(&mut self, event: &Event);
}

impl EventVec for Vec<Event> {
    fn remove_elem(&mut self, other: &Event) {
        let mut index = 0;
        for event in self.iter() {
            if event == other {
                break;
            }
            index += 1;
        }
        self.remove(index);
    }
}

pub fn check_events() {
    println!("Events check thread started");
    loop {
        {
            //Free the lock durring sleep
            let events = &mut NOTIFY_EVENT_FILE.write().unwrap();
            for mut event in events.stored.iter_mut() {
                println!("Checking {}", event.name);
                println!(
                    "Started {} > {} Duration",
                    event.started.elapsed().unwrap().as_secs(),
                    event.duration.as_secs()
                );

                if event.started.elapsed().unwrap().as_secs() > event.duration.as_secs() {
                    println!("Trigered {}", event.name);
                    if event.repeat {
                        event.started = time::SystemTime::now();
                    }
                    let _ = event.channel.say(&event.message).unwrap();
                } else {
                    println!("Not Trigered {}", event.name);
                }
            }
            events.stored.retain(|event| {
                if event.started.elapsed().unwrap().as_secs() > event.duration.as_secs() {
                    return false;
                }
                true
            });
            events.write_stored().unwrap();
        }
        thread::sleep(time::Duration::from_secs(60));
    }
}
