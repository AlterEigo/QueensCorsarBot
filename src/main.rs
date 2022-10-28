use serde::{Serialize,Deserialize};

use std::{collections::HashMap, fmt::write, time::Duration};
use serenity::{
    prelude::*,
    model::prelude::*,
    framework::StandardFramework, Client,
    framework::standard::{macros::{command,group}, CommandResult},
    utils::MessageBuilder
};

use lazy_static::lazy_static;
use std::sync::RwLock;
use std::fs::File;
use std::io::{BufReader,BufRead,Write,Error};
use serenity::async_trait;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;

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

    let private = msg.author.create_dm_channel(&ctx.http).await?;
    private.send_message(&ctx.http, |m| {
        m.content("Здесь могла бы быть ваша реклама")
    }).await.expect("Couldn't send the direct message");

    let reply = user.await_reply(&ctx.shard)
        .timeout(Duration::new(20, 0))
        .await;

    if let Some(msg) = reply {

    }

    Ok(())
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
        todo!()
    }

    async fn resume(&self, ctx: Context, _arg2: ResumedEvent) {
        println!("Resume event fired!");
    }
    
    async fn message(&self, ctx: Context, new_message: Message) {
        let user = &new_message.author;

        if let Some(guild) = new_message.guild(&ctx.cache) {
            println!("(From group: {})", guild.name);
        } else {
            println!("Received message from: {}", new_message.author.name);
            todo!();
        }
        println!("Contents: {}", new_message.content);
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
            .push("Пссст! Есть тут кто? Ало-о-о? Проверка связи!")
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
