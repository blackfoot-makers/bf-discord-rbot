use std::{
  env, process,
  sync::atomic::{AtomicBool, Ordering},
};

use log::{error, info};
use serenity::http::CacheHttp;
use serenity::model::id::ChannelId;
use serenity::model::Timestamp;
use serenity::{
  async_trait,
  model::{
    application::interaction::Interaction,
    channel::{Message, Reaction},
    event::{MessageUpdateEvent, ResumedEvent},
    gateway::Ready,
    guild::Member,
    id::GuildId,
  },
  prelude::*,
};

use crate::{
  core::{
    api,
    process::{archive_activity, database_update, getbotid, process_message},
    slash_command,
    validation::check_validation,
    validation::WaitingValidation,
  },
  features::{
    deployment::{DeploymentReactionsData, ValidationEmoji},
    invite_action, mecleanup, project_manager, Features,
  },
};

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

    #[allow(clippy::needless_borrow)]
    // Here clippy is wrong, we actually need to ref message before calling into
    database_update((&message).into(), false);
    archive_activity(&ctx, &message).await;
    if message.is_own(&ctx) || message.content.is_empty() {
      return;
    };
    process_message(ctx, message).await;
  }

  async fn message_update(
    &self,
    ctx: Context,
    _old_if_available: Option<Message>,
    new: Option<Message>,
    event: MessageUpdateEvent,
  ) {
    // Skip embeds as we do not deal with this kind of messages
    if event.embeds.is_some() && !event.embeds.as_ref().unwrap().is_empty() {
      return;
    }

    let chan_name = get_channel_name(&event.channel_id, &ctx).await;
    let event_clone = event.clone();
    info!(
      "[{}]({}) > {} edited message with: {}",
      event_clone.timestamp.unwrap_or_else(Timestamp::now),
      chan_name,
      event_clone.author.unwrap_or_default().name,
      event_clone.content.unwrap_or_default(),
    );
    #[allow(clippy::needless_borrow)]
    // Here clippy is wrong, we actually need to ref message before calling into
    database_update((&event).into(), true);
    let new_message = if let Some(message) = new {
      message
    } else {
      ctx
        .http()
        .get_message(event.channel_id.0, event.id.0)
        .await
        .unwrap()
    };

    if let Some(guild_channel) = ctx.cache.guild_channel(new_message.channel_id) {
      let messages_after = guild_channel
        .messages(&ctx.http, |retriever| {
          retriever.after(new_message.id).limit(1)
        })
        .await
        .expect("Unable to retrieve guild_channel messages");
      if !messages_after.is_empty() {
        let botid = getbotid(&ctx).await;
        let followup_message = messages_after.first().unwrap();
        if followup_message.author.id == botid {
          followup_message.delete(&ctx.http).await.unwrap();
          process_message(ctx, new_message).await;
        }
      }
    }
  }

  async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
    let botid = getbotid(&ctx).await;
    let isown = reaction
      .message(&ctx.http)
      .await
      .unwrap()
      .is_own(&ctx.cache);
    let user_id = reaction.user_id.unwrap();

    if user_id != botid && isown {
      let emoji = reaction.emoji.as_data();

      match &*emoji {
        "✅" | "%E2%9C%85" => {
          project_manager::check_subscribe(&ctx, &reaction, false).await;
          check_validation(&ctx, &reaction, &emoji).await;
          DeploymentReactionsData::validate(&ctx, &reaction, ValidationEmoji::Approve).await;
        }
        "❌" | "%E2%9D%8C" => {
          check_validation(&ctx, &reaction, &emoji).await;
          DeploymentReactionsData::validate(&ctx, &reaction, ValidationEmoji::Reject).await;
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
      .is_own(&ctx.cache);

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

  async fn guild_member_addition(&self, ctx: Context, mut new_member: Member) {
    invite_action::on_new_member_check(ctx, &mut new_member).await;
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
  let builder = Client::builder(token, GatewayIntents::all()).event_handler(Handler {
    is_loop_running: AtomicBool::new(false),
  });
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
