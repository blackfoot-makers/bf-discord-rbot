//! Handle the connection with discord and it's events.
use log::{error, info};
use rand;
use serenity::{
	model::{channel::Message, event::ResumedEvent, gateway::Ready, id::ChannelId},
	prelude::*,
};

use super::credentials::CREDENTIALS_FILE;
use features;

use super::commands::{
	ATTACKED, COMMANDS_LIST, CONTAIN_MSG_LIST, CONTAIN_REACTION_LIST, TAG_MSG_LIST,
};

/// Struct that old Traits Implementations to Handle the different events send by discord.
struct Handler;

fn allowed_channel(
	command_channel: Option<ChannelId>,
	message_channel: ChannelId,
	ctx: &Context,
) -> bool {
	return match command_channel {
		Some(ref chan) => {
			if chan != &message_channel {
				let _ = message_channel.say(
					&ctx.http,
					format!(
						"I am not allowed to issue this command in this channel ! Use {} instead.",
						chan.mention()
					),
				);
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
				let _ = message
					.channel_id
					.send_message(&ctx.http, |m| m.content(result));
				return true;
			}
		}
	}
	false
}

fn process_tag_msg(message_split: &Vec<&str>, message: &Message, ctx: &Context) -> bool {
	for (key, reaction) in TAG_MSG_LIST.iter() {
		if *key == message_split[0] {
			let _ = message.channel_id.say(&ctx.http, reaction);
			return true;
		}
	}
	false
}

fn process_contains(message: &Message, ctx: &Context) {
	for (key, text) in CONTAIN_MSG_LIST.iter() {
		if message.content.contains(key) {
			let _ = message.channel_id.say(&ctx.http, *text);
		}
	}

	for (key, reaction) in CONTAIN_REACTION_LIST.iter() {
		if message.content.contains(key) {
			let _ = message.react(ctx, *reaction).unwrap();
		}
	}
}

fn split_args(input: &String) -> Vec<&str> {
	let mut count = 0;
	let message_split_quote: Vec<&str> = input.split('"').collect();
	let mut result: Vec<&str> = Vec::new();
	for msg in message_split_quote {
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
const KEYS: [&str; 8] = [
	"🔑", "🗝", "🔏", "🔐", "🔒", "🔓", "🖱", "👓",
];

/// Anoying other channels
fn annoy_channel(ctx: &Context, message: &Message) {
	let herdingchatte = ChannelId(570275817804791809);
	let cybergod = ChannelId(588666452849065994);
	let testbot = ChannelId(555206410619584519);

	if message.channel_id == herdingchatte {
		let random = rand::random::<usize>() % CATS.len();
		message.react(ctx, CATS[random]).unwrap();
	}
	if message.channel_id == cybergod {
		let random = rand::random::<usize>() % KEYS.len();
		message.react(ctx, KEYS[random]).unwrap();
	}
	if message.channel_id == testbot {
		let random = rand::random::<usize>() % KEYS.len();
		message.react(ctx, KEYS[random]).unwrap();
	}
}

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

const ANNOYING: [&str; 6] = [
	"Ah oui mais y'a JPO",
	"Vous pourriez faire ça vous meme s'il vous plaît ? Je suis occupé",
	"Avant, Faut laver les vitres les gars",
	"Ah mais vous faites quoi ?",
	"Non mais tu as vu le jeu qui est sorti ?",
	"Je bosse sur un projet super innovant en ce moment, j'ai pas le temps",
];

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
			println!(
				"ATtacked? {}=={}",
				message.author.mention(),
				*ATTACKED.read()
			);
			if message.author.mention() == *ATTACKED.read() {
				let random = rand::random::<usize>() % 6;
				let _ = message.channel_id.say(&ctx.http, ANNOYING[random]).unwrap();
				return;
			}

			let line = message.content.clone();
			let author: &str = &message.author.tag();
			let mut message_split = split_args(&line);

			// Check if there is only the tag : "@bot"
			if message_split.len() == 1 {
				let _ = message.channel_id.say(&ctx.http, "What do you need ?");
				return;
			}

			// Removing tag
			message_split.remove(0);
			message_split.push(author);

			// will go through commands.rs definitions to try and execute the request
			if !process_tag_msg(&message_split, &message, &ctx)
				&& !process_command(&message_split, &message, &ctx)
			{
				let _ = message
					.channel_id
					.say(&ctx.http, "How about a proper request ?");
			}
		} else {
			process_contains(&message, &ctx);
		}
	}

	fn ready(&self, ctx: Context, ready: Ready) {
		info!("{} is connected!", ready.user.name);
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
