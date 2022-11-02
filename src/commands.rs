use serenity::{
    framework::standard::{
        macros::{command, group},
        CommandResult,
    },
    model::prelude::*,
    prelude::*,
    utils::MessageBuilder,
};

use crate::core::start_signup_session;
use crate::prelude::*;

/// Команда проверки связи с ботом
#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;

    Ok(())
}

/// Команда запроса правил сервера и запуска процесса регистрации
#[command]
async fn rules(ctx: &Context, msg: &Message) -> CommandResult {
    let logger = child_logger(ctx, "command::rules").await?;
    info!(logger, "Executing 'rules' command";
        "initiator name" => &msg.author.name,
        "guild id" => msg.guild_id.unwrap().0,
        "initiator id" => msg.author.id.0
    );

    let user = &msg.author;

    let response = MessageBuilder::new()
        .push("Отправляю тебе свод правил, ")
        .push_bold_safe(&msg.author.name)
        .push("!")
        .build();
    msg.channel_id.say(&ctx.http, response).await?;

    debug!(logger, "Starting sign up session");
    if let Err(why) = start_signup_session(&ctx, &user, &msg.guild_id.unwrap()).await {
        let msg = "Ого! Что-то дало сбой... Пожалуйста не забудь сообщить об этом случае Иннри!";
        send_privately(ctx, user, msg).await?;
        error!(logger, "Could not successfully register the user"; "reason" => format!("{:?}", why));
        Err(why.into())
    } else {
        Ok(())
    }
}

/// Структура с основными командами бота
#[group]
#[commands(ping, rules)]
struct General;
