use crate::constants::discordids::{PROJECT_ANOUNCEMENT_CHANNEL, PROJECT_CATEGORY};
use crate::core::{
  commands::{CallBackParams, CallbackReturn},
  parse,
  permissions::member_channel_read,
};
use crate::database::{NewProject, ProjectIds, INSTANCE};
use chrono::offset::Utc;
use chrono::DateTime;
use log::error;
use serenity::{
  model::{
    channel::{
      Channel, ChannelType, GuildChannel, PermissionOverwriteType, Reaction, ReactionType,
    },
    id::{ChannelId, UserId},
  },
  prelude::*,
};
use std::{collections::HashMap, sync::Arc, time::SystemTime};

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

pub fn create(params: CallBackParams) -> CallbackReturn {
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

  let overwrite = member_channel_read(params.message.author.id);
  newchan.create_permission(http, &overwrite)?;

  let client = project_args.get("client").unwrap_or(&"");
  let codex = project_args.get("codex").unwrap_or(&"#PXXX");
  let author_name = &*params.message.author.name;
  let lead = project_args.get("lead").unwrap_or(&author_name);
  let deadline = project_args.get("deadline").unwrap_or(&"N/A");
  let description = project_args.get("description").unwrap_or(&"N/A");
  let contexte = project_args.get("contexte").unwrap_or(&"N/A");
  let content = &format!(
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
  );
  let annoucement_message = ChannelId(PROJECT_ANOUNCEMENT_CHANNEL).say(http, content)?;
  let channel_message = newchan.say(http, content)?;
  channel_message.pin(http)?;
  {
    let mut db_instance = INSTANCE.write().unwrap();
    db_instance.project_add(NewProject {
      message_id: annoucement_message.id.0 as i64,
      channel_id: newchan.id.0 as i64,
      pinned_message_id: Some(channel_message.id.0 as i64),
      codex: Some(codex),
      client: Some(client),
      lead: Some(lead),
      deadline: Some(deadline),
      description: Some(description),
      contexte: Some(contexte),
    });
  }
  annoucement_message.react(http, "✅")?;
  if params.message.channel_id == ChannelId(PROJECT_ANOUNCEMENT_CHANNEL) {
    params.message.delete(http)?;
    return Ok(None);
  }
  Ok(Some(String::from("Done")))
}

pub fn delete(params: CallBackParams) -> CallbackReturn {
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

pub fn add_user(params: CallBackParams) -> CallbackReturn {
  let cache = &params.context.cache;
  let http = &params.context.http;
  let usertag = params.args[1];
  let add_perm = |guildchannel: &Arc<RwLock<GuildChannel>>, userid| {
    let overwrite = member_channel_read(UserId(userid));
    guildchannel
      .read()
      .create_permission(http, &overwrite)
      .unwrap();
    Ok(Some(format!("Added <@{}> Welcome !", userid)))
  };

  match params
    .message
    .channel(cache)
    .expect("Channel of message wasn't found")
  {
    Channel::Guild(guildchannel) => match parse::discord_str_to_id(usertag) {
      Ok(userid) => add_perm(&guildchannel, userid),
      Err(_error) => {
        if let Some(guild) = &guildchannel.read().guild(cache) {
          let guildptr = guild.read();
          let mut members = guildptr.members_nick_containing(usertag, false, false);
          if members.is_empty() {
            members = guildptr.members_username_containing(usertag, false, false);
          }
          if !members.is_empty() {
            if members.len() == 1 {
              let userid = members
                .first()
                .expect("Weird no first member...")
                .user_id()
                .0;
              add_perm(&guildchannel, userid)
            } else {
              let mut members_nick = String::new();
              for member in members {
                members_nick.push_str(&format!("{}, ", member.display_name()));
              }
              Ok(Some(format!(
                "Found too many member with this nickname: {}",
                &members_nick[..members_nick.len() - 2]
              )))
            }
          } else {
            Ok(Some(format!(
              "Didn't find any user with {} in their name",
              usertag
            )))
          }
        } else {
          Ok(Some(String::from(
            "Unable to find user using tag or nickname",
          )))
        }
      }
    },
    _ => Ok(Some(String::from(
      "This command is restricted to a guild channel",
    ))),
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
          let overwrite = member_channel_read(reaction.user_id);
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
