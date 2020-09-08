mod connection;
mod models;
mod queries;
mod schema;

use self::connection::{establish_connection, PgPool, PgPooledConnection};
use std::sync::RwLock;

pub use self::models::{Message, User};
pub use queries::*;

lazy_static! {
  pub static ref INSTANCE: RwLock<Instance> = RwLock::new(Instance::new());
}

impl Instance {
  pub fn new() -> Self {
    let mut instance = Instance {
      connection: establish_connection(),
      users: Vec::new(),
      messages: Vec::new(),
      airtable: Vec::new(),
      projects: Vec::new(),
      invites: Vec::new(),
    };
    instance.user_load();
    instance.message_load();
    instance.projects_load();
    instance.invites_load();
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
  pub airtable: Vec<AirtableRow>,
  pub projects: Vec<Project>,
  pub invites: Vec<Invite>,
}
