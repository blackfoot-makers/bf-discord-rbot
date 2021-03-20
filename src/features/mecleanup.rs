use serenity::{
  client::Context,
  model::channel::{Reaction, ReactionType},
};

pub async fn check_mecleanup(ctx: &Context, reaction: &Reaction) {
  let emoji_name = match &reaction.emoji {
    ReactionType::Unicode(unicode) => Some(&*unicode),
    _ => None,
  };
  if let Some(unicode) = emoji_name {
    if ["🧹"].contains(&&**unicode) {
      reaction
        .message(&ctx.http)
        .await
        .unwrap()
        .delete(&ctx.http)
        .await
        .unwrap();
    }
  }
}
