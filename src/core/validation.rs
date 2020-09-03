use serenity::{
  model::channel::{Message, Reaction, ReactionType},
  prelude::*,
};
use std::collections::HashMap;

lazy_static! {
  pub static ref TO_VALIDATE: RwLock<HashMap<u64, Box<dyn FnOnce() + Send + Sync>>> =
    RwLock::new(HashMap::new());
}

fn message_link(reaction: &Reaction) -> String {
  format!(
    "https://discordapp.com/channels/{}/{}/{}",
    reaction.guild_id.unwrap(),
    reaction.channel_id.0,
    reaction.message_id.0
  )
}

pub fn check_validation(ctx: &Context, reaction: &Reaction) {
  let emoji_name = match &reaction.emoji {
    ReactionType::Unicode(e) => e.clone(),
    ReactionType::Custom {
      animated: _,
      name,
      id: _,
    } => name.clone().unwrap(),
    _ => "".to_string(),
  };
  if ["✅", "❌"].contains(&&*emoji_name) {
    let mut to_validate = TO_VALIDATE.write();
    let callback = to_validate.remove(&reaction.message_id.0);
    if let Some(callback) = callback {
      let mut message = reaction.message(&ctx.http).unwrap();
      if emoji_name == "✅" {
        callback();
        message
          .channel_id
          .say(
            &ctx.http,
            format!(
              "<@{}> applied {}",
              reaction.user_id,
              message_link(&reaction),
            ),
          )
          .unwrap();
      } else if emoji_name == "❌" {
        let prevtext = message.content.clone();
        message
          .edit(&ctx.http, |message| {
            message.content(format!("~~{}~~", prevtext))
          })
          .unwrap();
      }
    }
  }
}

pub fn validate_command(
  responsse: &str,
  message: &Message,
  context: &Context,
  callback: Box<dyn FnOnce() + Send + Sync>,
) {
  let mut to_validate = TO_VALIDATE.write();
  let message = message.reply(&context.http, responsse).unwrap();
  message.react(&context.http, "✅").unwrap();
  message.react(&context.http, "❌").unwrap();
  to_validate.insert(message.id.0, callback);
}
