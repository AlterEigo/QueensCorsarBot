use serde::{Serialize,Deserialize};

use std::collections::HashMap;
use serenity::{
    prelude::*,
    model::prelude::*,
    framework::StandardFramework, Client,
    framework::standard::{macros::{command,group}, CommandResult},
    utils::MessageBuilder
};

use serenity::async_trait;

#[derive(serde::Serialize,serde::Deserialize,Debug,Clone,Default)]
struct RegistrationInfos {
    user_id: u64,
    user_discriminator: u16,
    user_name: String,
    accepted_rules: bool,
    nickname: Option<String>
}

impl RegistrationInfos {
    fn read_registry() -> HashMap<u64, RegistrationInfos> {
        unimplemented!()
    }

    fn write_registry(registry: &HashMap<u64, RegistrationInfos>) {
        unimplemented!()
    }

    pub fn update_or_insert(infos: &RegistrationInfos) -> Result<(), (u16, &'static str)> {
        unimplemented!()
    }

    pub fn get_entry(id: u64) -> Option<RegistrationInfos> {
        unimplemented!()
    }

    pub fn user_met(id: u64) -> bool {
        let records = RegistrationInfos::read_registry();
        records.contains_key(&id)
    }

    pub fn user_accepted_rules(id: u64) -> bool {
        let records = RegistrationInfos::read_registry();
        if let Some(infos) = records.get(&id) {
            infos.accepted_rules
        } else {
            false
        }
    }

    pub fn set_rules_accepted(id: u64, value: bool) -> Result<RegistrationInfos, (u16, &'static str)> {
        let infos = match RegistrationInfos::get_entry(id) {
            Some(r) => RegistrationInfos { accepted_rules: value, ..r.clone() },
            None => return Err((1, "id not found"))
        };
        RegistrationInfos::update_or_insert(&infos)?;
        Ok(infos)
    }

    pub fn set_nickname(id: u64, nickname: &str) -> Result<RegistrationInfos, (u16, &'static str)> {
        let infos = match RegistrationInfos::get_entry(id) {
            Some(r) => RegistrationInfos { nickname: Some(String::from(nickname)), ..r.clone() },
            None => return Err((1, "id not found"))
        };
        RegistrationInfos::update_or_insert(&infos)?;
        Ok(infos)
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;

    Ok(())
}

#[command]
async fn rules(ctx: &Context, msg: &Message) -> CommandResult {
    let user = ctx.http.get_current_user().await?;

    let response = MessageBuilder::new()
        .push("Отправляю тебе свод правил, ")
        .push_bold_safe(&msg.author.name)
        .push("!")
        .build();
    msg.channel_id.say(&ctx.http, response).await?;

    let infos = RegistrationInfos {
        user_id: user.id.0,
        user_discriminator: user.discriminator,
        user_name: user.name.clone(),
        accepted_rules: false,
        nickname: None
    };

    let private = msg.author.create_dm_channel(&ctx.http).await?;
    private.send_message(&ctx.http, |m| {
        m.content("Здесь могла бы быть ваша реклама")
    }).await.expect("Couldn't send the direct message");

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
    
    async fn message(&self, ctx: Context, new_message: Message) {

        println!("Received message from: {}", new_message.author.name);
        println!("{}", new_message.content);
        if let Some(id) = new_message.guild_id {
            println!("{}", id);
        }
    }
    
    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        let response = MessageBuilder::new()
            .push("Ничоси, ")
            .push_bold_safe(new_member.display_name())
            .push_line(" присоединился! О.О")
            .push_line("Ну здарова, чо!")
            .build();
        let defalt_channel = new_member.default_channel(&ctx.cache).expect("Couldn't retrieve the default channel!");
        let msg = defalt_channel.say(&ctx.http, response);
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
