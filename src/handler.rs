use crate::prelude::*;
use serenity::{model::prelude::*, prelude::*};
use slog::o;

use serenity::async_trait;

pub struct Handler;

/// Блок имплементации черт обработчика событий
///
/// Данный блок позволяет имплементировать методы Serenity
/// для перехвата и обработки программных событий Discord
#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let logger = match child_logger(&ctx, "event::message").await {
            Ok(value) => value,
            Err(why) => panic!("Failed to retrieve the logger: {:#?}", why),
        };
        let logger = logger.new(o!(
            "initiator" => format!("({}, {})", msg.author.name, msg.author.id.0),
            "unique execution id" => unique_nano(),
            "guild id" => match msg.guild(ctx.cache) {
                Some(guild) => format!("{}", guild.id),
                None => "None".to_owned()
            }
        ));

        info!(logger, "Message event fired"; "content" => msg.content);
    }

    /// Обработчик события полной готовности бота
    ///
    /// Данное событие вызывается как только бот установил
    /// соединение с серверами дискорда и готов к последующей
    /// инициализации и работе
    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        let logger = match child_logger(&ctx, "event::ready").await {
            Ok(value) => value,
            Err(why) => panic!("Failed to retrieve the logger: {:#?}", why),
        };

        info!(
            logger,
            "Bot successfully initialized and ready for requests"
        );
    }

    /// Обработчик события переподключения
    ///
    /// Уточнить условия при которых происходит вызов
    async fn resume(&self, ctx: Context, _arg2: ResumedEvent) {
        let logger = match child_logger(&ctx, "event::resume").await {
            Ok(value) => value,
            Err(why) => panic!("Failed to retrieve the logger: {:#?}", why),
        };

        info!(logger, "Resume event fired");
    }

    /// Выход пользователя из группы
    ///
    /// Событие при уходе пользователя из группы, будь то
    /// кик или самостоятельный уход
    async fn guild_member_removal(
        &self,
        ctx: Context,
        _guild_id: GuildId,
        user: User,
        member_data: Option<Member>,
    ) {
        // todo!()
    }

    /// Вход пользователя на сервер
    ///
    /// Событие при входе пользователя на сервер
    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        match start_signup_session(&ctx, &new_member.user, &new_member.guild_id).await {
            Err(contents) => {
                panic!("Something went wrong! Reason: {}", contents)
            }
            _ => (),
        };
    }
}
