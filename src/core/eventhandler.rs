use super::api;
use super::process::{
  annoy_channel, archive_activity, attacked, database_update, filter_outannoying_messages,
  personal_attack, process_command, process_contains, process_tag_msg, split_args,
};
use super::validation::{check_validation, WaitingValidation};
use crate::features::{invite_action, mecleanup, project_manager, Features};
use log::{error, info};
use serenity::{
  async_trait,
  client::bridge::gateway::GatewayIntents,
  model::{
    channel::{Message, Reaction},
    event::ResumedEvent,
    gateway::Ready,
    guild::Member,
    id::{GuildId, UserId},
  },
  prelude::*,
};
use std::{env, process, thread};

async fn getbotid(ctx: &Context) -> UserId {
  ctx.cache.current_user_id().await
}
/// Struct that old Traits Implementations to Handle the different events send by discord.
struct Handler;

#[async_trait]
impl EventHandler for Handler {
  /// Set a handler for the `message` event - so that whenever a new message
  /// is received - the closure (or function) passed will be called.
  ///
  /// Event handlers are dispatched through a threadpool, and so multiple
  /// events can be dispatched simultaneously.
  async fn message(&self, ctx: Context, message: Message) {
    let chan = message.channel(&ctx.cache).await.unwrap();
    let chanid = chan.id().to_string();
    let chan_name = match &chan.guild() {
      Some(guildchan) => String::from(guildchan.name()),
      None => chanid,
    };
    info!(
      "[{}]({}) > {} says: {}",
      message.timestamp, chan_name, message.author.name, message.content
    );

    database_update(&message);
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
      check_validation(&ctx, &reaction).await;
      project_manager::check_subscribe(&ctx, &reaction, false).await;
      mecleanup::check_mecleanup(&ctx, &reaction).await;
    }
  }

  async fn reaction_remove(&self, ctx: Context, reaction: Reaction) {
    let botid = getbotid(&ctx).await;
    if reaction.user_id.unwrap() != botid {
      project_manager::check_subscribe(&ctx, &reaction, true).await;
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

  async fn unknown(&self, _ctx: Context, name: String, raw: serde_json::value::Value) {
    info!("{} => {:?}", name, raw);
  }

  async fn resume(&self, ctx: Context, _: ResumedEvent) {
    info!("Resumed");
    let data = &mut ctx.data.write().await;
    // FIXME: This is cool but we never get a stoped event
    data.get_mut::<Features>().unwrap().thread_control.resume();
  }
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
    .event_handler(Handler)
    .intents(GatewayIntents::all());
  let mut client = builder.await.expect("Err creating client");
  {
    let mut data = client.data.write().await;
    data.insert::<Features>(Features::new());
    data.insert::<WaitingValidation>(WaitingValidation::default());
  }

  let cache_arc = client.cache_and_http.clone();
  thread::spawn(|| {
    api::run(cache_arc).unwrap();
  });

  // Finally, start a single shard, and start listening to events.
  // Shards will automatically attempt to reconnect, and will perform
  // exponential backoff until it reconnects.
  if let Err(why) = client.start().await {
    error!("Client error: {:?}", why);
  }
}
