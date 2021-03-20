use crate::constants::discordids::{PROJECT_ANOUNCEMENT_CHANNEL, PROJECT_CATEGORY};
use crate::core::{
  commands::{CallBackParams, CallbackReturn},
  parse,
  permissions::member_channel_read,
};
use crate::database::{NewProject, INSTANCE};
use chrono::{offset::Utc, DateTime};
use futures::{executor::block_on, FutureExt};
use log::error;
use procedural_macros::command;
use serenity::{
  http::Http,
  model::{
    channel::{
      Channel, ChannelType, GuildChannel, Message, PermissionOverwriteType, Reaction, ReactionType,
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

fn project_init<'fut>(
  project_args: HashMap<&'fut str, &'fut str>,
  project_chan: ChannelId,
  message: &'fut Message,
  http: &'fut Arc<Http>,
) -> CallbackReturn<'fut> {
  async move {
    let system_time = SystemTime::now();
    let datetime: DateTime<Utc> = system_time.into();

    let overwrite = member_channel_read(message.author.id);
    project_chan.create_permission(http, &overwrite).await?;

    let client = project_args.get("client").unwrap_or(&"");
    let codex = project_args.get("codex").unwrap_or(&"#PXXX");
    let author_name = &*message.author.name;
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
      project_chan.0,
      datetime.format("%d/%m/%Y"),
      client,
      codex,
      lead,
      deadline,
      description,
      contexte,
    );
    let annoucement_message = ChannelId(PROJECT_ANOUNCEMENT_CHANNEL)
      .say(http, content)
      .await?;
    let channel_message = project_chan.say(http, content).await?;
    channel_message.pin(http).await?;
    {
      let mut db_instance = INSTANCE.write().unwrap();
      db_instance.project_add(NewProject {
        message_id: annoucement_message.id.0 as i64,
        channel_id: project_chan.0 as i64,
        pinned_message_id: Some(channel_message.id.0 as i64),
        codex: Some(codex),
        client: Some(client),
        lead: Some(lead),
        deadline: Some(deadline),
        description: Some(description),
        contexte: Some(contexte),
      });
    }
    annoucement_message.react(http, '✅').await?;
    if message.channel_id == ChannelId(PROJECT_ANOUNCEMENT_CHANNEL) {
      message.delete(http).await?;
      return Ok(None);
    }
    Ok(Some(String::from(":ok:")))
  }
  .boxed()
}

#[command]
pub async fn create(params: CallBackParams) -> CallbackReturn {
  let project_args = match project_creation_args(&params.args[1..]) {
    Ok(result) => result,
    Err(error) => return Ok(Some(error)),
  };
  let mainguild = parse::get_main_guild(&params.context).await;
  let http = &params.context.http;
  let newchan = mainguild
    .create_channel(http, |channel| {
      channel
        .kind(ChannelType::Text)
        .category(PROJECT_CATEGORY)
        .name(project_args["name"])
    })
    .await?;

  project_init(
    project_args,
    newchan.id,
    params.message,
    &params.context.http,
  )
  .await
}

#[command]
pub async fn add(params: CallBackParams<'_>) -> CallbackReturn {
  let (project_chan_id, _) =
    parse::discord_str_to_id(params.args[1], Some(parse::DiscordIds::Channel))?;
  let project_chan = ChannelId(project_chan_id);
  let project_args = match project_creation_args(&params.args[2..]) {
    Ok(result) => result,
    Err(error) => return Ok(Some(error)),
  };

  project_init(
    project_args,
    project_chan,
    params.message,
    &params.context.http,
  )
  .await
}

#[command]
pub async fn delete(params: CallBackParams) -> CallbackReturn {
  match parse::discord_str_to_id(params.args[1], Some(parse::DiscordIds::Channel)) {
    Ok((target, _)) => {
      let resultcpy;
      {
        let mut db_instance = INSTANCE.write().unwrap();
        let result = db_instance.projects_delete(target)?;
        resultcpy = (String::from(result.0), result.1);
      }
      if let Some(project) = resultcpy.1 {
        let http = &params.context.http;
        ChannelId(project.channel_id as u64).delete(http).await?;
        ChannelId(PROJECT_ANOUNCEMENT_CHANNEL)
          .message(http, project.message_id as u64)
          .await?
          .delete(http)
          .await?;
      };

      Ok(Some(resultcpy.0))
    }
    Err(error) => Ok(Some(error)),
  }
}

#[command]
pub async fn add_user(params: CallBackParams<'_>) -> CallbackReturn {
  let cache = &params.context.cache;
  let http = &params.context.http;
  let usertag = params.args[1];
  let add_perm = |guildchannel: &GuildChannel, userid| {
    let overwrite = member_channel_read(UserId(userid));
    block_on(guildchannel.create_permission(http, &overwrite)).unwrap();
    Ok(Some(format!("Added <@{}> Welcome !", userid)))
  };

  match params
    .message
    .channel(cache)
    .await
    .expect("Channel of message wasn't found")
  {
    Channel::Guild(guildchannel) => {
      match parse::discord_str_to_id(usertag, Some(parse::DiscordIds::User)) {
        Ok((userid, _)) => add_perm(&guildchannel, userid),
        Err(_error) => {
          if let Some(guild) = &guildchannel.guild(cache).await {
            let member = guild.member_named(usertag);
            if let Some(member) = member {
              let userid = member.user.id.0;
              add_perm(&guildchannel, userid)
            } else {
              Ok(Some(format!(
                "Didn't find any user with {} in their name",
                usertag
              )))
            }
          } else {
            panic!("Unable to get guild from cache")
          }
        }
      }
    }
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

//   Ok(Some(String::from(":ok:")))
// }

pub async fn check_subscribe(ctx: &Context, reaction: &Reaction, removed: bool) {
  let emoji_name = match &reaction.emoji {
    ReactionType::Unicode(unicode) => Some(&*unicode),
    _ => None,
  };
  if let Some(unicode) = emoji_name {
    if ["✅"].contains(&&**unicode) {
      let mut project_chanid = 0;
      {
        let db_instance = INSTANCE.read().unwrap();
        if let Some((_index, project)) =
          db_instance.projects_search(reaction.message_id.0 as i64, parse::DiscordIds::Message)
        {
          project_chanid = project.channel_id;
        }
      }

      if project_chanid > 0 {
        if let Some(channel) = ctx.cache.guild_channel(project_chanid as u64).await {
          if removed {
            channel
              .delete_permission(
                &ctx.http,
                PermissionOverwriteType::Member(reaction.user_id.unwrap()),
              )
              .await
              .unwrap();
          } else {
            let overwrite = member_channel_read(reaction.user_id.unwrap());
            channel
              .create_permission(&ctx.http, &overwrite)
              .await
              .unwrap();
          }
        } else {
          error!("Unable to find project channel in cache");
        }
      }
    }
  }
}
