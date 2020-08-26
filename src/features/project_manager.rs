use crate::core::{
  commands::{CallBackParams, CallbackReturn},
  parse,
};
use chrono::offset::Utc;
use chrono::DateTime;
use serenity::{
  model::channel::{ChannelType, PermissionOverwrite, PermissionOverwriteType},
  model::Permissions,
};
use std::collections::HashMap;
use std::time::SystemTime;

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
        let right = &arg[index..];
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

const PROJECT_CATEGORY: u64 = 748316927780323409;
pub fn create_project(params: CallBackParams) -> CallbackReturn {
  let project_args = match project_creation_args(&params.args[1..params.args.len() - 1]) {
    Ok(result) => result,
    Err(error) => return Ok(Some(error)),
  };
  let blackfoot = parse::get_blackfoot(&params.context);

  let newchan = blackfoot
    .write()
    .create_channel(&params.context.http, |channel| {
      channel
        .kind(ChannelType::Text)
        .category(PROJECT_CATEGORY)
        .name(project_args["name"])
    })?;

  let allow = Permissions::SEND_MESSAGES;
  let deny = Permissions::empty();
  let overwrite = PermissionOverwrite {
    deny,
    allow,
    kind: PermissionOverwriteType::Member(params.message.author.id),
  };
  let system_time = SystemTime::now();
  let datetime: DateTime<Utc> = system_time.into();

  newchan.create_permission(&params.context.http, &overwrite)?;
  Ok(Some(format!(
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
    project_args.get("client").unwrap_or(&""),
    project_args.get("codex").unwrap_or(&"#PXXX"),
    project_args
      .get("lead")
      .unwrap_or(&&*params.message.author.name),
    project_args.get("deadline").unwrap_or(&"N/A"),
    project_args.get("description").unwrap_or(&"TBD"),
    project_args.get("contexte").unwrap_or(&"TBD"),
  )))
}
