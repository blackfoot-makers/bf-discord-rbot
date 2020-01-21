mod connection;
mod models;
pub mod queries;
mod schema;

use self::connection::{establish_connection, PgPool, PgPooledConnection};
use self::models::{Message, User};
use std::sync::RwLock;

lazy_static! {
  pub static ref INSTANCE: RwLock<Instance> = RwLock::new(Instance::new());
}

impl Instance {
  pub fn new() -> Self {
    let mut instance = Instance {
      connection: establish_connection(),
      users: Vec::new(),
      messages: Vec::new(),
    };
    instance.user_load();
    instance.message_load();
    instance
  }

  pub fn get_connection(&self) -> PgPooledConnection {
    self.connection.get().unwrap()
  }
}

pub struct Instance {
  connection: PgPool,
  pub users: Vec<User>,
  pub messages: Vec<Message>,
}
