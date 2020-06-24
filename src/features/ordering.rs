use crate::core::process::{CACHE, HTTP_STATIC};
use serenity::{
    model::{
        channel::ChannelType,
        id::{ChannelId, GuildId},
    },
    prelude::*,
};

pub fn ordering_channel_command(args: &[&str], context: &Context) -> String {
    let gid = GuildId(args[1].parse::<u64>().unwrap());
    return match guild_chanels_ordering(gid) {
        Ok((ordering, func)) => {
            crate::core::commands::validate_command(context, func);
            ordering
        }
        Err(why) => String::from(why),
    };
}

pub fn move_channels(chanid: u64, position: u64) {
    let cache_lock = CACHE.write().clone();

    let cache = cache_lock.read();
    match cache.guild_channel(ChannelId(chanid)) {
        Some(channel) => {
            let mut channel = channel.write();
            println!("{}>{}", channel.name(), channel.position);
            let http = HTTP_STATIC.write().clone().unwrap();
            if let Err(why) = channel.edit(&http, |chan| chan.position(position)) {
                println!("Unable to edit channel {}:\n{}", channel.name, why);
            }
        }
        None => println!("Channel {} not found", chanid),
    }
}

pub fn guild_chanels_ordering(
    gid: GuildId,
) -> Result<(String, Box<dyn FnOnce() + Send + Sync>), &'static str> {
    let cache_lock = CACHE.write().clone();
    let cache = cache_lock.read();
    match cache.guild(gid) {
        Some(guild) => {
            let channels = &mut guild.write().channels;
            println!("Channels {}:", channels.len());
            let mut vec_text: Vec<_> = channels
                .iter()
                .filter(|(_, chan)| chan.read().kind == ChannelType::Text)
                .collect();
            vec_text.sort_by(|chan, chan2| chan.1.read().name.cmp(&chan2.1.read().name));
            for (index, (_, chan)) in vec_text.iter().enumerate() {
                let mut channel = chan.write();
                println!("{}>{}", channel.name(), channel.position);
                if channel.position != index as i64 {
                    let http = HTTP_STATIC.write().clone().unwrap();
                    if let Err(why) = channel.edit(&http, |chan| chan.position(index as u64)) {
                        println!("Unable to edit channel {}:\n{}", channel.name, why);
                    }
                }
            }
            let mut vec_voice: Vec<_> = channels
                .iter()
                .filter(|(_, chan)| chan.read().kind == ChannelType::Voice)
                .collect();
            vec_voice.sort_by(|chan, chan2| chan.1.read().name.cmp(&chan2.1.read().name));
            let func = || {
                for (index, (_, chan)) in vec_voice.iter().enumerate() {
                    let mut channel = chan.write();
                    println!("{} > {}", channel.name(), channel.position);
                    if channel.position != index as i64 {
                        let http = HTTP_STATIC.write().clone().unwrap();
                        if let Err(why) = channel.edit(&http, |chan| chan.position(index as u64)) {
                            println!("Unable to edit channel {}:\n{}", channel.name, why);
                        }
                    }
                }
            };
            Ok((String::from("test"), Box::new(func)))
        }
        None => Err("Guild not found"),
    }
}
