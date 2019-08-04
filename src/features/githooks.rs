use hyper::rt::{run, Future};
use hyper::{Error, Server};
use log::info;
use rifling::{Constructor, Delivery, Hook};
use serenity::{http, model::id::ChannelId};
use std::sync::Arc;

const CHANNEL_BOTTEST: ChannelId = ChannelId(555206410619584519);

pub fn init(http: Arc<http::raw::Http>) {
  let mut cons = Constructor::new();
  let push_hook = Hook::new(
    "*",
    Some(String::from("secret")),
    move |delivery: &Delivery| {
      if delivery.event.clone().unwrap_or_default() == "push" {
        let payload = delivery.payload.clone().unwrap();
        let url = payload.get("head_commit").unwrap().get("url").unwrap();
        let _ = CHANNEL_BOTTEST.say(&http, format!("New commit {}", &url));
        info!("Received new commit {}", url)
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
  info!("Server running: {}", addr);
  run(server);
}
