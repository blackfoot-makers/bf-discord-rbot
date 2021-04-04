use futures::future::BoxFuture;
use procedural_macros::command;
use serenity::{
  model::channel::{Message, Reaction},
  prelude::*,
};
use std::collections::HashMap;

pub type ValidationCallback = Box<dyn FnOnce() -> BoxFuture<'static, ()> + Send + Sync>;
#[derive(Default)]
pub struct WaitingValidation {
  pub to_validate: HashMap<u64, ValidationCallback>,
}

impl TypeMapKey for WaitingValidation {
  type Value = WaitingValidation;
}

fn message_link(reaction: &Reaction) -> String {
  format!(
    "https://discordapp.com/channels/{}/{}/{}",
    reaction.guild_id.unwrap(),
    reaction.channel_id.0,
    reaction.message_id.0
  )
}

pub async fn check_validation(ctx: &Context, reaction: &Reaction, emoji: &str) {
  let data = &mut ctx.data.write().await;
  let waitingvalidation = data.get_mut::<WaitingValidation>().unwrap();

  let callback = waitingvalidation.to_validate.remove(&reaction.message_id.0);
  if let Some(callback) = callback {
    let mut message = reaction.message(&ctx.http).await.unwrap();
    if emoji == "✅" {
      callback().await;
      message
        .channel_id
        .say(
          &ctx.http,
          format!(
            "<@{}> applied {}",
            reaction.user_id.unwrap(),
            message_link(&reaction),
          ),
        )
        .await
        .unwrap();
    } else if emoji == "❌" {
      let prevtext = message.content.clone();
      message
        .edit(&ctx.http, |message| {
          message.content(format!("~~{}~~", prevtext))
        })
        .await
        .unwrap();
    }
  }
}

#[command]
pub async fn validate_command(
  responsse: &str,
  message: &Message,
  context: &Context,
  callback: ValidationCallback,
) -> BoxFuture<'fut, ()> {
  let data = &mut context.data.write().await;
  let waitingvalidation = data.get_mut::<WaitingValidation>().unwrap();

  let message = message.reply(&context.http, responsse).await.unwrap();
  message.react(&context.http, '✅').await.unwrap();
  message.react(&context.http, '❌').await.unwrap();
  waitingvalidation.to_validate.insert(message.id.0, callback);
}
