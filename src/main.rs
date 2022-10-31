use serde::{Serialize,Deserialize};

use std::{collections::HashMap, fmt::write, time::Duration};
use serenity::{
    prelude::*,
    model::prelude::*,
    framework::StandardFramework, Client,
    framework::standard::{macros::{command,group}, CommandResult},
    utils::MessageBuilder
};

use std::sync::RwLock;
use std::fs::File;
use std::io::{BufReader,BufRead,Write,Error};
use serenity::async_trait;

type UResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;

    Ok(())
}

async fn start_signup_session(ctx: &Context, user: &User, gid: &GuildId) -> UResult {
    let private = user.create_dm_channel(&ctx.http).await?;
    private.send_message(&ctx.http, |m| {
        m.content("Здесь могла бы быть ваша реклама")
    }).await.expect("Couldn't send the direct message");

    todo!()
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

async fn engage_registration(ctx: Context, user: &User) -> Result<Context, Box<dyn std::error::Error>> {
    let greeting = MessageBuilder::new()
        .push("Привет! Какой твой ник в игре?")
        .build();
    user.direct_message(&ctx.http, |m| {
        m.content(&greeting)
    }).await?;

    let reply = user.await_reply(&ctx.shard)
        .timeout(Duration::new(60 * 60, 0))
        .await.ok_or("Did not wait long enough")?;

    Ok(ctx)
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
        todo!()
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
        
        let greeting = MessageBuilder::new()
            .push("Привет! Какой твой ник в игре?")
            .build();
        new_member.user.direct_message(&ctx.http, |m| {
            m.content(&greeting)
        }).await.expect("Could not send the private message");
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
