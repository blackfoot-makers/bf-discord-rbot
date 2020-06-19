//! Handle the connection with discord and it's events.
use crate::{database, features::Features};
use log::{error, info};
use rand;
use serenity::{
    cache, http,
    model::channel::{Message, Reaction},
    model::{
        event::ResumedEvent,
        gateway::Ready,
        id::{ChannelId, UserId},
    },
    prelude::*,
};
use std::{env, process, str::FromStr, sync::Arc};

use super::commands::{
    ATTACKED, COMMANDS_LIST, CONTAIN_MSG_LIST, CONTAIN_REACTION_LIST, TAG_MSG_LIST,
};

/// Struct that old Traits Implementations to Handle the different events send by discord.
struct Handler;

lazy_static! {
    pub static ref HTTP_STATIC: RwLock<Option<Arc<http::Http>>> = RwLock::new(None);
    pub static ref CACHE: RwLock<cache::CacheRwLock> = RwLock::new(cache::CacheRwLock::default());
}

fn allowed_channel(
    command_channel: Option<ChannelId>,
    message_channel: ChannelId,
    ctx: &Context,
) -> bool {
    match command_channel {
        Some(ref chan) => {
            if chan != &message_channel {
                message_channel
                    .say(&ctx.http, format!("I am not allowed to issue this command in this channel ! Use {} instead.", chan.mention())).unwrap();
                return false;
            }
            true
        }
        None => true,
    }
}

fn allowed_user(expected: database::Role, user: &database::User) -> bool {
    let role = match database::Role::from_str(&*user.role) {
        Err(e) => {
            println!("Error {}", e);
            return false;
        }
        Ok(role) => (role),
    };

    role as u32 >= expected as u32
}

fn process_command(message_split: &[&str], message: &Message, ctx: &Context) -> bool {
    for (key, command) in COMMANDS_LIST.iter() {
        if *key == message_split[0] && allowed_channel(command.channel, message.channel_id, ctx) {
            {
                let db_instance = database::INSTANCE.read().unwrap();
                let user: &database::User = db_instance
                    .user_search(*message.author.id.as_u64())
                    .unwrap();
                if !allowed_user(command.permission, &user) {
                    message
                        .channel_id
                        .send_message(&ctx.http, |m| {
                            m.content(format!(
                                "You({}) are not allowed to run this command",
                                user.role
                            ))
                        })
                        .unwrap();
                    return true;
                }
            }
            // We remove default arguments: author and command name from the total
            let arguments = message_split.len() - 2;
            let result = if arguments >= command.argument_min && arguments <= command.argument_max {
                (command.exec)(message_split, &ctx)
            } else {
                format!("Usage: {}", command.usage.clone())
            };

            if !result.is_empty() {
                message
                    .channel_id
                    .send_message(&ctx.http, |m| m.content(result))
                    .unwrap();
            }
            return true;
        }
    }
    false
}

fn process_tag_msg(message_split: &[&str], message: &Message, ctx: &Context) -> bool {
    for (key, reaction) in TAG_MSG_LIST.iter() {
        if *key == message_split[0] {
            message.channel_id.say(&ctx.http, reaction).unwrap();
            return true;
        }
    }
    false
}

fn process_contains(message: &Message, ctx: &Context) {
    for (key, text) in CONTAIN_MSG_LIST.iter() {
        if message.content.contains(key) {
            message.channel_id.say(&ctx.http, *text).unwrap();
        }
    }

    for (key, reaction) in CONTAIN_REACTION_LIST.iter() {
        if message.content.contains(key) {
            message.react(ctx, *reaction).unwrap();
        }
    }
}

fn split_args(input: &str) -> Vec<&str> {
    let mut count = 0;
    let message_split_quote: Vec<&str> = input.split('"').collect();
    let mut result: Vec<&str> = Vec::new();
    for msg in message_split_quote {
        if msg.is_empty() {
            continue;
        }
        count += 1;
        if (count % 2) == 0 {
            result.push(msg);
        } else {
            let mut message_split_space: Vec<&str> = msg.trim().split(' ').collect();
            result.append(&mut message_split_space);
        }
    }
    result
}

const CATS: [&str; 12] = [
    "😺", "😸", "😹", "😻", "😼", "😽", "🙀", "😿", "😾", "🐈", "🐁", "🐭",
];
const KEYS: [&str; 8] = ["🔑", "🗝", "🔏", "🔐", "🔒", "🔓", "🖱", "👓"];

