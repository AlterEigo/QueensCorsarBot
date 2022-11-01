use std::{path::Path, fs::File, io::{BufReader, Read}};

use serenity::{
    model::prelude::*,
    prelude::*,
    utils::MessageBuilder
};

use crate::prelude::*;

/// Загрузка текста с правилами гильдии из предусмотренного файла
fn load_guild_rules() -> UResult<String> {
    let path = Path::new("rules.md");
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer)?;
    Ok(buffer)
}

pub async fn start_signup_session(ctx: &Context, user: &User, gid: &GuildId) -> UResult {
    let rules = load_guild_rules()?
        .split_terminator("===")
        .map(|paragraph| String::from(paragraph))
        .collect::<Vec<String>>();

    for paragraph in rules {
        let msg = MessageBuilder::new()
            .push(paragraph)
            .build();
        send_privately(ctx, user, &msg).await?;
    }

    let msg = MessageBuilder::new()
        .push_bold_line_safe("Принимаете ли вы свод правил гильдии? (Да/Нет)")
        .build();
    loop {
        if !user_is_in_guild(ctx, user, gid).await? {
            send_privately(ctx, user, "Увы, вы больше не состоите в группе гильдии!").await?;
            return Err(BotError::NotInGuild.into());
        }
        match query_from_user(ctx, user, &msg)
            .await
            .map(|m| m.to_lowercase())
        {
            Ok(r) => match r.as_str() {
                "да" | "+" | "ок" | "yes" | "y" => break,
                "нет" | "no" | "-" | "n" => {
                    send_privately(ctx, user, "Ну, на нет и суда нет! Если вдруг передумаешь - введи команду `!rules` в чате гильдии!").await?;
                    return Err(BotError::RulesRefused.into())
                },
                _ => {
                    send_privately(ctx, user, "Вы можете ответить только 'Да' или 'Нет'").await?;
                    continue;
                }
            },
            Err(_) => continue,
        }
    }

    let msg = MessageBuilder::new()
        .push_bold_line_safe("Теперь сообщи мне пожалуйста свой ник в игре, и я поставлю тебе его в группе")
        .build();
    let nickname = loop {
        if !user_is_in_guild(ctx, user, gid).await? {
            send_privately(ctx, user, "Увы, вы больше не состоите в группе гильдии!").await?;
            return Err(BotError::NotInGuild.into());
        }
        match query_from_user(ctx, user, &msg).await {
            Ok(r) => break r,
            Err(_) => continue,
        }
    };

    let role_id: u64 = 1036571268809498654;
    ctx.http
        .add_member_role(
            gid.0,
            user.id.0,
            role_id,
            Some("Автоматическое назначение роли"),
        )
        .await?;
    ctx.http
        .get_guild(gid.0)
        .await?
        .edit_member(&ctx.http, user.id.0, |member| member.nickname(nickname))
        .await?;

    let msg = MessageBuilder::new()
        .push_bold_line_safe("Всё готово! Тебе выдана роль в группе и поставлен псевдоним. Приятной игры!")
        .build();
    send_privately(ctx, user, &msg).await?;

    Ok(())
}

