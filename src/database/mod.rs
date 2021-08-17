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
      messages_edits: Vec::new(),
      airtable: Vec::new(),
      projects: Vec::new(),
      invites: Vec::new(),
      storage: Vec::new(),
    };
    instance.user_load();
    instance.message_load();
    instance.message_edits_load();
    instance.projects_load();
    instance.invites_load();
    instance.storage_load();
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
  pub storage: Vec<Storage>,
  pub messages_edits: Vec<MessageEdit>,
}

#[derive(Debug, Clone)]
pub enum StorageDataType {
  Mom,
  ProjectBottomMessage,
}

impl From<StorageDataType> for i64 {
  fn from(id: StorageDataType) -> Self {
    id as i64
  }
}
