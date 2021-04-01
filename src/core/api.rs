use crate::core::parse;
use std::env;

use parse::DiscordIds;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::State;
use rocket::{http::Status, yansi::Paint};
use serenity::{client::Context, model::id::ChannelId};

struct ApiKey<'r>(&'r str);

#[derive(Debug)]
enum ApiKeyError {
  Missing,
  Invalid,
}

lazy_static! {
  static ref APIKEY: String = env::var("API_KEY").expect("API_KEY WAS NOT FOUND");
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey<'r> {
  type Error = ApiKeyError;

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    fn is_valid(key: &str) -> bool {
      key == *APIKEY
    }

    match req.headers().get_one("api-key") {
      None => Outcome::Failure((Status::Unauthorized, ApiKeyError::Missing)),
      Some(key) if is_valid(key) => Outcome::Success(ApiKey(key)),
      Some(_) => Outcome::Failure((Status::Unauthorized, ApiKeyError::Invalid)),
    }
  }
}

#[get("/")]
fn index() -> &'static str {
  "hello"
}

#[put("/message/<channelid>", data = "<message>")]
async fn auth(
  channelid: &str,
  message: String,
  _apikey: ApiKey<'_>,
  ctx: State<'_, Context>,
) -> String {
  let discordid = parse::discord_str_to_id(channelid, Some(DiscordIds::Channel));
  match discordid {
    Ok((id, _)) => {
      ChannelId(id).say(&ctx.http, message).await.unwrap();
      String::from("done")
    }
    Err(_) => {
      format!("Unable to parse channelid: {}", channelid)
    }
  }
}

pub async fn run(ctx: Context) {
  const ADDRESS: &str = "0.0.0.0";
  const PORT: u32 = 8080;
  let figment = rocket::Config::figment()
    .merge(("port", 8080))
    .merge(("address", ADDRESS));

  let full_addr = format!("{}:{}", ADDRESS, PORT);
  info!(
    "{}{} {}",
    Paint::masked("🚀 "),
    Paint::default("Rocket has launched from").bold(),
    Paint::default(&full_addr).bold().underline()
  );

  rocket::custom(figment)
    .manage(ctx)
    .mount("/", routes![index])
    .mount("/auth", routes![auth])
    .launch()
    .await
    .unwrap();
}
