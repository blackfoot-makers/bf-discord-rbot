use std::time::SystemTime;

use crate::constants;
use crate::features::funny;
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
    })
    .await
    .unwrap();

  Ok(Some(String::from(":ok:")))
}

pub async fn handle_event(interaction: Interaction, ctx: Context) {
  if let Interaction::ApplicationCommand(command) = interaction {
    if command.data.name == "mom" {
      let mom_result = funny::which_mom_cmdless().await;
      command
        .create_interaction_response(&ctx.http, |res| {
          res
            .kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|resdata| resdata.content(mom_result.unwrap()))
        })
        .await
        .unwrap()
    } else if command.data.name == "mom-change" {
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
  }
}
