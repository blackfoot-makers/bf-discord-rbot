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
    countdown_day: f64,
    message: String,
    channel: ChannelId,
    repeat: time::Duration,
}
const ONE_DAY: i64 = (3600 * 24);

fn get_chan_id(chan_param: &str) -> Result<ChannelId, serenity::model::misc::ChannelIdParseError> {
    let chan = &chan_param[2..chan_param.len() - 1];
    let chan_id = ChannelId::from_str(chan)?;
    Ok(chan_id)
}

fn numstr_to_duration(delay_param: &str) -> time::Duration {
    time::Duration::new(delay_param.parse::<u64>().unwrap() * 60, 0)
}

fn datestr_to_timeduration(date_param: &str) -> Result<time::Duration, chrono::ParseError> {
    let now_year = Local::now().year();
    let date = Local.datetime_from_str(
        &format!("{}-{}:00", now_year, date_param),
        "%Y-%m-%d:%H:%M:%S",
    )?;
    let duration_chrono = date - Local::now();
    Ok(time::Duration::new(duration_chrono.num_seconds() as u64, 0))
}

fn datestr_to_days(date_param: &str) -> Result<i64, chrono::ParseError> {
    let now_year = Local::now().year();
    let date = Local.datetime_from_str(
        &format!("{}-{}:00", now_year, date_param),
        "%Y-%m-%d:%H:%M:%S",
    )?;
    let duration_chrono = date - Local::now();
    Ok(duration_chrono.num_seconds() / ONE_DAY)
}

fn save_event(new_event: Event) {
    let mut file = NOTIFY_EVENT_FILE.write().unwrap();
    file.stored.push(new_event);
    file.write_stored().unwrap();
}

impl Event {
    pub fn add_reminder(params: &Vec<&str>) -> String {
        let duration_time = match datestr_to_timeduration(params[2]) {
            Ok(duration) => duration,
            Err(e) => {
                eprintln!("{}", e);
                return "Invalid time format".to_string();
            }
        };
        let chan_id = get_chan_id(params[4]);
        let repeat = if params.len() != 6 {
            numstr_to_duration(params[5])
        } else {
            time::Duration::new(0, 0)
        };
        let new_event = Event {
            name: String::from(params[1]),
            duration: duration_time,
            started: time::SystemTime::now(),
            message: String::from(params[3]),
            channel: chan_id.unwrap(),
            repeat: repeat,
            countdown_day: 0.0,
        };
        save_event(new_event);
        "Ok".to_string()
    }

    pub fn add_countdown(params: &Vec<&str>) -> String {
        let duration_time = match datestr_to_timeduration(params[2]) {
            Ok(duration) => duration,
            Err(e) => {
                eprintln!("{}", e);
                return "Invalid time format".to_string();
            }
        };
        let countdown_day = match datestr_to_days(params[3]) {
            Ok(duration) => duration,
            Err(e) => {
                eprintln!("{}", e);
                return "Invalid time format".to_string();
            }
        };
        let chan_id = get_chan_id(params[6]);
        let new_event = Event {
            name: String::from(params[1]),
            duration: duration_time,
            started: time::SystemTime::now(),
            countdown_day: countdown_day as f64,
            message: String::from(params[5]),
            channel: chan_id.unwrap(),
            repeat: numstr_to_duration(params[4]),
        };
        save_event(new_event);

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
                if event.started.elapsed().unwrap().as_secs() >= event.duration.as_secs() {
                    println!("Trigered {}", event.name);
                    if event.repeat.as_secs() > 0 {
                        print!(
                            "Debug ==> {} {}",
                            event.repeat.as_secs(),
                            event.countdown_day
                        );

                        event.started = time::SystemTime::now();
                        event.duration = event.repeat;

                        let _ = event
                            .channel
                            .say(format!(
                                "J-{} : {}",
                                event.countdown_day as u64, &event.message
                            ))
                            .unwrap();
                        if event.countdown_day > 0.0 {
                            event.countdown_day -= event.repeat.as_secs() as f64 / ONE_DAY as f64;
                        }
                        println!(" {}", event.countdown_day);
                    } else {
                        let _ = event.channel.say(&event.message).unwrap();
                    }
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
        thread::sleep(time::Duration::from_secs(30));
    }
}
