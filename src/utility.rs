use crate::prelude::*;
use nanoid::nanoid;
use serenity::{model::prelude::*, prelude::*};
use slog::Logger;
use std::{sync::atomic::AtomicUsize, time::Duration};

use slog::o;
use std::sync::Arc;

static UNIQUE_COUNTER: AtomicUsize = AtomicUsize::new(1);
const NANOID_LEN: usize = 16;

/// Создание уникального идентификатора
///
/// Данная функция адаптирована для многопоточности
pub fn unique_id() -> usize {
    UNIQUE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

pub fn unique_nano() -> String {
    nanoid!(NANOID_LEN)
}

/// Запроса ввода от пользователя
///
/// Данная функция сначала представляет пользователю
/// указанное сообщение, затем ожидает его ответа в
/// течении 120 секунд и возвращает результат ввода.
/// Если же пользователь так и не ответил, возвращается
/// ошибка типа `BotError::TimedOut`
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

/// Создание дочерней копии основного логгера
pub async fn child_logger(ctx: &Context, submodule: &str) -> UResult<Logger> {
    let data = ctx.data.read().await;
    let loggers = data
        .get::<LoggersKey>()
        .ok_or(BotError::DataNotFound("Loggers hashmap"))?;
    loggers
        .get("root")
        .map(|logger| logger.new(o!("from" => submodule.to_owned())))
        .ok_or(BotError::DataNotFound("Root logger").into())
}

/// Проверка присутствия пользователя в указанной группе
pub async fn user_is_in_guild(ctx: &Context, user: &User, gid: &GuildId) -> UResult<bool> {
    let r = ctx
        .http
        .search_guild_members(gid.0, user.name.as_str(), Some(1))
        .await?;
    Ok(r.len() > 0)
}

/// Отправка личного сообщения пользователю
pub async fn send_privately(ctx: &Context, user: &User, msg: &str) -> UResult {
    let private = user.create_dm_channel(&ctx.http).await?;
    private.send_message(&ctx.http, |m| m.content(msg)).await?;
    Ok(())
}
