use strum_macros::{Display, EnumString};

#[derive(Queryable, Debug)]
pub struct User {
  pub id: i32,
  pub discordid: i64,
  pub role: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
  pub discordid: i64,
  pub role: &'a str,
}

#[derive(Insertable, Queryable, Debug)]
#[table_name = "messages"]
pub struct Message {
  pub id: i64,
  pub author: i64,
  pub content: String,
  pub channel: i64,
  pub date: Option<std::time::SystemTime>,
}

#[derive(Copy, Clone, Display, EnumString, PartialEq, PartialOrd)]
pub enum Role {
  Guest,
  User,
  Moderator,
  Admin,
}

#[derive(Queryable, Debug)]
pub struct AirtableRow {
  pub id: i32,
  pub aid: String,
  pub content: String,
  pub created_time: Option<std::time::SystemTime>,
}

#[derive(Insertable, Debug)]
#[table_name = "airtable"]
pub struct NewAirtableRow {
  pub aid: String,
  pub content: String,
  pub created_time: Option<std::time::SystemTime>,
}

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
}

#[derive(Insertable, Debug)]
#[table_name = "projects"]
pub struct NewProject<'a> {
  pub message_id: i64,
  pub channel_id: i64,
  pub codex: Option<&'a str>,
  pub client: Option<&'a str>,
  pub lead: Option<&'a str>,
  pub deadline: Option<&'a str>,
  pub description: Option<&'a str>,
  pub contexte: Option<&'a str>,
}

pub use super::schema::*;
