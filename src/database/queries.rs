use super::models::*;
use super::Instance;
use diesel::prelude::*;

impl Instance {
  pub fn user_load(&mut self) {
    use super::schema::users::dsl::*;

    let results = users
      .limit(5)
      .load::<User>(&self.get_connection())
      .expect("Error loading users");

    self.users = results;
  }

  pub fn user_add<'a>(&mut self, discordid: i64, role: &'a str) {
    let new_user = NewUser {
      discordid: discordid,
      role: role,
    };

    let newuser: User = diesel::insert_into(users::table)
      .values(&new_user)
      .get_result(&self.get_connection())
      .expect("Error saving new user");
    self.users.push(newuser);
  }
}
