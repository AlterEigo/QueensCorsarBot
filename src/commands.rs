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

/// Команда проверки связи с ботом
#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;

    Ok(())
}

/// Команда запроса правил сервера и запуска процесса регистрации
#[command]
async fn rules(ctx: &Context, msg: &Message) -> CommandResult {
    let user = &msg.author;

    let response = MessageBuilder::new()
        .push("Отправляю тебе свод правил, ")
        .push_bold_safe(&msg.author.name)
        .push("!")
        .build();
    msg.channel_id.say(&ctx.http, response).await?;

    match start_signup_session(&ctx, &user, &msg.guild_id.unwrap()).await {
        Err(why) => println!("[rules]: Something went wrong: {:?}", why),
        _ => (),
    };

    Ok(())
}

/// Структура с основными командами бота
#[group]
#[commands(ping, rules)]
struct General;
