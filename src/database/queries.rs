use super::establish_connection;
use super::models::*;
use diesel::prelude::*;

pub struct Users {
  connection: PgConnection,
}

impl Users {
  pub fn new() -> Self {
    Self {
      connection: establish_connection(),
    }
  }

  pub fn get(&self) {
    use super::schema::users::dsl::*;

    let results = users
      .limit(5)
      .load::<User>(&self.connection)
      .expect("Error loading users");

    println!("Displaying {} users", results.len());
    for user in results {
      println!("----------");
      println!("{}", user.id);
      println!("{}", user.discordid);
    }
  }

  pub fn create<'a>(discordid: i32, role: &'a str) -> User {
    let connection = establish_connection();

    let new_user = NewUser {
      discordid: discordid,
      role: role,
    };

    diesel::insert_into(users::table)
      .values(&new_user)
      .get_result(&connection)
      .expect("Error saving new user")
  }
}
