use log::{error, info};
use rocket::{http::RawStr, State};
use serenity::{client::Context, model::id::ChannelId};

#[get("/")]
fn index() -> &'static str {
  "hello"
}

#[put("/message/<channelid>")]
async fn auth(channelid: &str, ctx: State<'_, Context>) -> String {
  let http = ctx.http.clone();
  ChannelId(555206410619584519)
    .say(http, "Salut!")
    .await
    .unwrap();
  format!("sending: {}", channelid)
}

pub async fn run(ctx: Context) {
  rocket::ignite()
    .manage(ctx)
    .mount("/", routes![index])
    .mount("/auth", routes![auth])
    .launch()
    .await
    .unwrap();
}
