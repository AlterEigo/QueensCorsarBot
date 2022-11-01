use crate::prelude::*;
use serenity::{
    model::prelude::*,
    prelude::*,
};
use std::{
    collections::HashMap,
    time::Duration,
};

use std::sync::Arc;

pub async fn query_from_user(ctx: &Context, user: &User, msg: &str) -> UResult<String> {
    let dm = user.create_dm_channel(&ctx.http).await?;

    dm.send_message(&ctx.http, |m| m.content(msg)).await?;

    let reply = user
        .await_reply(&ctx.shard)
        .timeout(Duration::new(60 * 2, 0))
        .await
        .map(|m| Arc::try_unwrap(m).unwrap())
        .map(|m| m.content);

    reply.ok_or(BotError::TimedOut.into())
}

pub async fn user_is_in_guild(ctx: &Context, user: &User, gid: &GuildId) -> UResult<bool> {
    let r = ctx
        .http
        .search_guild_members(gid.0, user.name.as_str(), Some(1))
        .await?;
    Ok(r.len() > 0)
}

pub async fn send_privately(ctx: &Context, user: &User, msg: &str) -> UResult {
    let private = user.create_dm_channel(&ctx.http).await?;
    private.send_message(&ctx.http, |m| {
        m.content(msg)
    }).await?;
    Ok(())
}

pub struct Session;
impl TypeMapKey for Session {
    type Value = HashMap<UserId, bool>;
}

pub async fn push_session(ctx: &Context, uid: UserId, value: bool) {
    let mut data = ctx.data.write().await;
    let sessions = data.entry::<Session>().or_insert(HashMap::new());
    let entry = sessions.entry(uid).or_insert(true);
    *entry = value;
}

pub async fn pop_session(ctx: &Context, uid: UserId) {
    let mut data = ctx.data.write().await;
    let sessions = data.entry::<Session>().or_insert(HashMap::new());
    sessions.remove_entry(&uid);
}
