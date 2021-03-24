use super::parse;
use actix_cors::Cors;
use actix_web::{http, middleware, web, App, HttpMessage, HttpRequest, HttpServer, Responder};
use std::task::{Context, Poll};
use std::{env, pin::Pin};

use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use futures::future::{ok, Ready};
use futures::Future;

use serenity::{model::id::ChannelId, CacheAndHttp};
use std::sync::Arc;

pub struct Auth;

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S> for Auth
where
  S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
  B: 'static,
{
  type Request = ServiceRequest;
  type Response = ServiceResponse<B>;
  type Error = Error;
  type InitError = ();
  type Transform = AuthMiddleware<S>;
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ok(AuthMiddleware {
      service,
      apikey: env::var("API_KEY").unwrap(),
    })
  }
}

pub struct AuthMiddleware<S> {
  service: S,
  apikey: String,
}

impl<S, B> Service for AuthMiddleware<S>
where
  S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
  B: 'static,
{
  type Request = ServiceRequest;
  type Response = ServiceResponse<B>;
  type Error = Error;
  type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

  fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    self.service.poll_ready(cx)
  }

  fn call(&mut self, req: ServiceRequest) -> Self::Future {
    let token = if let Some(cookie) = req.cookie("token") {
      String::from(cookie.value())
    } else {
      String::new()
    };
    dbg!(token != self.apikey, &token, &self.apikey);
    if req.method() != "OPTIONS" && token != self.apikey {
      return Box::pin(async { Err(actix_web::error::ErrorUnauthorized("Unauthorized")) });
    }

    let fut = self.service.call(req);
    Box::pin(async move {
      let res = fut.await?;
      Ok(res)
    })
  }
}

async fn greet(_: HttpRequest) -> impl Responder {
  String::from("hello")
}

async fn send_message(
  req: HttpRequest,
  cache_and_http: web::Data<Arc<CacheAndHttp>>,
) -> impl Responder {
  let chan_id = req.match_info().get("chan_id");
  let message = req.match_info().get("message");
  if message.is_none() || chan_id.is_none() {
    return String::from("Missing params");
  }

  match parse::discord_str_to_id(chan_id.unwrap(), Some(parse::DiscordIds::Channel)) {
    Ok((chan_id, _)) => {
      ChannelId(chan_id)
        .send_message(&cache_and_http.http, |m| m.content(message.unwrap()))
        .await
        .unwrap();
      // cache_and_http.
      String::from("Done")
    }
    Err(_) => String::from("Unable to convert id to a discordid"),
  }
}

#[actix_web::main]
pub async fn run(cache_and_http: Arc<CacheAndHttp>) -> std::io::Result<()> {
  HttpServer::new(move || {
    let _ = env::var("API_KEY").expect("No api token was found");
    let cors = Cors::default()
      .allowed_origin_fn(|origin, _req_head| origin.as_bytes().ends_with(b".blackfoot.dev"))
      .allowed_methods(vec!["GET", "POST"])
      .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
      .allowed_header(http::header::CONTENT_TYPE)
      .max_age(3600);

    App::new()
      .wrap(cors)
      .wrap(middleware::Logger::new("\"%r\" %s %b %Dms"))
      .data(cache_and_http.clone())
      .route("/", web::get().to(greet))
      .service(
        web::scope("/auth")
          .wrap(Auth)
          .route("/send/{chan_id}/{message}", web::get().to(send_message)),
      )
  })
  .bind(("0.0.0.0", 8080))?
  .run()
  .await
}
