use std::time::SystemTime;

use crate::features::funny;
use crate::{constants, features::minecraft};
use chrono::{Datelike, Utc};
use procedural_macros::command;
use serenity::{
  client::Context,
  model::{
    application::{
      command::CommandOptionType,
      interaction::{Interaction, InteractionResponseType},
    },
    id::GuildId,
  },
};

use super::commands::{CallBackParams, CallbackReturn};

#[command]
pub async fn set(params: CallBackParams) -> CallbackReturn {
  GuildId(constants::discordids::GUILD_ID)
    .set_application_commands(&params.context.http, |commands| {
      commands
        .create_application_command(|command| {
          command
            .name("mom")
            .description("Witch user mom is currenly targeted")
        })
        .create_application_command(|command| {
          command
            .name("mom-change")
            .description("Change the current user mom targeted")
            .create_option(|o| {
              o.name("user")
                .description("Who is the new target")
                .kind(CommandOptionType::User)
                .required(true)
            })
        })
        .create_application_command(|command| {
          command
            .name("office-week")
            .description("Get the current week of the office")
        })
        .create_application_command(|command| {
          command
            .name("playing-mc")
            .description("Get the list of users connected to minecraft")
        })
    })
    .await
    .unwrap();

  Ok(Some(String::from(":ok:")))
}

pub async fn handle_event(interaction: Interaction, ctx: Context) {
  if let Interaction::ApplicationCommand(command) = interaction {
    match &*command.data.name {
      "mom" => {
        let mom_result = funny::which_mom_cmdless().await;
        command
          .create_interaction_response(&ctx.http, |res| {
            res
              .kind(InteractionResponseType::ChannelMessageWithSource)
              .interaction_response_data(|resdata| resdata.content(mom_result.unwrap()))
          })
          .await
          .unwrap()
      }
      "mom-change" => {
        let newuser = command
          .data
          .options
          .first()
          .unwrap()
          .value
          .as_ref()
          .unwrap()
          .as_str()
          .unwrap();
        let system_time_now = SystemTime::now();

        let mom_result =
          funny::mom_change_cmdless(&format!("<@{}>", newuser), system_time_now.into()).await;

        command
          .create_interaction_response(&ctx.http, |res| {
            res
              .kind(InteractionResponseType::ChannelMessageWithSource)
              .interaction_response_data(|resdata| resdata.content(mom_result.unwrap()))
          })
          .await
          .unwrap()
      }
      "office-week" => {
        let month = (Utc::now().iso_week().week() + 1) % 3 + 1;
        let result = format!("it's currently S0{month}");
        command
          .create_interaction_response(&ctx.http, |res| {
            res
              .kind(InteractionResponseType::ChannelMessageWithSource)
              .interaction_response_data(|resdata| resdata.content(result))
          })
          .await
          .unwrap()
      }
      "playing-mc" => {
        let players = minecraft::list_players();
        command
          .create_interaction_response(&ctx.http, |res| {
            res
              .kind(InteractionResponseType::ChannelMessageWithSource)
              .interaction_response_data(|resdata| resdata.content(players.join(" ")))
          })
          .await
          .unwrap()
      }
      _ => {}
    }
  }
}

#[test]
fn test_office_week() {
  let month = (Utc::now().iso_week().week() + 1) % 3 + 1;
  dbg!(month);
  //
}
