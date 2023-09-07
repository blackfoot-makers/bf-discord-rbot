use serenity::{
  model::prelude::{MessageId, Reaction},
  prelude::Context,
};
use std::{collections::HashMap, sync::Arc};
// We rename it to make it more clear because we use a tokio RwLock and not the std one.
use tokio::sync::RwLock as TokioRwLock;

use crate::{
  constants::{common::CODEFLOW_SUPERVISOR_URL, roles::ROLE_AUTHORIZED_TO_DEPLOY},
  core::parse,
};

pub struct DeploymentReactionsData {
  /// Unique ID used to dissociate deployments
  pub short_sha: String,
}

pub enum ValidationEmoji {
  Approve, // ✅
  Reject,  // ❌
}

impl ValidationEmoji {
  fn as_str(&self) -> &'static str {
    match self {
      Self::Approve => "approve",
      Self::Reject => "reject",
    }
  }
}

impl DeploymentReactionsData {
  /// If the function match a deployment_id it will deploy/reject the project by calling bf-codeflow-supervisor
  pub async fn validate(ctx: &Context, reaction: &Reaction, validation: ValidationEmoji) -> bool {
    let mut handler = REACTION_COLLECTORS.write().await;

    if let Some(reaction_collector) = handler.get(&reaction.message_id) {
      let user_id = reaction.user_id.unwrap();
      let validation_str = validation.as_str();
      let guild = parse::get_guild(reaction.channel_id, &ctx, None)
        .await
        .unwrap();

      log::info!(
        "{} deployment {}",
        validation_str,
        reaction_collector.short_sha
      );

      if let Ok(member) = guild.member(&ctx.http, user_id).await {
        if member
          .roles
          .iter()
          .any(|role| ROLE_AUTHORIZED_TO_DEPLOY.contains(&role.0))
        {
          reqwest::Client::new()
            .post(CODEFLOW_SUPERVISOR_URL.format_url(format!(
              "v1/two-factor-deployment/{}/{}",
              reaction_collector.short_sha, validation_str
            )))
            .send()
            .await
            .expect("Failed to send a request to bf-codeflow-supervisor");
          handler.remove_entry(&reaction.message_id);
        } else {
          log::info!("Deployment denied for user with id: {}", user_id);
        }
      }
      return true;
    }
    false
  }
}

lazy_static! {
  pub static ref REACTION_COLLECTORS: Arc<TokioRwLock<HashMap<MessageId, DeploymentReactionsData>>> =
    Arc::new(TokioRwLock::new(HashMap::new()));
}
