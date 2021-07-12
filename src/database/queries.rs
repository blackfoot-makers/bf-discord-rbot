pub use super::models::*;
use super::{Instance, StorageDataType};
use crate::core::parse::DiscordIds;
use diesel::prelude::*;
use std::error::Error;

impl Instance {
  db_load! {user_load, User, users}

  pub fn user_add(&mut self, discordid: i64, role: &'_ str) {
    let new_user = NewUser { discordid, role };

    let newuser: User = diesel::insert_into(users::table)
      .values(&new_user)
      .get_result(&self.get_connection())
      .expect("Error saving new user");
    self.users.push(newuser);
  }

  pub fn user_search_mut(&mut self, discordid: u64) -> Option<&mut User> {
    for user in self.users.iter_mut() {
      if user.discordid == discordid as i64 {
        return Some(user);
      }
    }
    None
  }

  pub fn user_role_update(&mut self, discord_id: u64, new_role: Role) -> String {
    use super::schema::users::dsl::*;

    let conn = self.get_connection();

    let mut user: &mut User = match self.user_search_mut(discord_id) {
      Some(user) => user,
      None => return String::from("User not found"),
    };

    diesel::update(users.find(user.id))
      .set(role.eq(new_role.to_string()))
      .get_result::<User>(&conn)
      .expect("Diesel: Unable to save new role");
    user.role = new_role.to_string();

    format!("Updated {} to {}", user.discordid, user.role)
  }

  pub fn user_search(&self, discordid: u64) -> Option<&User> {
    for user in self.users.iter() {
      if user.discordid == discordid as i64 {
        return Some(user);
      }
    }
    None
  }

  db_load! {message_load, Message, messages}
  db_load! {message_edits_load, MessageEdit, messages_edits}

  db_add! { message_add, Message, Message, messages }
  db_add! { message_edit_add, MessageEdit, MessageEdit, messages_edits }

  #[allow(dead_code)]
  pub fn mesage_delete(&mut self, messages_id: Vec<i64>) -> Vec<Message> {
    use super::schema::messages::dsl::*;
    if messages_id.is_empty() {
      return Vec::new();
    }

    let conn = self.get_connection();

    let filter = messages.filter(id.eq_any(&messages_id)).or_filter(id.eq(0));
    diesel::delete(filter)
      .execute(&conn)
      .expect("Diesel: Unable to delete messages");
    let previous_bottom_list: Vec<Message> = self
      .messages
      .drain_filter(|msg| messages_id.contains(&msg.id))
      .collect();
    previous_bottom_list
  }

  pub fn airtable_row_add(
    &mut self,
    aid: &str,
    created_time: Option<std::time::SystemTime>,
    content: &str,
  ) -> bool {
    match self.airtable_row_search(aid.to_string()) {
      Some(_) => false,
      None => {
        let new_row = NewAirtableRow {
          aid: aid.to_string(),
          content: content.to_string(),
          created_time,
        };

        let new_row: AirtableRow = diesel::insert_into(airtable::table)
          .values(&new_row)
          .get_result(&self.get_connection())
          .expect("Error saving new airtable_row");
        self.airtable.push(new_row);
        true
      }
    }
  }

  pub fn airtable_row_search(&self, aid: String) -> Option<&AirtableRow> {
    for row in self.airtable.iter() {
      if row.aid == aid {
        return Some(row);
      }
    }
    None
  }

  db_load! {airtable_load, AirtableRow, airtable}

  db_add! {project_add, NewProject, Project, projects}

  db_load! {projects_load, Project, projects}

  pub fn projects_search(&self, id: i64, typeid: DiscordIds) -> Option<(usize, &Project)> {
    for (index, project) in self.projects.iter().enumerate() {
      match typeid {
        DiscordIds::Message => {
          if project.message_id == id {
            return Some((index, project));
          }
        }
        DiscordIds::Channel => {
          if project.channel_id == id {
            return Some((index, project));
          }
        }
        _ => {}
      }
    }
    None
  }

  pub fn projects_delete(
    &mut self,
    p_channel_id: u64,
  ) -> Result<(&str, Option<Project>), Box<dyn Error + Send + Sync>> {
    use super::schema::projects::dsl::*;

    if let Some((index, project)) = self.projects_search(p_channel_id as i64, DiscordIds::Channel) {
      diesel::delete(projects.filter(id.eq(project.id))).execute(&self.get_connection())?;
      let project = self.projects.remove(index);
      return Ok((":ok:", Some(project)));
    }
    Ok(("Channel wasn't found", None))
  }

  db_load! {invites_load, Invite, invites}

  pub fn invite_search(&mut self, code: &str) -> Option<&mut Invite> {
    for invite in self.invites.iter_mut() {
      if invite.code == code {
        return Some(invite);
      }
    }
    None
  }

  pub fn invite_update(
    &mut self,
    p_code: String,
    p_count: Option<i32>,
    p_actionchannel: Option<i64>,
    p_actionrole: Option<i64>,
  ) -> Result<(i32, Invite), Box<dyn Error + Send + Sync>> {
    let connection = &self.get_connection();
    if let Some(invite) = self.invite_search(&p_code) {
      use super::schema::invites::dsl::*;
      let updated: Invite = diesel::update(invites.filter(id.eq(invite.id)))
        .set((
          used_count.eq(p_count.unwrap_or(invite.used_count)),
          actionchannel.eq(p_actionchannel.or(invite.actionchannel)),
          actionrole.eq(p_actionrole.or(invite.actionrole)),
        ))
        .get_result(connection)?;
      let used_diff = p_count.unwrap_or(invite.used_count) - invite.used_count;
      *invite = updated.clone();
      Ok((used_diff, updated))
    } else {
      let new_invite = NewInvite {
        code: p_code,
        used_count: p_count.unwrap_or(0),
        actionchannel: p_actionchannel,
        actionrole: p_actionrole,
      };
      let new_invite: Invite = diesel::insert_into(invites::table)
        .values(&new_invite)
        .get_result(&self.get_connection())
        .expect("Error saving new airtable_row");
      self.invites.push(new_invite.clone());
      Ok((0, new_invite))
    }
  }

  db_load! { storage_load, Storage, storage}

  db_add! { storage_add, NewStorage, Storage, storage }

  pub fn find_storage_type(&self, storage_type: StorageDataType) -> Option<&Storage> {
    let compare_storage = storage_type as i64;
    self
      .storage
      .iter()
      .find(|storage| storage.datatype == compare_storage)
  }

  pub fn storage_update(&mut self, storage_id: i32, new_data: &str) {
    use super::schema::storage::dsl::*;

    let conn = self.get_connection();

    let newstorage = diesel::update(storage.find(storage_id))
      .set(data.eq(new_data))
      .get_result::<Storage>(&conn)
      .expect("Diesel: Unable to update storage element");
    self.storage.drain_filter(|s| s.id == storage_id);
    self.storage.push(newstorage);
  }

  pub fn storage_delete(&mut self, storage_id: Vec<i32>) -> Vec<Storage> {
    use super::schema::storage::dsl::*;
    if storage_id.is_empty() {
      return Vec::new();
    }

    let conn = self.get_connection();

    let filter = storage.filter(id.eq_any(&storage_id)).or_filter(id.eq(0));
    diesel::delete(filter)
      .execute(&conn)
      .expect("Diesel: Unable to delete storage");
    let previous_bottom_list: Vec<Storage> = self
      .storage
      .drain_filter(|msg| storage_id.contains(&msg.id))
      .collect();
    previous_bottom_list
  }
}
