use super::process::{
  annoy_channel, archive_activity, attacked, database_update, filter_outannoying_messages,
  personal_attack, process_command, process_contains, process_tag_msg, split_args,
  trigger_inchannel,
};
use super::validation::{check_validation, WaitingValidation};
use super::{api, slash_command};
use crate::features::{invite_action, mecleanup, project_manager, Features};
use log::{error, info};
use serenity::model::id::ChannelId;
use serenity::{
  async_trait,
  client::bridge::gateway::GatewayIntents,
  model::{
    channel::{Message, Reaction},
    event::{MessageUpdateEvent, ResumedEvent},
    gateway::Ready,
    guild::Member,
    id::{GuildId, UserId},
    interactions::Interaction,
  },
  prelude::*,
};
use std::{
  env, process,
  sync::atomic::{AtomicBool, Ordering},
};

async fn getbotid(ctx: &Context) -> UserId {
  ctx.cache.current_user_id().await
}
/// Struct that old Traits Implementations to Handle the different events send by discord.
struct Handler {
  is_loop_running: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
  /// Set a handler for the `message` event - so that whenever a new message
  /// is received - the closure (or function) passed will be called.
  ///
  /// Event handlers are dispatched through a threadpool, and so multiple
  /// events can be dispatched simultaneously.
  async fn message(&self, ctx: Context, message: Message) {
    let chan_name = get_channel_name(&message.channel_id, &ctx).await;
    info!(
      "[{}]({}) > {} says: {}",
      message.timestamp, chan_name, message.author.name, message.content
    );

    database_update((&message).into(), false);
    archive_activity(&ctx, &message).await;
    if message.is_own(&ctx).await || message.content.is_empty() {
      return;
    };
    personal_attack(&ctx, &message).await;
    annoy_channel(&ctx, &message).await;
    filter_outannoying_messages(&ctx, &message);

    //Check if i am tagged in the message else do the reactions
    // check for @me first so it's considered a command
    let botid = getbotid(&ctx).await.0;
    if message.content.starts_with(&*format!("<@!{}>", botid))
      || message.content.starts_with(&*format!("<@{}>", botid))
    {
      if attacked(&ctx, &message).await {
        return;
      }
      let line = message.content.clone();
      let mut message_split = split_args(&line);

      // Check if there is only the tag : "@bot"
      if message_split.len() == 1 {
        message
          .channel_id
          .say(&ctx.http, "What do you need ?")
          .await
          .unwrap();
        return;
      }
      // Removing tag
      message_split.remove(0);

      // will go through commands.rs definitions to try and execute the request
      if !process_tag_msg(&message_split, &message, &ctx).await
        && !process_command(&message_split, &message, &ctx).await
      {
        message
          .channel_id
          .say(&ctx.http, "How about a proper request ?")
          .await
          .unwrap();
      }
    } else {
      process_contains(&message, &ctx).await;
    }
    trigger_inchannel(&message, &ctx).await;
  }

  async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
    let botid = getbotid(&ctx).await;
    let isown = reaction
      .message(&ctx.http)
      .await
      .unwrap()
      .is_own(&ctx.cache)
      .await;

    if reaction.user_id.unwrap() != botid && isown {
      let emoji = reaction.emoji.as_data();
      match &*emoji {
        "✅" => {
          project_manager::check_subscribe(&ctx, &reaction, false).await;
          check_validation(&ctx, &reaction, &emoji).await;
        }
        "❌" => {
          check_validation(&ctx, &reaction, &emoji).await;
        }
        "🧹" => {
          mecleanup::check_mecleanup(&ctx, &reaction).await;
        }
        _ => {}
      }
    }
  }

  async fn reaction_remove(&self, ctx: Context, reaction: Reaction) {
    let botid = getbotid(&ctx).await;
    let isown = reaction
      .message(&ctx.http)
      .await
      .unwrap()
      .is_own(&ctx.cache)
      .await;

    if reaction.user_id.unwrap() != botid && isown {
      let emoji = reaction.emoji.as_data();
      #[allow(clippy::single_match)] // TODO: remove this when we have more eventualy
      match &*emoji {
        "✅" => {
          project_manager::check_subscribe(&ctx, &reaction, true).await;
        }
        _ => {}
      }
    }
  }

  async fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, mut new_member: Member) {
    invite_action::on_new_member_check(ctx, &guild_id, &mut new_member).await;
  }

  async fn ready(&self, ctx: Context, ready: Ready) {
    info!("{} is connected!", ready.user.name);

    let data = &mut ctx.data.write().await;
    let feature = data.get_mut::<Features>().unwrap();
    if !feature.running {
      feature.running = true;
      feature.run(&ctx.http);
    }
  }

  async fn message_update(
    &self,
    ctx: Context,
    _old_if_available: Option<Message>,
    _new: Option<Message>,
    event: MessageUpdateEvent,
  ) {
    let chan_name = get_channel_name(&event.channel_id, &ctx).await;
    let event_clone = event.clone();
    info!(
      "[{}]({}) > {} edited message with: {}",
      event_clone.timestamp.unwrap(),
      chan_name,
      event_clone.author.unwrap_or_default().name,
      event_clone.content.unwrap_or_default(),
    );

    database_update((&event).into(), true);
  }

  async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    slash_command::handle_event(interaction, ctx).await;
  }

  async fn unknown(&self, _ctx: Context, name: String, raw: serde_json::value::Value) {
    info!("{} => {:?}", name, raw);
  }

  async fn resume(&self, ctx: Context, _: ResumedEvent) {
    info!("Resumed");
    let data = &mut ctx.data.write().await;
    // FIXME: This is cool but we never get a stoped event
    data.get_mut::<Features>().unwrap().thread_control.resume();
  }

  async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
    info!("Cache ready");
    if !self.is_loop_running.load(Ordering::Relaxed) {
      tokio::spawn(async move { api::run(ctx).await });
      self.is_loop_running.swap(true, Ordering::Relaxed);
    }
  }
}

async fn get_channel_name(channel_id: &ChannelId, ctx: &Context) -> String {
  let chan = channel_id.to_channel(&ctx.http).await.unwrap();
  let chanid = chan.id().to_string();
  let chan_name = match &chan.guild() {
    Some(guildchan) => String::from(guildchan.name()),
    None => chanid,
  };
  chan_name
}

/// Get the discord token from `CREDENTIALS_FILE` and run the client.
#[tokio::main]
pub async fn bot_connect() {
  info!("Bot Connecting");

  let token: String = match env::var("token") {
    Ok(token) => token,
    Err(error) => {
      error!("Token error: {}", error);
      process::exit(0x001);
    }
  };

  // Create a new instance of the Client, logging in as a bot. This will
  // automatically prepend your bot token with "Bot ", which is a requirement
  // by Discord for bot users.
  let builder = Client::builder(token)
    .event_handler(Handler {
      is_loop_running: AtomicBool::new(false),
    })
    .intents(GatewayIntents::all());
  let mut client = builder.await.expect("Err creating client");
  {
    let mut data = client.data.write().await;
    data.insert::<Features>(Features::new());
    data.insert::<WaitingValidation>(WaitingValidation::default());
  }

  // Finally, start a single shard, and start listening to events.
  // Shards will automatically attempt to reconnect, and will perform
  // exponential backoff until it reconnects.
  if let Err(why) = client.start().await {
    error!("Client error: {:?}", why);
  }
}
