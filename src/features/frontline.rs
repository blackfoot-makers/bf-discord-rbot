use crate::core::commands::{CallBackParams, CallbackReturn};
// use crate::database;
use ftp::FtpStream;
use log::error;
use procedural_macros::command;
use serenity::{http, model::id::ChannelId};
use std::collections::HashMap;
use std::env;
// use std::error::Error;
use crate::constants::discordids::ANNOYED_CHAN_HERDINGCHATTE;
use std::sync::{Arc, RwLock};
use std::{thread, time};

lazy_static! {
  pub static ref DIRECTORY_WATCH: RwLock<HashMap<String, usize>> = RwLock::new(HashMap::new());
}

fn ftp_connect() -> FtpStream {
  let host: String = env::var("FRONTLINE_FTP_HOST").expect("FRONTLINE_FTP_HOST isn't set");
  let user = env::var("FRONTLINE_FTP_USER").expect("FRONTLINE_FTP_USER isn't set");
  let password = env::var("FRONTLINE_FTP_PASSWORD").expect("FRONTLINE_FTP_PASSWORD isn't set");
  let mut ftp_stream = FtpStream::connect(host).unwrap();
  let _ = ftp_stream.login(&*user, &*password).unwrap();
  ftp_stream
}

#[command]
pub async fn add_directory(params: CallBackParams) -> CallbackReturn {
  let dir_target = String::from(params.args[1]);
  let mut ftp_stream = ftp_connect();
  let root = ftp_stream.nlst(None).expect("Unable to list ftp dir");
  if !root.contains(&dir_target) {
    return Ok(Some(String::from(
      "I didn't find this directory in the ftp",
    )));
  }

  DIRECTORY_WATCH.write().unwrap().insert(dir_target, 0);
  Ok(Some(String::from(":ok:")))
}

pub async fn check<F>(http: Arc<http::Http>, threads_check: F)
where
  F: for<'a> Fn(),
{
  match env::var("FRONTLINE_FTP_HOST") {
    Ok(token) => token,
    Err(_) => {
      error!("frontile host wasn't set, skiping feature");
      return;
    }
  };
  // {
  //   let mut db_instance = database::INSTANCE.write().unwrap();
  //   db_instance.airtable_load();
  // }

  loop {
    threads_check();

    let mut noupdate_dir = String::new();
    {
      let mut ftp_stream = ftp_connect();
      let mut directorys = DIRECTORY_WATCH.write().unwrap();

      for (directory, count) in directorys.iter_mut() {
        let path = format!("{}/Activity", directory);
        let list = ftp_stream
          .nlst(Some(&*path))
          .expect("Unable to list ftp dir");
        if *count < list.len() {
          *count = list.len();
        } else {
          noupdate_dir = String::from(directory);
        }
      }
      ftp_stream.quit().unwrap();
    }
    if !noupdate_dir.is_empty() {
      ChannelId(ANNOYED_CHAN_HERDINGCHATTE)
        .say(
          &http,
          format!("<@344498090801364993> 😱 No update on {}", noupdate_dir),
        )
        .await
        .unwrap();
    }
    thread::sleep(time::Duration::from_secs(60 * 20));
  }
}
