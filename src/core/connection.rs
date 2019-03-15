//! Handle the connection with discord and it's events.

use features;
// discontinued
// use features::mail;
// use features::monitor;
use notify::Event;
use rand;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use serenity::CACHE;
use std::collections::HashMap;
use std::process;
use CREDENTIALS_FILE;

/// Struct that old Traits Implementations to Handle the different events send by discord.
struct Handler;

/// `ChannelId` for main channel
// pub static CHANNEL_MAIN: ChannelId = ChannelId(464783335559004162);
/// `ChannelId` for mail channel
// pub static CHANNEL_MAILS: ChannelId = ChannelId(469143963585216512);
// pub static CHANNEL_BOT: ChannelId = ChannelId(454614003650527232);

struct Command {
	exec: fn(&Vec<&str>) -> String,
	argument_min: usize,
	argument_max: usize,
	channel: ChannelId,
	usage: String,
}

trait IsEmpty {
	fn is_empty(&self) -> bool;
}

impl IsEmpty for ChannelId {
	fn is_empty(&self) -> bool {
		self.0 == 0
	}
}

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

lazy_static! {
	static ref ATTACKED: RwLock<String> = RwLock::new(String::new());
	static ref TAG_MSG_LIST: HashMap<&'static str, &'static str> = hashmap![
		"ping" => "pong",
		"bad" => "😢",
		"Bonjour !" => "Bonsoir !",
		"Bonjour" => "Bonsoir !",
		"🖕" => "🖕"
	];
	static ref CONTAIN_MSG_LIST: HashMap<&'static str, &'static str> = hashmap![
		"keke" => "https://media.giphy.com/media/26ufju9mygxXmfjos/giphy.gif",
		"kéké" => "https://media.giphy.com/media/26ufju9mygxXmfjos/giphy.gif",
		"bad bot" => "😎"
	];
	static ref CONTAIN_REACTION_LIST: HashMap<&'static str, &'static str> = hashmap![
		"👊" => "👊",
		"licorne" => "🦄",
		"leslie" => "🦄",
		"max" => "🍌",
		"retard" => "⌚",
		"pm" => "🐱"
	];
	static ref COMMANDS_LIST: HashMap<&'static str, Command> = hashmap![
		"quit" =>
		Command {
			exec: quit,
			argument_min: 0,
			argument_max: 0,
			channel: ChannelId(0),
			usage: String::from("Usage : @BOT quit"),
		},
		// "health" =>
		// Command {
		// 	exec: monitor::display_codes,
		// 	argument_min: 0,
		// 	argument_max: 0,
		// 	channel: ChannelId(0),
		// 	usage: String::from("Usage : @BOT health"),
		// },
		// "mail" =>
		// Command {
		// 	exec: mail::content,
		// 	argument_min: 0,
		// 	argument_max: 0,
		// 	channel: CHANNEL_MAILS,
		// 	usage: String::from("Usage : @BOT mail"),
		// },
		// "unassigned" =>
		// Command {
		// 	exec: mail::display_unassigned,
		// 	argument_min: 0,
		// 	argument_max: 0,
		// 	channel: CHANNEL_MAILS,
		// 	usage: String::from("Usage : @BOT unassigned"),
		// },
		// "assigned" =>
		// Command {
		// 	exec: mail::display_assigned,
		// 	argument_min: 0,
		// 	argument_max: 1,
		// 	channel: CHANNEL_MAILS,
		// 	usage: String::from("Usage : @BOT assigned [@TAG]"),
		// },
		// "resolved" =>
		// Command {
		// 	exec: mail::display_resolved,
		// 	argument_min: 0,
		// 	argument_max: 1,
		// 	channel: CHANNEL_MAILS,
		// 	usage: String::from("Usage : @BOT resolved [@TAG]"),
		// },
		// "assign" =>
		// Command {
		// 	exec: mail::assign,
		// 	argument_min: 1,
		// 	argument_max: 2,
		// 	channel: CHANNEL_MAILS,
		// 	usage: String::from("Usage : @BOT assign MAIL_ID [@TAG]"),
		// },
		// "resolve" =>
		// Command {
		// 	exec: mail::resolve,
		// 	argument_min: 1,
		// 	argument_max: 2,
		// 	channel: CHANNEL_MAILS,
		// 	usage: String::from("Usage : @BOT resolve MAIL_ID [@TAG]"),
		// },
		"reminder" =>
		Command {
			exec: Event::add_reminder,
			argument_min: 4,
			argument_max: 5,
			channel: ChannelId(0),
			usage: String::from("Usage : @BOT reminder NAME DATE(MONTH-DAY:HOURS:MINUTES) MESSAGE CHANNEL REPEAT(delay in minutes)"),
		},
		"countdown" =>
		Command {
			exec: Event::add_countdown,
			argument_min: 6,
			argument_max: 6,
			channel: ChannelId(0),
			usage: String::from("Usage : @BOT countdown NAME START_DATE(MONTH-DAY:HOURS) END_DATE(MONTH-DAY:HOURS) DELAY_OF_REPETITION(minutes) MESSAGE CHANNEL"),
		},
		"import" =>
		Command {
			exec: features::slackimport::import,
			argument_min: 0,
			argument_max: 0,
			channel: ChannelId(0),
			usage: String::from("Usage : @BOT import"),
		}
	];
}

