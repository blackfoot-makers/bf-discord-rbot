use crate::constants::discordids::GUILD_ID;
use crate::database::INSTANCE;
use serenity::{model::guild::Member, model::id::GuildId, prelude::*};

pub fn get_invite_counts_diff(ctx: Context, guild_id: GuildId) {
  let invites = guild_id.invites(&ctx.http).unwrap();
  for invite in invites {
    let db_instance = INSTANCE.write().unwrap();
    // db_instance
  }
}

pub fn check(ctx: Context, guild_id: GuildId, member: Member) {
  get_invite_counts_diff(ctx, guild_id)
}

pub fn new_invite() {}

pub fn create() {}
