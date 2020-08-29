use crate::constants::discordids::{PROJECT_ANOUNCEMENT_CHANNEL, PROJECT_CATEGORY};
use crate::core::{
  commands::{CallBackParams, CallbackReturn},
  parse,
};
use crate::database::{NewProject, ProjectIds, INSTANCE};
use chrono::offset::Utc;
use chrono::DateTime;
use log::error;
use serenity::{
  model::{
    channel::{ChannelType, PermissionOverwrite, PermissionOverwriteType, Reaction, ReactionType},
    id::{ChannelId, UserId},
    Permissions,
  },
  prelude::*,
};
use std::{collections::HashMap, time::SystemTime};

const ARGUMENT_LIST: [&str; 6] = [
  "codex",
  "client",
  "lead",
  "deadline",
  "description",
  "contexte",
];

pub fn project_creation_args<'a>(args: &'a [&str]) -> Result<HashMap<&'a str, &'a str>, String> {
  let mut project_args: HashMap<&str, &str> = HashMap::new();
  for arg in args {
    let find = arg.find('=');
    if let Some(index) = find {
      let left = &arg[..index];
      if ARGUMENT_LIST.contains(&left) {
        let right = &arg[index + 1..];
        project_args.insert(left, right);
      } else {
        return Err(format!("Invalid argument {}", arg));
      }
    } else {
      if project_args.contains_key("name") {
        return Err(format!("Unexpected argument {}", arg));
      }
      project_args.insert("name", arg);
    }
  }
  if project_args.contains_key("name") {
    return Ok(project_args);
  }
  Err(String::from("Missing name."))
}

fn user_add_permission(user: UserId) -> PermissionOverwrite {
  let allow = Permissions::READ_MESSAGES;
  let deny = Permissions::empty();
  PermissionOverwrite {
    deny,
    allow,
    kind: PermissionOverwriteType::Member(user),
  }
}

pub fn create_project(params: CallBackParams) -> CallbackReturn {
  let project_args = match project_creation_args(&params.args[1..]) {
    Ok(result) => result,
    Err(error) => return Ok(Some(error)),
  };
  let blackfoot = parse::get_main_guild(&params.context);
  let http = &params.context.http;
  let newchan = blackfoot.write().create_channel(http, |channel| {
    channel
      .kind(ChannelType::Text)
      .category(PROJECT_CATEGORY)
      .name(project_args["name"])
  })?;

  let system_time = SystemTime::now();
  let datetime: DateTime<Utc> = system_time.into();

  let overwrite = user_add_permission(params.message.author.id);
  newchan.create_permission(http, &overwrite)?;

  let client = project_args.get("client").unwrap_or(&"");
  let codex = project_args.get("codex").unwrap_or(&"#PXXX");
  let author_name = &*params.message.author.name;
  let lead = project_args.get("lead").unwrap_or(&author_name);
  let deadline = project_args.get("deadline").unwrap_or(&"N/A");
  let description = project_args.get("description").unwrap_or(&"N/A");
  let contexte = project_args.get("contexte").unwrap_or(&"N/A");
  let message = ChannelId(PROJECT_ANOUNCEMENT_CHANNEL).say(
    http,
    format!(
      "Création de <#{}>.

**Fiche de projet**
---
**Date de création** : {}
**Client** : {}
**Codex** : {}
**Lead projet** : {}
**Deadline (si applicable)** : {}
**Brief projet** : {}
**Contexte projet** : {}
    ",
      newchan.id,
      datetime.format("%d/%m/%Y"),
      client,
      codex,
      lead,
      deadline,
      description,
      contexte,
    ),
  )?;
  {
    let mut db_instance = INSTANCE.write().unwrap();
    db_instance.project_add(NewProject {
      message_id: message.id.0 as i64,
      channel_id: newchan.id.0 as i64,
      codex: Some(codex),
      client: Some(client),
      lead: Some(lead),
      deadline: Some(deadline),
      description: Some(description),
      contexte: Some(contexte),
    });
  }
  message.react(http, "✅")?;
  Ok(Some(String::from("Done")))
}

pub fn delete_project(params: CallBackParams) -> CallbackReturn {
  match parse::discord_str_to_id(params.args[1]) {
    Ok(target) => {
      let mut db_instance = INSTANCE.write().unwrap();
      let (result, project) = db_instance.projects_delete(target)?;
      if let Some(project) = project {
        let http = &params.context.http;
        ChannelId(project.channel_id as u64).delete(http)?;
        ChannelId(PROJECT_ANOUNCEMENT_CHANNEL)
          .message(http, project.message_id as u64)?
          .delete(http)?;
      };

      Ok(Some(String::from(result)))
    }
    Err(error) => Ok(Some(String::from(error))),
  }
}

// fn update_project(params: CallBackParams) -> CallbackReturn {
//   let project_args = match project_creation_args(&params.args[1..]) {
//     Ok(result) => result,
//     Err(error) => return Ok(Some(error)),
//   };

//   Ok(Some(String::from("Done")))
// }

pub fn check_subscribe(ctx: &Context, reaction: &Reaction, removed: bool) {
  let emoji_name = match &reaction.emoji {
    ReactionType::Unicode(e) => e.clone(),
    ReactionType::Custom {
      animated: _,
      name,
      id: _,
    } => name.clone().unwrap(),
    _ => "".to_string(),
  };
  if ["✅"].contains(&&*emoji_name) {
    let db_instance = INSTANCE.read().unwrap();
    if let Some((_index, project)) =
      db_instance.projects_search(reaction.message_id.0 as i64, ProjectIds::MessageId)
    {
      if let Some(channel) = ctx.cache.read().guild_channel(project.channel_id as u64) {
        if removed {
          channel
            .read()
            .delete_permission(&ctx.http, PermissionOverwriteType::Member(reaction.user_id))
            .unwrap();
        } else {
          let overwrite = user_add_permission(reaction.user_id);
          channel
            .read()
            .create_permission(&ctx.http, &overwrite)
            .unwrap();
        }
      } else {
        error!("Unable to find project channel in cache");
      }
    }
  }
}
