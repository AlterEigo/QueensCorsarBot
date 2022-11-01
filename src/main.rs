use serde::{Serialize,Deserialize};

use std::{collections::HashMap, fmt::{write, Display}, time::Duration};
use serenity::{
    prelude::*,
    model::prelude::*,
    framework::StandardFramework, Client,
    framework::standard::{macros::{command,group}, CommandResult},
    utils::MessageBuilder, http
};

use std::sync::RwLock;
use std::sync::Arc;
use std::fs::File;
use std::io::{BufReader,BufRead,Write,Error};
use serenity::async_trait;

type UResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
enum BotError {
    TimedOut
}

unsafe impl Send for BotError {}
unsafe impl Sync for BotError {}
impl std::error::Error for BotError {}
impl Display for BotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::TimedOut => write!(f, "Operation timed out!")?
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

    dm.send_message(&ctx.http, |m| {
        m.content(msg)
    }).await?;

    let reply = user.await_reply(&ctx.shard)
        .timeout(Duration::new(60 * 2, 0))
        .await
        .map(|m| Arc::try_unwrap(m).unwrap())
        .map(|m| m.content);

    reply.ok_or(BotError::TimedOut.into())
}

async fn start_signup_session(ctx: &Context, user: &User, gid: &GuildId) -> UResult {
    let private = user.create_dm_channel(&ctx.http).await?;
    loop {
        private.send_message(&ctx.http, |m| {
            m.content("Тут типа правила")
             .content("Согласны ли вы с правилами?")
        }).await?;

        let reply = user.await_reply(&ctx.shard)
            .timeout(Duration::new(60 * 2, 0))
            .await
            .map(|msg| Arc::try_unwrap(msg).unwrap())
            .map(|msg| msg.content);

        // 1. Проверяем на наличие в гильдии

        if reply.is_none() {
            private.send_message(&ctx.http, |m| {
                m.content("Упс! Время истекло. Попробуем еще раз?")
            }).await?;
            continue;
        }

        // 1. Проверяем наличие в гильдии
        // 2. Проверка значения ответа

        private.send_message(&ctx.http, |m| {
            m.content("Отлично! Позволь узнать твой ник в игре?")
        }).await?;

        let reply = user.await_reply(&ctx.shard)
            .timeout(Duration::new(60 * 2, 0))
            .await
            .map(|msg| Arc::try_unwrap(msg).unwrap())
            .map(|msg| msg.content);

        if reply.is_none() {
            private.send_message(&ctx.http, |m| {
                m.content("Упс! Время истекло. Попробуем еще раз?")
            }).await?;
            continue;
        }

        // 1. Проверяем наличие в гильдии

        let reply = reply.unwrap();

        let role_id: u64 = 1036571268809498654;
        ctx.http.add_member_role(gid.0, user.id.0, role_id, Some("Автоматическое назначение роли")).await?;
        ctx.http.get_guild(gid.0).await?
            .edit_member(&ctx.http, user.id.0, |member| {
                member.nickname(reply)
            }).await?;

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
#[commands(ping,rules)]
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

    async fn guild_member_removal(&self, ctx: Context, _guild_id: GuildId, user: User, member_data: Option<Member>) {
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
            Err(contents) => { panic!("Something went wrong! Reason: {}", contents) },
            _ => ()
        };
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("QUEENSCORSAR_TOKEN").expect("Couldn't fetch the API token from the environment");
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP);
    
    let intents = GatewayIntents::non_privileged() |
        GatewayIntents::MESSAGE_CONTENT |
        GatewayIntents::GUILD_MEMBERS;
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
