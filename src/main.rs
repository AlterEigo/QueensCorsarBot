use serde::{Deserialize, Serialize};

use serenity::{
    framework::standard::{
        macros::{command, group},
        CommandResult,
    },
    framework::StandardFramework,
    http,
    model::prelude::*,
    prelude::*,
    utils::MessageBuilder,
    Client,
};
use std::{
    collections::HashMap,
    fmt::{write, Display},
    time::Duration,
};

use serenity::async_trait;
use std::fs::File;
use std::io::{BufRead, BufReader, Error, Write};
use std::sync::Arc;
use std::sync::RwLock;

type UResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
enum BotError {
    TimedOut,
    NotInGuild,
    RulesRefused,
}

unsafe impl Send for BotError {}
unsafe impl Sync for BotError {}
impl std::error::Error for BotError {}
impl Display for BotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::TimedOut => write!(f, "Operation timed out!")?,
            Self::NotInGuild => write!(f, "User is not a part of the current session's guild!")?,
            Self::RulesRefused => write!(f, "User explicitly declined the rules")?,
        }
        Ok(())
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;

    Ok(())
}

async fn query_from_user(ctx: &Context, user: &User, msg: &str) -> UResult<String> {
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

async fn user_is_in_guild(ctx: &Context, user: &User, gid: &GuildId) -> UResult<bool> {
    let r = ctx
        .http
        .search_guild_members(gid.0, user.name.as_str(), Some(1))
        .await?;
    Ok(r.len() > 0)
}

async fn send_privately(ctx: &Context, user: &User, msg: &str) -> UResult {
    todo!()
}

async fn start_signup_session(ctx: &Context, user: &User, gid: &GuildId) -> UResult {
    let private = user.create_dm_channel(&ctx.http).await?;
    loop {
        let msg = MessageBuilder::new().push("Тут типа свод правил").build();
        send_privately(ctx, user, &msg).await?;

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
                    "нет" | "no" | "-" | "n" => return Err(BotError::RulesRefused.into()),
                    _ => {
                        send_privately(ctx, user, "Вы можете ответить только 'Да' или 'Нет'").await?;
                        continue;
                    }
                },
                Err(_) => continue,
            }
        }

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

        break;
    }

    Ok(())
}

#[command]
async fn rules(ctx: &Context, msg: &Message) -> CommandResult {
    let user = &msg.author;

    let response = MessageBuilder::new()
        .push("Отправляю тебе свод правил, ")
        .push_bold_safe(&msg.author.name)
        .push("!")
        .build();
    msg.channel_id.say(&ctx.http, response).await?;

    start_signup_session(&ctx, &user, &msg.guild_id.unwrap()).await?;

    Ok(())
}

struct Session;
impl TypeMapKey for Session {
    type Value = HashMap<UserId, bool>;
}

async fn push_session(ctx: &Context, uid: UserId, value: bool) {
    let mut data = ctx.data.write().await;
    let sessions = data.entry::<Session>().or_insert(HashMap::new());
    let entry = sessions.entry(uid).or_insert(true);
    *entry = value;
}

async fn pop_session(ctx: &Context, uid: UserId) {
    let mut data = ctx.data.write().await;
    let sessions = data.entry::<Session>().or_insert(HashMap::new());
    sessions.remove_entry(&uid);
}

#[group]
#[commands(ping, rules)]
struct General;

struct Handler;

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
        // let response = MessageBuilder::new()
        // .push("Ничоси, ")
        // .push_bold_safe(new_member.display_name())
        // .push_line(" присоединился! О.О")
        // .push_line("Ну здарова, чо!")
        // .build();
        // let defalt_channel = new_member.default_channel(&ctx.cache).expect("Couldn't retrieve the default channel!");
        // let msg = defalt_channel.say(&ctx.http, response);

        // let greeting = MessageBuilder::new()
        // .push("Привет! Какой твой ник в игре?")
        // .build();
        // new_member.user.direct_message(&ctx.http, |m| {
        // m.content(&greeting)
        // }).await.expect("Could not send the private message");

        match start_signup_session(&ctx, &new_member.user, &new_member.guild_id).await {
            Err(contents) => {
                panic!("Something went wrong! Reason: {}", contents)
            }
            _ => (),
        };
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("QUEENSCORSAR_TOKEN")
        .expect("Couldn't fetch the API token from the environment");
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP);

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client!");

    if let Err(why) = client.start().await {
        println!("An error occured while running the client: {:?}", why);
    }
    Ok(())
}
