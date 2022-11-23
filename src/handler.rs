use crate::{prelude::*, core::application::SendersKey};
use serenity::{model::prelude::*, prelude::*};
use slog::o;
use crate::application::PipesKey;

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
            "guild id" => match msg.guild(ctx.cache) {
                Some(guild) => format!("{}", guild.id),
                None => "None".to_owned()
            },
            "initiator" => format!("({}, {})", msg.author.name, msg.author.id.0),
            "unique execution id" => unique_nano(),
        ));

        let bot_uid = 1034395163302297600;
        if msg.author.id != bot_uid {
            info!(logger, "Message event fired"; "content" => msg.content.clone());
        } else {
            info!(logger, "Message event fired");
            return;
        }

        let source_chat_id = 1034419827525296191 as u64;
        if msg.channel_id != source_chat_id {
            return;
        }

        let author_fmt = {
            let res = msg.author_nick(&ctx.http).await;
            if let Some(nickname) = res {
                format!("{} ({})", nickname, msg.author.name)
            } else {
                msg.author.name
            }
        };
        let cmd = Command {
            kind: CommandKind::ForwardMessage {
                from: ActorInfos { server: "discord_server_id".to_owned(), name: author_fmt },
                to: ActorInfos { server: "telegram_server_id".to_owned(), name: Default::default() },
                content: msg.content
            },
            sender_bot_family: BotFamily::Discord,
            protocol_version: PROTOCOL_VERSION
        };
        
        {
            let data = ctx.data.write().await;
            let senders = data.get::<SendersKey>().unwrap();
            let sender = senders.get("tgsender").unwrap();
            if let Err(why) = sender.send(cmd) {
                error!(logger, "Could not send a command to the other process; reason: {:#?}", why);
            }
        }

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

        let data = ctx.data.write().await;
        let pipes = data.get::<PipesKey<Context>>().unwrap();
        let pipe = pipes.get("cmdserver").unwrap();

        if let Err(why) = pipe.send(ctx.clone()) {
            error!(logger, "Could not send bot context via the provided pipe"; "reason" => format!("{:#?}", why));
        }

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
