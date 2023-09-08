lazy_static! {
  pub static ref CODEFLOW_SUPERVISOR_URL: String = std::env::var("CODEFLOW_SUPERVISOR_URL")
    .expect("missing CODEFLOW_SUPERVISOR_URL env variables");
  pub static ref SUPERVISOR_API_KEY: String =
    std::env::var("SUPERVISOR_API_KEY").expect("missing SUPERVISOR_API_KEY env variables");
}

impl CODEFLOW_SUPERVISOR_URL {
  pub fn format_url(&self, path: String) -> String {
    format!("{}/{}", self.to_string(), path)
  }
}
