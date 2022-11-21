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
use std::thread;

const CRATE_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const TOKEN_ENV: &'static str = "QUEENSCORSAR_TOKEN";

#[tokio::main]
async fn main() -> UResult {
    let logger = logger::configure_compact_root()?;
    info!(logger, "Starting QueenCorsar bot";
        "upstream" => "https://github.com/AlterEigo/QueensCorsarBot",
        "email" => "iaroslav.sorokin@gmail.com",
        "author" => "Iaroslav Sorokin",
        "version" => CRATE_VERSION,
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
        "options" => format!("{:#?}", GENERAL_GROUP.options),
        "group" => &GENERAL_GROUP.name,
        "prefix" => "!",
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
    debug!(logger, "Serenity client initialized";
        "event handler" => "crate::Handler",
        "framework" => "StandardFramework");

    let runtime = tokio::runtime::Runtime::new()?;
    let client_thread = runtime.spawn(async move {
        if let Err(why) = client.start().await {
            crit!(logger, "A critical error occured while running serenity client"; "reason" => format!("{:?}", why));
            return UResult::Err(why.into());
        }
        UResult::Ok(())
    });

    thread::scope(move |scope| -> UResult {
        let _ = scope.spawn(move || -> UResult {
            runtime.block_on(client_thread)??;
            Ok(())
        });
        Ok(())
    })?;

    // if let Err(why) = client.start().await {
    // crit!(logger, "A critical error occured while running serenity client"; "reason" => format!("{:?}", why));
    // return Err(why.into());
    // }
    // info!(logger, "Client terminated with no errors, goodbye!");
    Ok(())
}
