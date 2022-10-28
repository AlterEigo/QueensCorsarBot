use serde::{Serialize,Deserialize};

use std::{collections::HashMap, fmt::write};
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

static REGISTRY_PATH: &'static str = "registry.json";
lazy_static! {
    static ref REGISTRY: RwLock<HashMap<u64, RegistrationInfos>> = {
        let path = std::path::Path::new(REGISTRY_PATH);
        if !path.exists() {
            let mut file = File::create(path).unwrap();
            write!(file, "{{}}").unwrap();
            file.sync_all().unwrap();
        }
        let registry = std::fs::read_to_string(REGISTRY_PATH).expect("Could not read the registry");
        let registry: HashMap<u64, RegistrationInfos> = serde_json::from_str(&registry).expect("Registry is corrupted (invalid json)");

        RwLock::new(registry)
    };
}

#[derive(serde::Serialize,serde::Deserialize,Debug,Clone,Default)]
struct RegistrationInfos {
    user_id: u64,
    user_discriminator: u16,
    user_name: String,
    accepted_rules: bool,
    nickname: Option<String>
}

impl RegistrationInfos {
    pub fn update_or_insert(infos: &RegistrationInfos) -> Result<(), (u16, &'static str)> {
        let mut records = REGISTRY.write().unwrap();
        if records.contains_key(&infos.user_id) {
            *records.get_mut(&infos.user_id).unwrap() = infos.clone();
        } else {
            records.insert(infos.user_id, infos.clone());
        }
        RegistrationInfos::save_registry(&*records).unwrap();
        Ok(())
    }

    fn save_registry(records: &HashMap<u64, RegistrationInfos>) -> Result<(), (u16, &'static str)> {
        let serialized = match serde_json::to_string(records) {
            Ok(v) => v,
            Err(_) => return Err((62, "Failed to serialize registry data to JSON"))
        };

        let mut registry = match File::create(REGISTRY_PATH) {
            Ok(v) => v,
            Err(_) => return Err((63, "Failed to create or open the registry file for writing"))
        };
     
        match registry.write_all(&serialized.into_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err((64, "Failed to write the registry into the file"))
        }
    }

    pub fn get_entry(id: u64) -> Option<RegistrationInfos> {
        let records = REGISTRY.read().unwrap();
        records.get(&id).cloned()
    }

    pub fn remove_entry(id: u64) {
        let mut records = REGISTRY.write().unwrap();
        records.remove_entry(&id);
        RegistrationInfos::save_registry(&*records).unwrap();
    }

    pub fn user_met(id: u64) -> bool {
        let records = REGISTRY.read().unwrap();
        records.contains_key(&id)
    }

    pub fn user_accepted_rules(id: u64) -> bool {
        let records = REGISTRY.read().unwrap();
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
    let user = &msg.author;

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

    RegistrationInfos::update_or_insert(&infos).unwrap();

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

    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        // Synchronize the registry with present players
        println!("Ready event fired!");
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
            // Is the player still present in the known guilds
            // => Player will not be present in the registry

            // Have we encountered this player before
            if !RegistrationInfos::user_met(user.id.0) {
                // Check if the player is still present on the server
                // true => create a record
                // false => deny request
            }

            // Did the player accept the rules

            // Do the player have a nickname set
        }
        println!("Contents: {}", new_message.content);
    }

    async fn guild_member_removal(&self, ctx: Context, _guild_id: GuildId, user: User, member_data: Option<Member>) {
        RegistrationInfos::remove_entry(user.id.0);
        println!("User with name '{}' (id: {}) has left the server. All associated data has been erased.", user.name, user.id.0);
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

        let infos = RegistrationInfos {
            user_id: new_member.user.id.0,
            user_name: new_member.user.name.clone(),
            user_discriminator: new_member.user.discriminator,
            accepted_rules: false,
            nickname: None
        };
        RegistrationInfos::update_or_insert(&infos).unwrap();
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
