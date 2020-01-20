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

pub use super::schema::users;
