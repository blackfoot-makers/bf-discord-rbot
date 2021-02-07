A discord bot written in Rust

This bot is personal and isn't meant to be used by others

The documentation for this project is located at [doc/rbot-discord](doc/rbot_discord/index.html)

To run this bot, just fill a new .env file at the root directory of the project with your information
Run the diesel migrations and use cargo run.

# Starting the project

## .env

```bash
token=<THE_DISCORD_BOT_TOKEN>
DATABASE_URL=postgres://<user>:<password>@localhost/discordbot
```

## [Diesel](https://diesel.rs/)

Install the diesel-cli with: `cargo install diesel_cli`
and run the migrations: `diesel migration run`

## Run

`cargo run`

# Deployement

Build the docker image and start it as a service

```bash
docker build . -t greefine/discord-rbot
docker service create --network host --name discordbot greefine/discord-rbot
```

> Note: don't forget the .env before building !
