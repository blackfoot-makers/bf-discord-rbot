#[cfg(not(feature = "production"))]
pub use development::*;
#[cfg(feature = "production")]
pub use production::*;
use serenity::model::prelude::ChannelId;

pub mod production {
  pub const GUILD_ID: u64 = 464779118857420811;
  pub const PROJECT_CATEGORY: u64 = 481747896539152384;
  pub const PROJECT_ANOUNCEMENT_CHANNEL: u64 = 747066293135605791;
  pub const USER_ROLE: u64 = 464781714179358730;
  pub const ARCHIVE_CATEGORY: u64 = 585403527564886027;
  pub const AITABLE_NOTIFY_CHAN: u64 = 501406998085238784;
  pub const DEVOPS_CHANNEL: u64 = 892745636489855046;
  pub const CDC_CRA_CHANNEL: u64 = 651436625909252129;
  pub const ANNOYED_CHAN_HERDINGCHATTE: u64 = 570275817804791809;
  pub const ANNOYED_CHAN_CYBERGOD: u64 = 588666452849065994;
  pub const ANNOYED_CHAN_TESTBOT: u64 = 555206410619584519;
  pub const DEPLOYMENT_CHAN: u64 = 826412321801764894; // todo: change to a real channel
}

pub mod development {
  // DEV CHAN == 555206410619584519
  pub const GUILD_ID: u64 = 339372728366923776;
  pub const PROJECT_ANOUNCEMENT_CHANNEL: u64 = 828569963156471808;
  pub const PROJECT_CATEGORY: u64 = 780106716837969951;
  pub const USER_ROLE: u64 = 735611852796461089;
  pub const ARCHIVE_CATEGORY: u64 = 822511597137559562;
  pub const AIRBNB_CHAN: u64 = 555206410619584519;
  pub const AITABLE_NOTIFY_CHAN: u64 = 555206410619584519;
  pub const DEVOPS_CHANNEL: u64 = 555206410619584519;
  pub const ANNOYED_CHAN_HERDINGCHATTE: u64 = 555206410619584519;
  pub const ANNOYED_CHAN_CYBERGOD: u64 = 555206410619584519;
  pub const ANNOYED_CHAN_TESTBOT: u64 = 555206410619584519;
  pub const DEPLOYMENT_CHAN: u64 = 826412321801764894;
}

lazy_static! {
  pub static ref TWO_FACTOR_DEPLOYMENT_CHANNEL: ChannelId = ChannelId(DEPLOYMENT_CHAN);
}