const HERDINGCHATTE: ChannelId = ChannelId(570275817804791809);
const CYBERGOD: ChannelId = ChannelId(588666452849065994);
const TESTBOT: ChannelId = ChannelId(555206410619584519);
/// Anoying other channels
fn annoy_channel(ctx: &Context, message: &Message) {
    if message.channel_id == HERDINGCHATTE {
        let random_active = rand::random::<usize>() % 10;
        if random_active == 0 {
            let random_icon = rand::random::<usize>() % CATS.len();
            message.react(ctx, CATS[random_icon]).unwrap();
        }
    }
    if message.channel_id == CYBERGOD {
        let random_active = rand::random::<usize>() % 10;
        if random_active == 0 {
            let random_icon = rand::random::<usize>() % KEYS.len();
            message.react(ctx, KEYS[random_icon]).unwrap();
        }
    }
    if message.channel_id == TESTBOT {
        let random_active = rand::random::<usize>() % 10;
        if random_active == 0 {
            let random_icon = rand::random::<usize>() % KEYS.len();
            message.react(ctx, KEYS[random_icon]).unwrap();
        }
    }
}

const FILTERED: [&str; 1] = ["🔥"];
const PM: UserId = UserId(365228504817729539);
fn filter_outannoying_messages(ctx: &Context, message: &Message) {
    if message.author.id != PM {
        return;
    }
    for annoying in FILTERED.iter() {
        if message.content.replace(annoying, "").trim().is_empty() {
            println!("Has been filtered !");
            let _ = message.delete(ctx);
        }
    }
}

const ANNOYING: [&str; 6] = [
    "Ah oui mais y'a JPO",
    "Vous pourriez faire ça vous meme s'il vous plaît ? Je suis occupé",
    "Avant, Faut laver les vitres les gars",
    "Ah mais vous faites quoi ?",
    "Non mais tu as vu le jeu qui est sorti ?",
    "Je bosse sur un projet super innovant en ce moment, j'ai pas le temps",
];

fn personal_attack(ctx: &Context, message: &Message) {
    if message.author.mention() == *ATTACKED.read() {
        const ANNOYING: [&str; 11] = [
            "🐧", "💩", "🍌", "💣", "👾", "🐔", "📛", "🔥", "‼", "⚡", "⚠",
        ];
        let random1 = rand::random::<usize>() % ANNOYING.len();
        let random2 = rand::random::<usize>() % ANNOYING.len();
        message.react(ctx, ANNOYING[random1]).unwrap();
        message.react(ctx, ANNOYING[random2]).unwrap();
    }
}

// fn parse_githook_reaction(ctx: Context, reaction: Reaction) {
// 	let channel = ChannelId(555206410619584519); //TODO : Channel register

// 	let emoji_name = match &reaction.emoji {
// 		ReactionType::Unicode(e) => e.clone(),
// 		ReactionType::Custom {
// 			animated: _,
// 			name,
// 			id: _,
// 		} => name.clone().unwrap(),
// 		_ => "".to_string(),
// 	};
// 	debug!("Reaction emoji: {}", emoji_name);
// 	if reaction.channel_id == channel {
// 		if emoji_name == "✅" {
// 			let message = reaction.message(&ctx.http).unwrap();
// 			if message.is_own(&ctx.cache) {
// 				let closing_tag = message.content.find("]").unwrap_or_default();
// 				if closing_tag > 0 {
// 					let params = &message.content[1..closing_tag];
// 					let params_split: Vec<&str> = params.split('/').collect();
// 					if params_split.len() == 3 {
// 						// features::docker::deploy_from_reaction(
// 						// 	params_split[0].to_string(),
// 						// 	params_split[1].to_string(),
// 						// 	params_split[2].to_string(),
// 						// 	ctx.http.clone(),
// 						// );
// 						// return;
// 					}
// 				}

// 				eprintln!(
// 					"Reaction/githook: Invalid params parse : [{}]",
// 					message.content
// 				);
// 			}
// 		}
// 	}
// }

fn database_update(message: &Message) {
    let mut db_instance = database::INSTANCE.write().unwrap();
    let author_id = *message.author.id.as_u64() as i64;
    if !db_instance.users.iter().any(|e| e.discordid == author_id) {
        db_instance.user_add(author_id, &*database::Role::Guest.to_string());
    }
    db_instance.message_add(
        *message.id.as_u64() as i64,
        author_id,
        &message.content,
        *message.channel_id.as_u64() as i64,
    );
}

impl EventHandler for Handler {
    /// Set a handler for the `message` event - so that whenever a new message
    /// is received - the closure (or function) passed will be called.
    ///
    /// Event handlers are dispatched through a threadpool, and so multiple
    /// events can be dispatched simultaneously.
    fn message(&self, ctx: Context, message: Message) {
        println!("{} says: {}", message.author.name, message.content);

        database_update(&message);
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
            // Check if Attacked
            if message.author.mention() == *ATTACKED.read() {
                let random = rand::random::<usize>() % 6;
                message.channel_id.say(&ctx.http, ANNOYING[random]).unwrap();
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

    fn reaction_add(&self, _ctx: Context, _reaction: Reaction) {
        // parse_githook_reaction(ctx, reaction);
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
