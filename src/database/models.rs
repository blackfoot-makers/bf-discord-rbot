#![allow(clippy::extra_unused_lifetimes)]
use chrono::NaiveDateTime;
use strum_macros::{Display, EnumString};

#[derive(Queryable, Debug)]
pub struct User {
  pub id: i32,
  pub discordid: i64,
  pub role: String,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
  pub discordid: i64,
  pub role: &'a str,
}

#[derive(Insertable, Queryable, Debug, Serialize, Clone)]
#[diesel(table_name = messages)]
pub struct Message {
  pub id: i64,
  pub author: i64,
  pub content: String,
  pub channel: i64,
  pub date: Option<std::time::SystemTime>,
}

#[derive(Queryable, Debug, Serialize, Clone)]
pub struct MessageEdit {
  pub id: i32,
  pub author: i64,
  pub content: String,
  pub channel: i64,
  pub date: Option<std::time::SystemTime>,
  pub parrent_message_id: i64,
}

#[derive(Insertable, Debug, Serialize, Clone)]
#[diesel(table_name = messages_edits)]
pub struct NewMessageEdit {
  pub id: Option<i32>,
  pub author: i64,
  pub content: String,
  pub channel: i64,
  pub date: Option<std::time::SystemTime>,
  pub parrent_message_id: i64,
}

#[derive(Copy, Clone, Display, EnumString, PartialEq, Eq, PartialOrd)]
pub enum Role {
  Guest,
  User,
  Moderator,
  Admin,
}

#[allow(dead_code)]
#[derive(Queryable, Debug)]
pub struct Project {
  pub id: i32,
  pub message_id: i64,
  pub channel_id: i64,
  pub codex: String,
  pub client: String,
  pub lead: String,
  pub deadline: String,
  pub description: String,
  pub contexte: String,
  pub created_at: std::time::SystemTime,
  pub pinned_message_id: Option<i64>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = projects)]
pub struct NewProject<'a> {
  pub message_id: i64,
  pub channel_id: i64,
  pub codex: Option<&'a str>,
  pub client: Option<&'a str>,
  pub lead: Option<&'a str>,
  pub deadline: Option<&'a str>,
  pub description: Option<&'a str>,
  pub contexte: Option<&'a str>,
  pub pinned_message_id: Option<i64>,
}

#[derive(Queryable, Debug, Clone)]
pub struct Invite {
  pub id: i32,
  pub code: String,
  pub actionrole: Option<i64>,
  pub actionchannel: Option<i64>,
  pub used_count: i32,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = invites)]
pub struct NewInvite {
  pub code: String,
  pub actionrole: Option<i64>,
  pub actionchannel: Option<i64>,
  pub used_count: i32,
}

#[allow(dead_code)]
#[derive(Queryable, Debug, Clone)]
pub struct Storage {
  pub id: i32,
  pub datatype: i64,
  pub dataid: Option<i64>,
  pub data: String,
  pub date: Option<std::time::SystemTime>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = storage)]
pub struct NewStorage<'a> {
  pub datatype: i64,
  pub dataid: Option<i64>,
  pub data: &'a str,
  pub date: Option<std::time::SystemTime>,
}

#[derive(Queryable, Debug, Clone)]
pub struct Event {
  pub id: i32,
  pub author: i64,
  pub content: String,
  pub channel: i64,
  pub trigger_date: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = events)]
pub struct NewEvent<'a> {
  pub author: i64,
  pub content: &'a str,
  pub channel: i64,
  pub trigger_date: NaiveDateTime,
}

pub use super::schema::*;
