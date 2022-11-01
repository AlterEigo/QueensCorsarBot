mod core;
mod handler;
mod utility;
mod prelude;
mod commands;

use crate::prelude::*;
use serenity::{
    framework::StandardFramework,
    model::prelude::*,
    Client,
};

#[tokio::main]
async fn main() -> UResult {
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
