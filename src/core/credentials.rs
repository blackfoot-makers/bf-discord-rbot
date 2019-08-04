/// Store the credential for the mail account, and discord token.
use core::files;
use std::sync::Arc;

lazy_static! {
  pub static ref CREDENTIALS_FILE: Arc<files::FileReader<Credentials>> = Arc::new(files::build(
    String::from("credentials.json"),
    Credentials::new()
  ));
}

#[derive(Serialize, Deserialize)]
pub struct Credentials {
  pub email: String,
  pub password: String,
  pub domain: String,
  pub token: String,
}

impl Credentials {
  pub fn new() -> Credentials {
    Credentials {
      email: String::new(),
      password: String::new(),
      domain: String::new(),
      token: String::new(),
    }
  }
}