fn process_command(message_split: &Vec<&str>, message: &Message) -> bool {
	for (key, command) in COMMANDS_LIST.iter() {
		if *key == message_split[0] {
			if !command.channel.is_empty() && command.channel != message.channel_id {
				let _ = message.channel_id.say(format!(
					"I am not allowed issue this command in this channel ! Use {} instead",
					command.channel.mention()
				));
				return true;
			}
			// We remove default arguments: author and command name from the total
			let arguments = message_split.len() - 2;
			let result = if arguments >= command.argument_min && arguments <= command.argument_max {
				(command.exec)(message_split)
			} else {
				command.usage.clone()
			};
			let _ = message.channel_id.send_message(|m| m.content(result));
			return true;
		}
	}
	false
}

fn process_tag_msg(message_split: &Vec<&str>, message: &Message) -> bool {
	for (key, reaction) in TAG_MSG_LIST.iter() {
		if *key == message_split[0] {
			let _ = message.channel_id.say(reaction);
			return true;
		}
	}
	false
}

fn process_contains(message: &Message) {
	for (key, text) in CONTAIN_MSG_LIST.iter() {
		if message.content.contains(key) {
			let _ = message.channel_id.say(*text);
		}
	}

	for (key, reaction) in CONTAIN_REACTION_LIST.iter() {
		if message.content.contains(key) {
			let _ = message.react(*reaction).unwrap();
		}
	}
}

fn quit(_args: &Vec<&str>) -> String {
	println!("Quitting.");
	process::exit(0x0100);
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

impl EventHandler for Handler {
	/// Set a handler for the `message` event - so that whenever a new message
	/// is received - the closure (or function) passed will be called.
	///
	/// Event handlers are dispatched through a threadpool, and so multiple
	/// events can be dispatched simultaneously.
	fn message(&self, _context: Context, message: Message) {
		println!("{} says: {}", message.author.name, message.content);
		if message.is_own() || message.content.is_empty() {
			return;
		};
		if message.author.mention() == *ATTACKED.read() {
			const ANNOYING: [&str; 11] = [
				"🐧", "💩", "🍌", "💣", "👾", "🐔", "📛", "🔥", "‼", "⚡", "⚠",
			];
			let random1 = rand::random::<usize>() % 6;
			let random2 = rand::random::<usize>() % 6;
			message.react(ANNOYING[random1]).unwrap();
			message.react(ANNOYING[random2]).unwrap();
		}
		//Check if i am tagged in the message else do the reactions
		// check for @me first so it's considered a command
		if message
			.content
			.starts_with(&*format!("<@{}>", CACHE.read().user.id.as_u64()))
		{
			if message.author.mention() == *ATTACKED.read() {
				const ANNOYING: [&str; 6] = [
					"Ah oui mais y'a JPO",
					"Vous pourriez faire ça vous meme s'il vous plaît ? Je suis occupé",
					"Avant, Faut laver les vitres les gars",
					"Ah mais vous faites quoi ?",
					"Non mais tu as vu le jeu qui est sorti ?",
					"Je bosse sur un projet super innovant en ce moment, j'ai pas le temps",
				];
				let random = rand::random::<usize>() % 6;
				let _ = message.channel_id.say(ANNOYING[random]).unwrap();
				return;
			}

			let line = message.content.clone();
			let author: &str = &message.author.tag();

			let mut message_split = split_args(&line);
			if message_split.len() == 1 {
				let _ = message.channel_id.say("What do you need ?");
				return;
			}
			// Removing tag
			message_split.remove(0);
			message_split.push(author);
			if message_split[0] == "attack" && message_split.len() == 3 {
				ATTACKED.write().clear();
				ATTACKED.write().push_str(message_split[1]);
				let _ = message
					.channel_id
					.say(format!("Prepare yourself {} !", message_split[1]));
				return;
			}

			if !process_tag_msg(&message_split, &message) && !process_command(&message_split, &message) {
				let _ = message.channel_id.say("How about a proper request ?");
			}
		} else {
			process_contains(&message);
		}
	}

	fn ready(&self, _: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);
		features::run();
	}
}

/// Get the discord token from `CREDENTIALS_FILE` and run the client.
pub fn bot_connect() {
	println!("Bot Connecting");

	let token = &CREDENTIALS_FILE.stored.token;

	// Create a new instance of the Client, logging in as a bot. This will
	// automatically prepend your bot token with "Bot ", which is a requirement
	// by Discord for bot users.
	let mut client = Client::new(token, Handler).expect("Err creating client");

	// Finally, start a single shard, and start listening to events.
	//
	// Shards will automatically attempt to reconnect, and will perform
	// exponential backoff until it reconnects.
	if let Err(why) = client.start() {
		println!("Client error: {:?}", why);
	}
}
