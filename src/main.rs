mod commands;
mod core;
mod handler;
mod logger;
mod prelude;
mod utility;

use crate::prelude::*;
use serenity::{framework::StandardFramework, model::prelude::*, Client};
use slog::{crit, debug, info, Logger};
use std::collections::HashMap;

const CRATE_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const TOKEN_ENV: &'static str = "QUEENSCORSAR_TOKEN";

#[tokio::main]
async fn main() -> UResult {
    let logger = logger::configure_full_root();
    info!(logger, "Starting QueenCorsar bot";
        "version" => CRATE_VERSION,
        "author" => "Iaroslav Sorokin",
        "email" => "iaroslav.sorokin@gmail.com",
        "upstream" => "https://github.com/AlterEigo/QueensCorsarBot"
    );

    let token = match std::env::var(TOKEN_ENV) {
        Ok(value) => value,
        Err(why) => {
            crit!(logger, "Could not retrieve Discord API token";
                "reason" => format!("{:}", why),
                "variable" => TOKEN_ENV
            );
            return Err(why.into());
        }
    };
    debug!(
        logger,
        "Successfully retrieved Discord API token from the environment"
    );

    let prefix = "!";
    let framework = StandardFramework::new()
        .configure(|c| c.prefix(prefix))
        .group(&GENERAL_GROUP);
    debug!(logger, "Serenity standard framework initialized";
        "prefix" => "!",
        "group" => &GENERAL_GROUP.name,
        "options" => format!("{:#?}", GENERAL_GROUP.options)
    );

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS;
    debug!(logger, "Gateway intents initialized"; "intents" => format!("{:#?}", intents));

    let mut loggers_map: HashMap<String, Logger> = HashMap::new();
    loggers_map
        .entry("root".to_owned())
        .or_insert(logger.clone());

    let mut client = match Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<LoggersKey>(loggers_map)
        .await
    {
        Ok(c) => c,
        Err(why) => {
            crit!(logger, "Could not initialize serenity client";
                "reason" => format!("{:?}", why)
            );
            return Err(why.into());
        }
    };
    debug!(logger, "Successfully initialized serenity client";
        "event handler" => "crate::Handler",
        "framework" => "StandardFramework");

    if let Err(why) = client.start().await {
        crit!(logger, "A critical error occured while running serenity client"; "reason" => format!("{:?}", why));
        return Err(why.into());
    }
    info!(logger, "Client terminated with no errors, goodbye!");
    Ok(())
}
