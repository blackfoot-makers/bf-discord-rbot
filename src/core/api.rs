use crate::{
  constants::{common::TWO_FACTOR_DEPLOYMENT_CHANNEL, discordids::DEVOPS_CHANNEL},
  core::parse,
  database::{self, Message},
  features::deployment::{DeploymentReactionsData, REACTION_COLLECTORS},
};
use parse::DiscordIds;
use rocket::{
  http::{Method, Status},
  request::{FromRequest, Outcome, Request},
  serde::json::Json,
  State,
};
use rocket_cors::{AllowedOrigins, CorsOptions};
use serde_derive::Deserialize;
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

//
#[post("/deployment/<short_sha>")]
async fn two_factor_deployment(
  short_sha: &str,
  _apikey: ApiKey<'_>,
  ctx: &State<Context>,
) -> (Status, String) {
  let sent_msg: serenity::model::prelude::Message = TWO_FACTOR_DEPLOYMENT_CHANNEL
    .send_message(&ctx.http, |m| m.content("React with ✅ or ❌"))
    .await
    .unwrap();
  let accept = sent_msg.react(&ctx.http, '✅').await.unwrap();
  let reject = sent_msg.react(&ctx.http, '❌').await.unwrap();

  match REACTION_COLLECTORS.try_write() {
    Err(err) => {
      log::error!("Failed to write into REACTION_COLLECTORS: {} ", err);

      return (Status::InternalServerError, "error".to_string());
    }
    Ok(mut react_collect) => {
      react_collect.insert(
        sent_msg.id,
        DeploymentReactionsData {
          short_sha: short_sha.to_string(),
          accept: accept.emoji,
          reject: reject.emoji,
        },
      );
    }
  }
  (Status::Ok, ":ok:".to_string())
}

#[post("/message/<channelid>", data = "<message>")]
async fn send_message(
  channelid: &str,
  message: String,
  _apikey: ApiKey<'_>,
  ctx: &State<Context>,
) -> (Status, String) {
  if message.len() > 2000 {
    error!("Too Long Message ({})", message.len());
    return (
      Status::BadRequest,
      format!("Too Long Message ({})", message.len()),
    );
  }
  let discordid = parse::discord_str_to_id(channelid, Some(DiscordIds::Channel));
  match discordid {
    Ok((id, _)) => {
      ChannelId(id).say(&ctx.http, message).await.unwrap();
      (Status::Ok, ":ok:".to_string())
    }
    Err(_) => (
      Status::BadRequest,
      format!("Unable to parse channelid: {}", channelid),
    ),
  }
}

#[derive(Debug, Deserialize)]
pub struct GCPAlert {
  pub version: String,
  pub incident: Incident,
}

#[derive(Debug, Deserialize)]
pub struct Incident {
  pub incident_id: String,
  pub scoping_project_id: String,
  pub url: String,
  pub started_at: i64,
  pub ended_at: i64,
  pub state: String,
  pub summary: String,
  pub resource_type_display_name: String,
  pub resource_display_name: String,
}

#[post("/webhook/gcp_alert", format = "json", data = "<alert>")]
async fn webhook_from_gcp(alert: Json<GCPAlert>, ctx: &State<Context>) -> String {
  ChannelId(DEVOPS_CHANNEL)
    .say(
      &ctx.http,
      format!(
        "{}: {}\n{}\n```json\n{:#?}\n```",
        alert.0.incident.scoping_project_id,
        alert.0.incident.summary,
        alert.0.incident.url,
        alert.0
      ),
    )
    .await
    .unwrap();
  String::from("")
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
  let shutdown: rocket::config::Shutdown = rocket::config::Shutdown {
    ctrlc: false,
    ..Default::default()
  };
  let figment = rocket::Config::figment()
    .merge(("port", PORT))
    .merge(("address", ADDRESS))
    .merge(("shutdown", shutdown));
  let cors = CorsOptions::default()
    .allowed_origins(AllowedOrigins::all())
    .allowed_methods(
      vec![Method::Get, Method::Post, Method::Patch]
        .into_iter()
        .map(From::from)
        .collect(),
    )
    .allow_credentials(true);

  let _ = rocket::custom(figment)
    .manage(ctx)
    .mount("/", routes![index])
    .mount(
      "/auth",
      routes![
        send_message,
        get_channel_message,
        webhook_from_gcp,
        two_factor_deployment
      ],
    )
    .attach(cors.to_cors().unwrap())
    .launch()
    .await
    .unwrap();
}
