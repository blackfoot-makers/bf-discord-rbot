use crate::constants::discordids;
use log::error;
use regex::Regex;
use serenity::{
  model::{channel::Channel, guild::Guild},
  prelude::*,
};
use strum_macros::Display;

#[derive(PartialEq, Debug, Display)]
pub enum DiscordIds {
  Message,
  Channel,
  Role,
  User,
}

pub async fn get_main_guild(context: &Context) -> Guild {
  context
    .cache
    .guild(discordids::GUILD_ID)
    .await
    .expect("Unable to find main guild")
}

pub async fn get_guild(
  channel: Channel,
  context: &Context,
  gid: Option<&String>,
) -> Result<Guild, String> {
  match channel {
    Channel::Private(_) => match gid {
      Some(gid) => {
        let id = match gid.parse::<u64>() {
          Ok(id) => id,
          Err(parse_error) => {
            error!("{}", parse_error);
            return Err(String::from("Invalid guild id"));
          }
        };
        match context.cache.guild(id).await {
          Some(guild) => Ok(guild),
          None => Err(format!("Guild: {} not found", gid)),
        }
      }
      None => Ok(get_main_guild(context).await),
    },
    Channel::Guild(guildchan) => Ok(guildchan.guild(&context.cache).await.unwrap()),
    _ => Err(String::from("This doesn't work in this channel")),
  }
}

pub fn discord_str_to_id(
  id: &str,
  exepected_type: Option<DiscordIds>,
) -> Result<(u64, DiscordIds), String> {
  let size = id.len();
  const SIZEBIGINT: usize = 18;
  if size < SIZEBIGINT {
    return Err(String::from("Unable to parse, text isn't an disocrd ID"));
  }

  if size == SIZEBIGINT {
    let parsedid = id.parse::<u64>().expect("Unable to parse Id, not numeric");
    Ok((parsedid, DiscordIds::Channel))
  } else {
    let parsedid = id[size - (SIZEBIGINT + 1)..size - 1]
      .parse::<u64>()
      .expect("Unable to parse Id, badly formated");
    let identifier = &id[0..size - (SIZEBIGINT + 1)];
    let discordtype: DiscordIds = match identifier {
      "<@" | "<@!" => DiscordIds::User,
      "<#" => DiscordIds::Channel,
      "<@&" => DiscordIds::Role,
      _ => DiscordIds::Channel,
      // Channel can't be pinged so no identifier sadly
      // _ => return Err(&*format!("Incored type for discordid: {}", identifier)),
    };
    if let Some(expected) = exepected_type {
      if expected != discordtype {
        let msg = format!(
          "Mismatched type, expected: {}, got: {}",
          expected, discordtype
        );
        return Err(msg);
      }
    }
    Ok((parsedid, discordtype))
  }
}

#[test]
fn test_split_message_args() {
  assert_eq!(
    vec![r#"test=testas"#],
    split_message_args(r#"test="testas""#)
  );
  assert_eq!(
    vec![r#"test=\"test\""#],
    split_message_args(r#"test=\"test\""#)
  );
  assert_eq!(vec!["test=test"], split_message_args("test=test"));
  assert_eq!(
    vec!["test=test jambon"],
    split_message_args("test=\"test jambon\"")
  );

  assert_eq!(
    vec![
      r#"test=test jambon"#,
      r#"dd"#,
      r#"testos=1"#,
      r#"ddd"#,
      r#"d"#,
      r#"dd"#,
      r#" d d d "#
    ],
    split_message_args(r#"test="test jambon" dd "testos=1" ddd d dd " d d d " "#)
  );
}

pub fn split_message_args(input: &str) -> Vec<String> {
  let regex_split = Regex::new(r#"([^"\s]*"[^"\n]*"[^"\s]*)|([^\s]+)"#).unwrap();
  regex_split
    .find_iter(input)
    .map(|m| {
      let matche_str = m.as_str();
      let mut escaped = false;
      matche_str
        .chars()
        .filter(|c| {
          let mut keep = true;
          if c == &'"' && !escaped {
            keep = false;
          }
          // This XOR is to account for escaped "\"
          escaped = !escaped && c == &'\\';
          keep
        })
        .collect()
    })
    .collect()
}
