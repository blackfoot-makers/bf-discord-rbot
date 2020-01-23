pub use super::models::*;
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

  pub fn user_search_mut(&mut self, discordid: &u64) -> Option<&mut User> {
    for user in self.users.iter_mut() {
      if user.discordid == *discordid as i64 {
        return Some(user);
      }
    }
    None
  }

  pub fn user_role_update(&mut self, discord_id: &u64, new_role: &Role) -> String {
    use super::schema::users::dsl::*;

    let conn = self.get_connection();

    let mut user: &mut User = match self.user_search_mut(&discord_id) {
      Some(user) => user,
      None => return String::from("User not found"),
    };

    diesel::update(users.find(user.id))
      .set(role.eq(new_role.to_string()))
      .get_result::<User>(&conn)
      .expect("Diesel: Unable to save new role");
    user.role = new_role.to_string();

    return String::from("Done");
  }

  pub fn user_search(&self, discordid: &u64) -> Option<&User> {
    for user in self.users.iter() {
      if user.discordid == *discordid as i64 {
        return Some(user);
      }
    }
    None
  }

  pub fn message_load(&mut self) {
    use super::schema::messages::dsl::*;

    let results = messages
      .limit(5)
      .load::<Message>(&self.get_connection())
      .expect("Error loading messages");

    self.messages = results;
  }

  pub fn message_add<'a>(&mut self, id: i64, author: i64, content: &String, channel: i64) {
    let new_message = Message {
      id,
      author,
      content: content.clone(),
      channel,
    };

    let new_message: Message = diesel::insert_into(messages::table)
      .values(&new_message)
      .get_result(&self.get_connection())
      .expect("Error saving new user");
    self.messages.push(new_message);
  }
}
