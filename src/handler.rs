use crate::prelude::*;
use serenity::{
    model::prelude::*,
    prelude::*,
};

use serenity::async_trait;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // fn message(&self, ctx: Context, msg: Message) {
    // unimplemented!();
    // }

    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        // Synchronize the registry with present players
        println!("Ready event fired!");
        // todo!()
    }

    async fn resume(&self, ctx: Context, _arg2: ResumedEvent) {
        println!("Resume event fired!");
    }

    async fn guild_member_removal(
        &self,
        ctx: Context,
        _guild_id: GuildId,
        user: User,
        member_data: Option<Member>,
    ) {
        // todo!()
    }

    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        match start_signup_session(&ctx, &new_member.user, &new_member.guild_id).await {
            Err(contents) => {
                panic!("Something went wrong! Reason: {}", contents)
            }
            _ => (),
        };
    }
}

