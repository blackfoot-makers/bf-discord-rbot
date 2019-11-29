#[derive(Queryable)]
pub struct User {
  pub id: i32,
  pub discordid: i32,
  pub role: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
  pub discordid: i32,
  pub role: &'a str,
}

pub use super::schema::users;
