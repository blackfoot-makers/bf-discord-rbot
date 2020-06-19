use crate::core::process::CACHE;
use serenity::model::id::GuildId;

pub fn guild_chanels(gid: GuildId) {
    let cache_lock = CACHE.read();
    let cache = cache_lock.read();
    match cache.guild(gid) {
        Some(guild) => {
            let channels = &guild.read().channels;
            println!("Channels {}:", channels.len());

            channels.iter().for_each(|item| {
                let channel = item.1.read();
                println!("{}>{}", channel.name(), channel.position);
            });
        }
        None => println!("Guild not found"),
    }
}
