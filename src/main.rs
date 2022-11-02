mod commands;
mod core;
mod handler;
mod logger;
mod prelude;
mod utility;

use crate::prelude::*;
use serenity::{framework::StandardFramework, model::prelude::*, prelude::TypeMapKey, Client};
use slog::{crit, debug, error, info, o, warn, Logger};
use std::collections::HashMap;

const CRATE_VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> UResult {
    let logger = logger::configure_compact_root();
    info!(logger, "Starting QueenCorsar bot"; "version" => CRATE_VERSION, "author" => "Iaroslav Sorokin (iaroslav.sorokin@gmail.com)");

    let token = std::env::var("QUEENSCORSAR_TOKEN")
        .expect("Couldn't fetch the API token from the environment");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
        .group(&GENERAL_GROUP);

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS;

    let mut loggers_map: HashMap<String, Logger> = HashMap::new();
    loggers_map
        .entry("root".to_owned())
        .or_insert(logger.clone());

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<LoggersKey>(loggers_map)
        .await
        .expect("Error creating client!");

    if let Err(why) = client.start().await {
        println!("An error occured while running the client: {:?}", why);
    }
    Ok(())
}
