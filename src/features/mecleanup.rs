use serenity::{client::Context, model::channel::Reaction};

pub async fn check_mecleanup(ctx: &Context, reaction: &Reaction) {
  reaction
    .message(&ctx.http)
    .await
    .unwrap()
    .delete(&ctx.http)
    .await
    .unwrap();
}
