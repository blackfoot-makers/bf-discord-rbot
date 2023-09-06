use serenity::model::prelude::ChannelId;

pub const TWO_FACTOR_DEPLOYMENT_CHANNEL: ChannelId = ChannelId(745312984481398806);

lazy_static! {
  pub static ref CODEFLOW_SUPERVISOR_URL: String = std::env::var("CODEFLOW_SUPERVISOR_URL")
    .expect("missing CODEFLOW_SUPERVISOR_URL env variables");
}

impl CODEFLOW_SUPERVISOR_URL {
  pub fn format_url(&self, path: String) -> String {
    format!("{}/{}", self.to_string(), path)
  }
}
