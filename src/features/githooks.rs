use hyper::rt::{run, Future};
use hyper::{Error, Server};
use log::info;
use rifling::{Constructor, Delivery, Hook};
use serenity::{http, model::id::ChannelId};
use std::sync::Arc;
use std::thread;

pub const CHANNEL_BOTTEST: ChannelId = ChannelId(555206410619584519);

pub fn notify(payload: &serde_json::Value, http: Arc<http::raw::Http>) {
  let url = payload["head_commit"]["url"].as_str().unwrap();
  let user: String = payload["repository"]["owner"]["name"]
    .as_str()
    .unwrap()
    .to_string();
  let repository: String = payload["repository"]["name"].as_str().unwrap().to_string();
  let branch: String = (&payload["ref"].as_str().unwrap()["refs/heads/".len()..]).to_string();
  CHANNEL_BOTTEST
    .say(
      &http,
      format!("[{}/{}/{}]: new commit {}", user, repository, branch, url),
    )
    .unwrap();
}

pub fn init(http: Arc<http::raw::Http>) {
  let mut cons = Constructor::new();
  let push_hook = Hook::new(
    "*",
    Some(String::from("secret")),
    move |delivery: &Delivery| {
      if delivery.event.clone().unwrap_or_default() == "push" {
        let payload = delivery.payload.clone().unwrap();
        println!("Received payload for : {}", &payload["head_commit"]["url"]);
        let http_clone = http.clone();
        thread::spawn(move || notify(&payload, http_clone));
      } else {
        info!(
          "Received unmanaged event {}",
          delivery.event.clone().unwrap_or_default()
        );
      }
    },
  );

  cons.register(push_hook);
  let addr = "0.0.0.0:4567".parse().unwrap();
  let server = Server::bind(&addr)
    .serve(cons)
    .map_err(|e: Error| println!("Error: {:?}", e));
  println!("Server running: {}", addr);
  run(server);
}
