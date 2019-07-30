//! Import slack to discord, using exported files
//!
//! Prety dumb with no error management, simply visit every files and subdirectory from a path
//! Read and Parse the content from the files, and output them as formated messages to discord.

use core::files;
use serenity::model::channel::ChannelType;
use serenity::model::channel::GuildChannel;
use serenity::model::id::ChannelId;
use serenity::model::id::GuildId;
use serenity::prelude::Context;
use std::fs::{self, DirEntry};
use std::io;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
/// Minimal structure representation from a Slack json export file
struct Message {
    #[link_name = "type"]
    #[serde(default)]
    type_: String,
    #[serde(default)]
    user: String,
    #[serde(default)]
    text: String,
    #[serde(default)]
    ts: String,
}

/// Fixed guild to export slack files to
static GUILD: GuildId = GuildId(464779118857420811);

/// Visit every entry in a directory
/// Create the discord channel to send the message to from the folder name
/// Use `visit_files` as file reader.
fn visit_dirs(
    dir: &Path,
    cb: &Fn(&DirEntry, &GuildChannel),
    channel: GuildChannel,
    ctx: &Context
) -> io::Result<()> {
    if dir.is_dir() {
        let mut paths: Vec<_> = fs::read_dir(dir).unwrap().map(|r| r.unwrap()).collect();
        paths.sort_by_key(|dir| dir.path());
        for path in paths {
            let file = path.path();
            if file.is_dir() {
                let channel = GUILD.create_channel(ctx,
                    file.file_name().unwrap().to_str().unwrap(),
                    ChannelType::Text,
                    None,
                );
                visit_dirs(&file, cb, channel.unwrap(), ctx)?;
            } else {
                cb(&path, &channel);
            }
        }
    }
    Ok(())
}

/// Read and parse the files, output the found messages, to the discord associated channel
fn visit_files(entry: &DirEntry, channel: &GuildChannel) {
    println!("Working on : {:?}", entry.path());
    let path: String = entry.path().to_str().unwrap().to_string();
    let vec: Vec<Message> = Vec::new();
    let file = files::build(path.to_owned(), vec);
    for x in file.stored {
        if x.text != "" {
            let mut actual = 0;
            let mut len = x.text.len();
            while len != 0 {
                let mut c_number: usize = if len > 1500 { 1500 } else { len };
                channel
                    .say(&format!(
                        "**{}**: {}",
                        &x.user,
                        &x.text[actual..actual + c_number]
                    ))
                    .unwrap();
                len -= c_number;
                actual += c_number;
            }
        }
    }
}

#[allow(dead_code)]
/// Entry point for importing Slack to discord
pub fn import(_args: &Vec<&str>) -> String {
    let channel = GUILD.channels().unwrap()[&ChannelId(464779119809396757)].clone();
    visit_dirs(Path::new("./SlackExport"), &visit_files, channel).unwrap();
    String::from("Started importing Slack")
}
