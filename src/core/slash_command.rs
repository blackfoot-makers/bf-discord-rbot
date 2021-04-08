use std::time::SystemTime;

use crate::constants;
use crate::features::funny;
use procedural_macros::command;
use serenity::{
  client::Context,
  model::{
    id::GuildId,
    interactions::{ApplicationCommandOptionType, Interaction, InteractionResponseType},
  },
};

use super::commands::{CallBackParams, CallbackReturn};

#[command]
pub async fn set(params: CallBackParams) -> CallbackReturn {
  let application_id: u64 = params.context.cache.current_user_id().await.0; // usually this will be the bot's UserId

  let _ = Interaction::create_guild_application_command(
    &params.context.http,
    GuildId(constants::discordids::GUILD_ID),
    application_id,
    |a| {
      a.name("mom")
        .description("Witch user mom is currenly targeted")
    },
  )
  .await
  .unwrap();
  let _ = Interaction::create_guild_application_command(
    &params.context.http,
    GuildId(constants::discordids::GUILD_ID),
    application_id,
    |a| {
      a.name("mom-change")
        .description("Change the current user mom targeted")
        .create_interaction_option(|o| {
          o.name("user")
            .description("Who is the new target")
            .kind(ApplicationCommandOptionType::User)
            .required(true)
        })
    },
  )
  .await
  .unwrap();

  Ok(Some(String::from(":ok:")))
}

pub async fn handle_event(interaction: Interaction, ctx: Context) {
  let interaction_data = if let Some(interaction_data) = &interaction.data {
    interaction_data
  } else {
    return;
  };

  if interaction_data.name == "mom" {
    let mom_result = funny::which_mom_cmdless().await;
    interaction
      .create_interaction_response(&ctx.http, |res| {
        res
          .kind(InteractionResponseType::ChannelMessageWithSource)
          .interaction_response_data(|resdata| resdata.content(mom_result.unwrap()))
      })
      .await
      .unwrap()
  } else if interaction_data.name == "mom-change" {
    let newuser = interaction_data
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
      funny::mom_change_cmdless(&*format!("<@{}>", newuser), system_time_now.into()).await;

    interaction
      .create_interaction_response(&ctx.http, |res| {
        res
          .kind(InteractionResponseType::ChannelMessageWithSource)
          .interaction_response_data(|resdata| resdata.content(mom_result.unwrap()))
      })
      .await
      .unwrap()
  }
}
