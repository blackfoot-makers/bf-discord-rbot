use crate::core::parse;
use crate::database::{self, Message};
use parse::DiscordIds;
use rocket::http::{Method, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::json::Json;
use rocket::State;
use rocket_cors::{AllowedOrigins, CorsOptions};
use serenity::{client::Context, model::id::ChannelId};
use std::env;

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
      key == &*format!("Bearer {}", *APIKEY)
    }

    match req.headers().get_one("Authorization") {
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

#[post("/message/<channelid>", data = "<message>")]
async fn send_message(
  channelid: &str,
  message: String,
  _apikey: ApiKey<'_>,
  ctx: &State<Context>,
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

#[get("/channel/<channelid>")]
async fn get_channel_message(
  channelid: &str,
  _apikey: ApiKey<'_>,
  _ctx: &State<Context>,
) -> Result<Json<Vec<Message>>, String> {
  let discordid = parse::discord_str_to_id(channelid, Some(DiscordIds::Channel));
  match discordid {
    Ok((id, _)) => {
      let message_from_channel;
      {
        let db_instance = database::INSTANCE.write().unwrap();
        message_from_channel = db_instance
          .messages
          .clone()
          .into_iter()
          .filter(|message| message.channel == id as i64)
          .collect();
      }
      Ok(Json(message_from_channel))
    }
    Err(_) => Err(format!("Unable to parse channelid: {}", channelid)),
  }
}

pub async fn run(ctx: Context) {
  const ADDRESS: &str = "0.0.0.0";
  const PORT: u32 = 8080;
  let figment = rocket::Config::figment()
    .merge(("port", PORT))
    .merge(("address", ADDRESS));
  let cors = CorsOptions::default()
    .allowed_origins(AllowedOrigins::all())
    .allowed_methods(
      vec![Method::Get, Method::Post, Method::Patch]
        .into_iter()
        .map(From::from)
        .collect(),
    )
    .allow_credentials(true);

  rocket::custom(figment)
    .manage(ctx)
    .mount("/", routes![index])
    .mount("/auth", routes![send_message, get_channel_message])
    .attach(cors.to_cors().unwrap())
    .launch()
    .await
    .unwrap();
}
