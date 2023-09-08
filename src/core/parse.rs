use crate::constants::discordids;
use log::error;
use regex::Regex;
use serenity::{
  model::{
    channel::Channel,
    id::{ChannelId, GuildId},
  },
  prelude::*,
};
use strum_macros::Display;

#[derive(PartialEq, Eq, Debug, Display)]
pub enum DiscordIds {
  Message,
  Channel,
  Role,
  User,
}

pub fn main_guild_id() -> GuildId {
  GuildId(discordids::GUILD_ID)
}

pub async fn get_guild(
  channel_id: ChannelId,
  context: &Context,
  gid: Option<&String>,
) -> Result<GuildId, String> {
  let channel = channel_id.to_channel(&context.http).await.unwrap();
  match channel {
    Channel::Private(_) => match gid {
      Some(gid) => match gid.parse::<u64>() {
        Ok(id) => Ok(GuildId(id)),
        Err(parse_error) => {
          error!("{}", parse_error);
          Err(String::from("Invalid guild id"))
        }
      },
      None => Ok(main_guild_id()),
    },
    Channel::Guild(guildchan) => Ok(guildchan.guild_id),
    _ => Err(String::from("This doesn't work in this channel")),
  }
}

lazy_static! {
  static ref DISCORD_IDS_REGEX: Regex =
    Regex::new(r#"<(?<type>@!?|#|@&)(?<id>[0-9]{18,24})>"#).unwrap();
}

pub fn discord_str_to_id(
  id: &str,
  exepected_type: Option<DiscordIds>,
) -> Result<(u64, DiscordIds), String> {
  let Some(captures) = DISCORD_IDS_REGEX.captures(id) else {
    return Err("Did not match an discord id".to_string());
  };

  let kind = captures.name("type").unwrap();
  let discord_type: DiscordIds = match kind.as_str() {
    "@" | "@!" => DiscordIds::User,
    "#" => DiscordIds::Channel,
    "@&" => DiscordIds::Role,
    _ => return Err(format!("Incorect type for discordid: {}", id)),
  };
  if let Some(expected) = exepected_type {
    if expected != discord_type {
      let msg = format!(
        "Mismatched type, expected: {}, got: {}",
        expected, discord_type
      );
      return Err(msg);
    }
  }
  let id = captures.name("id").unwrap();
  // This is already validated by the regex, error should not be possible.
  let id = id.as_str().parse().unwrap();

  Ok((id, discord_type))
}

#[test]
fn test_discord_str_to_id() {
  assert_eq!(
    discord_str_to_id("<@256544457594241024>", None).unwrap().1,
    DiscordIds::User
  );
  assert_eq!(
    discord_str_to_id("<@173013989180178432>", None).unwrap().1,
    DiscordIds::User
  );
  assert_eq!(
    discord_str_to_id("<@173013989180178432>", None).unwrap().1,
    DiscordIds::User
  );
  assert_eq!(
    discord_str_to_id("<@1096007878160154697>", None).unwrap().1,
    DiscordIds::User
  );
  assert_eq!(
    discord_str_to_id("<@!1096007878160154697>", None)
      .unwrap()
      .1,
    DiscordIds::User
  );
  assert_eq!(
    discord_str_to_id("<#1096007878160154697>", None).unwrap().1,
    DiscordIds::Channel
  );
  assert_eq!(
    discord_str_to_id("<#852815758911340565>", None).unwrap().1,
    DiscordIds::Channel
  );
  assert_eq!(
    discord_str_to_id("<@&1096007878160154697>", None)
      .unwrap()
      .1,
    DiscordIds::Role
  );
  assert_eq!(
    discord_str_to_id("<@&1148945189935788056>", None)
      .unwrap()
      .1,
    DiscordIds::Role
  );
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

lazy_static! {
  static ref MESSAGE_SPLIT: Regex = Regex::new(r#"([^"\s]*"[^"\n]*"[^"\s]*)|([^\s]+)"#).unwrap();
}
pub fn split_message_args(input: &str) -> Vec<String> {
  let list_of_quotations = ['“', '”', '‘', '’', '«', '»', '„', '“'];

  let input_clean: String = input
    .chars()
    .map(|c| {
      if list_of_quotations.contains(&c) {
        '"'
      } else {
        c
      }
    })
    .collect();
  MESSAGE_SPLIT
    .find_iter(&input_clean)
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
          escaped = !escaped && c == &'\\';
          keep
        })
        .collect()
    })
    .collect()
}

// <:pepe_cucumber:887736509292228668>
pub fn emoji_str_convert(emoji_str: &str) -> Option<(bool, &str, &str)> {
  lazy_static! {
    static ref REGEX_EMOJI: Regex = Regex::new(r#"<(a)?:(.*):([0-9]{18})>"#).unwrap();
  }
  REGEX_EMOJI.captures(emoji_str).map(|captures| {
    (
      captures.get(1).is_some(),
      captures.get(2).unwrap().as_str(),
      captures.get(3).unwrap().as_str(),
    )
  })
}
