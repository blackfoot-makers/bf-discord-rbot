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
}

#[derive(Copy, Clone, Display, EnumString)]
pub enum Role {
  Guest,
  User,
  Moderator,
  Admin,
}

pub use super::schema::*;
