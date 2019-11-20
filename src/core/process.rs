//! Handle the connection with discord and it's events.
use super::credentials::CREDENTIALS_FILE;
use features;
use log::{debug, error, info};
use rand;
use serenity::http;
use serenity::{
	model::channel::{Message, Reaction, ReactionType},
	model::{event::ResumedEvent, gateway::Ready, id::ChannelId},
	prelude::*,
};

use super::commands::{
	ATTACKED, COMMANDS_LIST, CONTAIN_MSG_LIST, CONTAIN_REACTION_LIST, TAG_MSG_LIST,
};

/// Struct that old Traits Implementations to Handle the different events send by discord.
struct Handler;

lazy_static! {
	pub static ref HTTP_STATIC: RwLock<Option<std::sync::Arc<http::raw::Http>>> = RwLock::new(None);
}

fn allowed_channel(
	command_channel: Option<ChannelId>,
	message_channel: ChannelId,
	ctx: &Context,
) -> bool {
	return match command_channel {
		Some(ref chan) => {
			if chan != &message_channel {
				message_channel
					.say(
						&ctx.http,
						format!(
							"I am not allowed to issue this command in this channel ! Use {} instead.",
							chan.mention()
						),
					)
					.unwrap();
				return false;
			}
			return true;
		}
		None => true,
	};
}

fn process_command(message_split: &Vec<&str>, message: &Message, ctx: &Context) -> bool {
	for (key, command) in COMMANDS_LIST.iter() {
		if *key == message_split[0] {
			if allowed_channel(command.channel, message.channel_id, ctx) {
				// We remove default arguments: author and command name from the total
				let arguments = message_split.len() - 2;
				let result = if arguments >= command.argument_min && arguments <= command.argument_max {
					(command.exec)(message_split)
				} else {
					command.usage.clone()
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
	}
	false
}

fn process_tag_msg(message_split: &Vec<&str>, message: &Message, ctx: &Context) -> bool {
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

fn split_args(input: &String) -> Vec<&str> {
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
	println!("result: {:?}", &result);
	result
}

const CATS: [&str; 12] = [
	"😺", "😸", "😹", "😻", "😼", "😽", "🙀", "😿", "😾", "🐈", "🐁", "🐭",
];
const KEYS: [&str; 8] = ["🔑", "🗝", "🔏", "🔐", "🔒", "🔓", "🖱", "👓"];

/// Anoying other channels
fn annoy_channel(ctx: &Context, message: &Message) {
	let herdingchatte = ChannelId(570275817804791809);
	let cybergod = ChannelId(588666452849065994);
	let testbot = ChannelId(555206410619584519);

	if message.channel_id == herdingchatte {
		let random_active = rand::random::<usize>() % 10;
		if random_active == 0 {
			let random_icon = rand::random::<usize>() % CATS.len();
			message.react(ctx, CATS[random_icon]).unwrap();
		}
	}
	if message.channel_id == cybergod {
		let random_active = rand::random::<usize>() % 10;
		if random_active == 0 {
			let random_icon = rand::random::<usize>() % KEYS.len();
			message.react(ctx, KEYS[random_icon]).unwrap();
		}
	}
	if message.channel_id == testbot {
		let random_active = rand::random::<usize>() % 10;
		if random_active == 0 {
			let random_icon = rand::random::<usize>() % KEYS.len();
			message.react(ctx, KEYS[random_icon]).unwrap();
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

fn parse_githook_reaction(ctx: Context, reaction: Reaction) {
	let channel = ChannelId(555206410619584519); //TODO : Channel register

	let emoji_name = match &reaction.emoji {
		ReactionType::Unicode(e) => e.clone(),
		ReactionType::Custom {
			animated: _,
			name,
			id: _,
		} => name.clone().unwrap(),
		_ => "".to_string(),
	};
	debug!("Reaction emoji: {}", emoji_name);
	if reaction.channel_id == channel {
		if emoji_name == "✅" {
			let message = reaction.message(&ctx.http).unwrap();
			if message.is_own(&ctx.cache) {
				let closing_tag = message.content.find("]").unwrap_or_default();
				if closing_tag > 0 {
					let params = &message.content[1..closing_tag];
					let params_split: Vec<&str> = params.split('/').collect();
					if params_split.len() == 3 {
						features::docker::deploy_from_reaction(
							params_split[0].to_string(),
							params_split[1].to_string(),
							params_split[2].to_string(),
							ctx.http.clone(),
						);
						return;
					}
				}

				eprintln!(
					"Reaction/githook: Invalid params parse : [{}]",
					message.content
				);
			}
		}
	}
}

impl EventHandler for Handler {
	/// Set a handler for the `message` event - so that whenever a new message
	/// is received - the closure (or function) passed will be called.
	///
	/// Event handlers are dispatched through a threadpool, and so multiple
	/// events can be dispatched simultaneously.
	fn message(&self, ctx: Context, message: Message) {
		println!("{} says: {}", message.author.name, message.content);
		if message.is_own(&ctx) || message.content.is_empty() {
			return;
		};
		personal_attack(&ctx, &message);
		annoy_channel(&ctx, &message);

		//Check if i am tagged in the message else do the reactions
		// check for @me first so it's considered a command
		if message
			.content
			.starts_with(&*format!("<@{}>", ctx.cache.read().user.id.as_u64()))
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

	fn reaction_add(&self, ctx: Context, reaction: Reaction) {
		parse_githook_reaction(ctx, reaction);
	}

	fn ready(&self, ctx: Context, ready: Ready) {
		info!("{} is connected!", ready.user.name);
		let mut arc = HTTP_STATIC.write();
		*arc = Some(ctx.http.clone());
		features::run(&ctx.http);
	}

	fn resume(&self, _: Context, _: ResumedEvent) {
		info!("Resumed");
	}
}

/// Get the discord token from `CREDENTIALS_FILE` and run the client.
pub fn bot_connect() {
	info!("Bot Connecting");

	let token = &CREDENTIALS_FILE.stored.token;

	// Create a new instance of the Client, logging in as a bot. This will
	// automatically prepend your bot token with "Bot ", which is a requirement
	// by Discord for bot users.
	let mut client = Client::new(token, Handler).expect("Err creating client");

	// Finally, start a single shard, and start listening to events.
	// Shards will automatically attempt to reconnect, and will perform
	// exponential backoff until it reconnects.
	if let Err(why) = client.start() {
		error!("Client error: {:?}", why);
	}
}
