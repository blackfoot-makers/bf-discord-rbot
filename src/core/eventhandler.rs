use super::process::{
    annoy_channel, archive_activity, attacked, database_update, filter_outannoying_messages,
    personal_attack, process_command, process_contains, process_tag_msg, split_args, CACHE,
    HTTP_STATIC,
};
use super::validation::check_validation;
use crate::features::Features;
use log::{error, info};
use serenity::{
    model::channel::{Message, Reaction},
    model::{event::ResumedEvent, gateway::Ready},
    prelude::*,
};
use std::{env, process};

/// Struct that old Traits Implementations to Handle the different events send by discord.
struct Handler;

impl EventHandler for Handler {
    /// Set a handler for the `message` event - so that whenever a new message
    /// is received - the closure (or function) passed will be called.
    ///
    /// Event handlers are dispatched through a threadpool, and so multiple
    /// events can be dispatched simultaneously.
    fn message(&self, ctx: Context, message: Message) {
        println!("{} says: {}", message.author.name, message.content);

        database_update(&message);
        archive_activity(&ctx, &message);
        if message.is_own(&ctx) || message.content.is_empty() {
            return;
        };
        personal_attack(&ctx, &message);
        annoy_channel(&ctx, &message);
        filter_outannoying_messages(&ctx, &message);

        //Check if i am tagged in the message else do the reactions
        // check for @me first so it's considered a command
        let cache = ctx.cache.read();
        let userid = cache.user.id.as_u64();
        if message.content.starts_with(&*format!("<@!{}>", userid))
            || message.content.starts_with(&*format!("<@{}>", userid))
        {
            if attacked(&ctx, &message) {
                return;
            }
            let line = message.content.clone();
            let author: &str = &message.author.tag();
            let mut message_split = split_args(&line);

            // Check if there is only the tag : "@bot"
            if message_split.len() == 1 {
                message
                    .channel_id
                    .say(&ctx.http, "What do you need ?")
                    .unwrap();
                return;
            }

            // Removing tag
            message_split.remove(0);
            message_split.push(author);

            // will go through commands.rs definitions to try and execute the request
            if !process_tag_msg(&message_split, &message, &ctx)
                && !process_command(&message_split, &message, &ctx)
            {
                message
                    .channel_id
                    .say(&ctx.http, "How about a proper request ?")
                    .unwrap();
            }
        } else {
            process_contains(&message, &ctx);
        }
    }

    fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let userid: u64;
        {
            let cache = ctx.cache.read();
            userid = *cache.user.id.as_u64();
        }
        if reaction.user_id.0 != userid {
            // parse_githook_reaction(ctx, reaction);
            check_validation(ctx, reaction);
        }
    }

    fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        let mut arc = HTTP_STATIC.write();
        *arc = Some(ctx.http.clone());
        let mut cache = CACHE.write();
        *cache = ctx.cache;

        let data = &mut ctx.data.write();
        let feature = data.get_mut::<Features>().unwrap();
        if !feature.running {
            feature.running = true;
            feature.run(&ctx.http);
        }
    }

    fn unknown(&self, _ctx: Context, name: String, raw: serde_json::value::Value) {
        info!("{} => {:?}", name, raw);
    }

    fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
        // let data = &mut ctx.data.write();
        // data.get_mut::<Features>().unwrap().thread_control.resume();
    }
}

/// Get the discord token from `CREDENTIALS_FILE` and run the client.
pub fn bot_connect() {
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
    let mut client = Client::new(token, Handler).expect("Err creating client");
    {
        let mut data = client.data.write();
        data.insert::<Features>(Features::new());
    }

    // Finally, start a single shard, and start listening to events.
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}
