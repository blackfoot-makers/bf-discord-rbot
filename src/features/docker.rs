use super::gitcommands::CHANNEL_BOTTEST;
use bollard::container::{
    Config, CreateContainerOptions, HostConfig, LogOutput, LogsOptions, StartContainerOptions,
};
use bollard::image::BuildImageOptions;
use bollard::{Docker, DockerChain};
use failure::Error;
use futures::Async;
use futures::{Future, Stream};
use serde::ser::Serialize;
use serenity::http;
use serenity::model::id::ChannelId;
use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::thread;
use tokio::runtime::Runtime;

type Poll<T, E> = Result<Async<T>, E>;
pub struct SayFuture {
    chan: ChannelId,
    content: String,
    http: Arc<http::raw::Http>,
    docker: DockerChain,
}

impl Future for SayFuture {
    type Item = (DockerChain, ());
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.chan.say(self.http.clone(), &self.content).unwrap();
        Ok(Async::Ready((self.docker.clone(), ())))
    }
}

pub fn deploy_from_gitcommand(payload: &serde_json::Value, http: Arc<http::raw::Http>) {
    let user: String = payload["repository"]["owner"]["name"]
        .as_str()
        .unwrap()
        .to_string();
    let repository: String = payload["repository"]["name"].as_str().unwrap().to_string();
    let branch: String = (&payload["ref"].as_str().unwrap()["refs/heads/".len()..]).to_string();

    thread::spawn(move || deploy(user, repository, branch, http));
}

pub fn deploy_from_reaction(
    user: String,
    repository: String,
    branch: String,
    http: Arc<http::raw::Http>,
) {
    thread::spawn(move || deploy(user, repository, branch, http));
}

pub fn deploy_test(user: String, repository: String, branch: String, http: Arc<http::raw::Http>) {
    thread::spawn(move || deploy(user, repository, branch, http));
}

fn create_and_logs<T>(
    docker: DockerChain,
    name: &'static str,
    config: Config<T>,
    http: Arc<http::raw::Http>,
) -> impl Stream<Item = LogOutput, Error = Error>
where
    T: AsRef<str> + Eq + Hash + Serialize,
{
    docker
        .create_container(Some(CreateContainerOptions { name: name }), config)
        .and_then(move |(docker, _)| {
            // thread::spawn(move || CHANNEL_BOTTEST.say(http, "Build success container started"));
            docker.start_container(name, None::<StartContainerOptions<String>>)
        })
        .and_then(move |(docker, _)| SayFuture {
            chan: CHANNEL_BOTTEST,
            content: String::from("Pouet"),
            http,
            docker,
        })
        .and_then(move |(docker, _)| {
            docker.logs(
                name,
                Some(LogsOptions {
                    follow: true,
                    stdout: true,
                    stderr: false,
                    ..Default::default()
                }),
            )
        })
        .map(|(_, stream)| stream)
        .into_stream()
        .flatten()
}

fn deploy(user: String, repository: String, branch: String, http: Arc<http::raw::Http>) {
    CHANNEL_BOTTEST
        .say(
            &http,
            format!("Deploying preview -> {}/{}->{}", user, repository, branch),
        )
        .unwrap();

    let mut tarball = reqwest::get(&format!(
        "https://github.com/{}/{}/archive/{}.tar.gz",
        user, repository, branch
    ))
    .unwrap();

    let mut rt = Runtime::new().unwrap();
    let docker = Docker::connect_with_unix_defaults().unwrap();

    let image_name: String = format!("{}-{}", repository.to_lowercase(), branch.to_lowercase());

    let build_image_options = BuildImageOptions {
        dockerfile: format!("{}-{}/Dockerfile", repository, branch),
        t: image_name.clone(),
        ..Default::default()
    };

    let mut contents = Vec::new();
    tarball.copy_to(&mut contents).unwrap();

    let future = docker
        .build_image(build_image_options, None, Some(contents.into()))
        .map(|v| {
            println!("{:?}", v);
            v
        })
        .collect()
        .map_err(|e| {
            println!("{:?}", e);
            ()
        })
        .map(|_| ());

    rt.spawn(future);
    rt.shutdown_on_idle().wait().unwrap();

    let mut exposed_ports = HashMap::new();
    exposed_ports.insert("3000/tcp", HashMap::new());

    let host_config = HostConfig {
        publish_all_ports: Some(true),
        ..Default::default()
    };

    let static_name: &'static str = Box::leak(image_name.into_boxed_str());
    let config = Config {
        exposed_ports: Some(exposed_ports),
        image: Some(static_name),
        host_config: Some(host_config),
        ..Default::default()
    };

    let stream = create_and_logs(
        docker.chain(),
        "preview-my-app-container",
        config,
        http.clone(),
    );

    let future = stream
        .map_err(|e| eprintln!("{:?}", e))
        .for_each(|x| Ok(println!("{:?}", x)));

    let mut rt_container = Runtime::new().unwrap();

    rt_container.spawn(future);
    rt_container.shutdown_on_idle().wait().unwrap();
}

/// Previously in process.rs
// fn parse_gitcommand_reaction(ctx: Context, reaction: Reaction) {
// 	let channel = ChannelId(555206410619584519); //TODO : Channel register

// 	let emoji_name = match &reaction.emoji {
// 		ReactionType::Unicode(e) => e.clone(),
// 		ReactionType::Custom {
// 			animated: _,
// 			name,
// 			id: _,
// 		} => name.clone().unwrap(),
// 		_ => "".to_string(),
// 	};
// 	debug!("Reaction emoji: {}", emoji_name);
// 	if reaction.channel_id == channel {
// 		if emoji_name == "✅" {
// 			let message = reaction.message(&ctx.http).unwrap();
// 			if message.is_own(&ctx.cache) {
// 				let closing_tag = message.content.find("]").unwrap_or_default();
// 				if closing_tag > 0 {
// 					let params = &message.content[1..closing_tag];
// 					let params_split: Vec<&str> = params.split('/').collect();
// 					if params_split.len() == 3 {
// 						// features::docker::deploy_from_reaction(
// 						// 	params_split[0].to_string(),
// 						// 	params_split[1].to_string(),
// 						// 	params_split[2].to_string(),
// 						// 	ctx.http.clone(),
// 						// );
// 						// return;
// 					}
// 				}

// 				eprintln!(
// 					"Reaction/gitcommand: Invalid params parse : [{}]",
// 					message.content
// 				);
// 			}
// 		}
// 	}
// }
