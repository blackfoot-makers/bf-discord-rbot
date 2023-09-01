use serenity::model::prelude::{MessageId, ReactionType};
use std::{collections::HashMap, sync::Arc};
// We rename it to make it more clear because we use a tokio RwLock and not the std one.
use tokio::sync::RwLock as TokioRwLock;

pub struct DeploymentReactions {
  pub accept: ReactionType,
  pub reject: ReactionType,
}

lazy_static! {
  pub static ref REACTION_COLLECTORS: Arc<TokioRwLock<HashMap<MessageId, DeploymentReactions>>> =
    Arc::new(TokioRwLock::new(HashMap::new()));
}
