use crate::{
  constants,
  core::{
    commands::{CallBackParams, CallbackReturn},
    parse::{self, discord_str_to_id},
    permissions::member_channel_read,
  },
};
use crate::{
  constants::discordids::{PROJECT_ANOUNCEMENT_CHANNEL, PROJECT_CATEGORY},
  database,
};
use crate::{
  core::parse::DiscordIds,
  database::{NewProject, INSTANCE},
};
use chrono::{offset::Utc, DateTime};
use futures::FutureExt;
use log::error;
use procedural_macros::command;
use serenity::{
  http::Http,
  model::{
    channel::{
      Channel, ChannelType, GuildChannel, Message, PermissionOverwriteType, Reaction, ReactionType,
    },
    guild::Guild,
    id::{ChannelId, UserId},
  },
  prelude::*,
};
use std::{collections::HashMap, error::Error, sync::Arc, time::SystemTime};

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

async fn add_permission(
  context: &Context,
  guildchannel: &GuildChannel,
  userid: u64,
) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
  let overwrite = member_channel_read(UserId(userid));
  guildchannel
    .create_permission(&context.http, &overwrite)
    .await
    .unwrap();
  Ok(Some(format!("Added <@{}> Welcome !", userid)))
}

#[command]
pub async fn add_user(params: CallBackParams<'_>) -> CallbackReturn {
  let cache = &params.context.cache;
  let usertag = params.args[1];

  match params
    .message
    .channel(cache)
    .await
    .expect("Channel of message wasn't found")
  {
    Channel::Guild(guildchannel) => {
      match parse::discord_str_to_id(usertag, Some(parse::DiscordIds::User)) {
        Ok((userid, _)) => add_permission(&params.context, &guildchannel, userid).await,
        Err(_error) => {
          if let Some(guild) = &guildchannel.guild(cache).await {
            let member = guild.member_named(usertag);
            if let Some(member) = member {
              let userid = member.user.id.0;
              add_permission(&params.context, &guildchannel, userid).await
            } else {
              check_containing(&params.context, guild, usertag, guildchannel).await
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

async fn check_containing(
  context: &Context,
  guild: &Guild,
  usertag: &str,
  guildchannel: GuildChannel,
) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
  let members = guild.members_containing(usertag, false, false).await;
  if !members.is_empty() {
    if members.len() == 1 {
      let userid = members[0].0.user.id.0;
      add_permission(context, &guildchannel, userid).await
    } else {
      let membernames: String = members
        .iter()
        .map(|member| format!("{}, ", member.1))
        .collect();
      Ok(Some(format!(
        "Found too many users with this name: {},\n found: {}",
        usertag, membernames
      )))
    }
  } else {
    Ok(Some(format!(
      "Didn't find any user with {} in their name",
      usertag
    )))
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

pub async fn bottom_list_current(context: &Context, message: &Message) {
  delete_previous_bottom_message(context).await;
  let text_projects_channels = list_projects(message, context).await;

  for channel_chunk in text_projects_channels.chunks(11) {
    let mut list_message = String::new();
    let mut list_channels = String::new();
    for (index, channel) in channel_chunk.iter().enumerate() {
      let project_item = &*format!("{}\t{}\n", constants::NUMBERS[index], channel.1.mention());
      list_message.push_str(project_item);

      list_channels.push_str(&*format!("{},", channel.1.id.0));
    }
    let message = ChannelId(PROJECT_ANOUNCEMENT_CHANNEL)
      .say(&context.http, list_message)
      .await
      .unwrap();
    for index in 0..channel_chunk.len() {
      message
        .react(
          &context.http,
          ReactionType::Unicode(String::from(constants::NUMBERS[index])),
        )
        .await
        .unwrap();
    }

    {
      let mut db_instance = database::INSTANCE.write().unwrap();
      let time: SystemTime = SystemTime::from(message.timestamp);
      db_instance.storage_add(database::NewStorage {
        datatype: database::StorageDataType::ProjectBottomMessage.into(),
        data: &*list_channels,
        dataid: Some(*message.id.as_u64() as i64),
        date: Some(time),
      });
    }
  }
}

async fn list_projects<'a>(message: &Message, context: &Context) -> Vec<(ChannelId, GuildChannel)> {
  let gid = message.guild_id.unwrap();
  let cache = context.cache.clone();
  let guild = cache
    .guild(gid)
    .await
    .expect("Critical: Guild from message not found");
  let text_projects_channels: Vec<_> = guild
    .channels
    .iter()
    .filter(|(_, chan)| {
      chan.kind == ChannelType::Text
        && match chan.category_id {
          Some(chan) => chan == PROJECT_CATEGORY,
          _ => false,
        }
    })
    .map(|e| (*e.0, e.1.clone()))
    .collect();

  text_projects_channels
}

async fn delete_previous_bottom_message(context: &Context) {
  let previous_bottom_list_messages;
  {
    let mut db_instance = database::INSTANCE.write().unwrap();
    let ids_previous_bottom_message = db_instance
      .storage
      .iter()
      .filter(|stored| stored.datatype == database::StorageDataType::ProjectBottomMessage as i64)
      .map(|stored| stored.id)
      .collect();
    previous_bottom_list_messages = db_instance.storage_delete(ids_previous_bottom_message);
  }
  for stored in previous_bottom_list_messages {
    ChannelId(constants::discordids::PROJECT_ANOUNCEMENT_CHANNEL)
      .message(&context.http, stored.dataid.unwrap() as u64)
      .await
      .unwrap()
      .delete(&context.http)
      .await
      .unwrap();
  }
}

pub async fn check_subscribe_bottom_list(
  ctx: &Context,
  reaction: &Reaction,
  removed: bool,
  emoji: &str,
) {
  let channels_id = {
    let db_instance = database::INSTANCE.write().unwrap();
    db_instance
      .storage
      .iter()
      .find(|stored| stored.dataid.unwrap() == reaction.message_id.0 as i64)
      .unwrap()
      .data
      .clone()
  };
  let vec_channels_id: Vec<&str> = channels_id.split(',').collect();
  let number = constants::NUMBERS
    .iter()
    .position(|number| number == &&*emoji);
  let channel_id = if let Some(number) = number {
    if let Some(channel_id) = vec_channels_id.get(number) {
      channel_id
    } else {
      return;
    }
  } else {
    return;
  };

  if removed {
    ChannelId(
      parse::discord_str_to_id(channel_id, Some(DiscordIds::Channel))
        .unwrap()
        .0,
    )
    .delete_permission(
      &ctx.http,
      PermissionOverwriteType::Member(reaction.user_id.unwrap()),
    )
    .await
    .unwrap();
  } else {
    let overwrite = member_channel_read(reaction.user_id.unwrap());
    ChannelId(
      parse::discord_str_to_id(channel_id, Some(DiscordIds::Channel))
        .unwrap()
        .0,
    )
    .create_permission(&ctx.http, &overwrite)
    .await
    .unwrap();
  }
}

#[command]
pub async fn remove_user_from_all(params: CallBackParams<'_>) -> CallbackReturn {
  if let Ok((useid, _)) = discord_str_to_id(params.args[1], Some(DiscordIds::User)) {
    let guild = params
      .message
      .guild(&params.context)
      .await
      .expect("Unable to get guild from cache");
    let text_projects_channels: Vec<_> = guild
      .channels
      .iter()
      .filter(|(_, chan)| {
        chan.kind == ChannelType::Text
          && match chan.category_id {
            Some(chan) => chan == PROJECT_CATEGORY,
            _ => false,
          }
      })
      .collect();

    for (channel_id, _) in text_projects_channels {
      channel_id
        .delete_permission(
          &params.context.http,
          PermissionOverwriteType::Member(UserId(useid)),
        )
        .await
        .unwrap();
    }

    Ok(Some(String::from(":ok:")))
  } else {
    Ok(Some(String::from("Unable to parse userid")))
  }
}
