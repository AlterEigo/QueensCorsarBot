use crate::prelude::*;
use qcproto::prelude::*;
use serenity::{framework::StandardFramework, model::prelude::*, Client};
use slog::{crit, debug, info, Logger};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

const CRATE_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const TOKEN_ENV: &'static str = "QUEENSCORSAR_TOKEN";

#[derive(Clone)]
pub struct BootstrapRequirements {
    pub logger: slog::Logger,
    // pub config: config::Config,
}

pub fn bootstrap_command_server(ctx: &BootstrapRequirements) -> UResult {
    let srv_addr = format!(
        "{}",
        "/tmp/qcorsar.discord.sock".to_owned()
        // ctx.config.general.sock_addr.to_string_lossy().into_owned()
    );

    let command_handler = Arc::new(DefaultCommandHandler::new(ctx.logger.clone()));
    let command_dispatcher = Arc::new(DefaultCommandDispatcher::new(
        command_handler,
        ctx.logger.clone(),
    ));
    let stream_handler = Arc::new(DefaultUnixStreamHandler::new(
        command_dispatcher,
        ctx.logger.clone(),
    ));
    let cmd_server = CommandServer::new()
        .logger(ctx.logger.clone())
        .server_addr(&srv_addr)
        .stream_handler(stream_handler)
        .build()?;
    cmd_server.listen()
}

pub async fn bootstrap_application(ctx: BootstrapRequirements) -> UResult {
    info!(ctx.logger, "Starting QueenCorsar bot";
        "upstream" => "https://github.com/AlterEigo/QueensCorsarBot",
        "email" => "iaroslav.sorokin@gmail.com",
        "author" => "Iaroslav Sorokin",
        "version" => CRATE_VERSION,
    );

    let token = match std::env::var(TOKEN_ENV) {
        Ok(value) => value,
        Err(why) => {
            crit!(ctx.logger, "Could not retrieve Discord API token";
                "reason" => format!("{:}", why),
                "variable" => TOKEN_ENV
            );
            return Err(why.into());
        }
    };
    debug!(
        ctx.logger,
        "Successfully retrieved Discord API token from the environment"
    );

    let prefix = "!";
    let framework = StandardFramework::new()
        .configure(|c| c.prefix(prefix))
        .group(&GENERAL_GROUP);
    debug!(ctx.logger, "Serenity standard framework initialized";
        "options" => format!("{:#?}", GENERAL_GROUP.options),
        "group" => &GENERAL_GROUP.name,
        "prefix" => "!",
    );

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS;
    debug!(ctx.logger, "Gateway intents initialized"; "intents" => format!("{:#?}", intents));

    let mut loggers_map: HashMap<String, Logger> = HashMap::new();
    loggers_map
        .entry("root".to_owned())
        .or_insert(ctx.logger.clone());

    let mut client = match Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<LoggersKey>(loggers_map)
        .await
    {
        Ok(c) => c,
        Err(why) => {
            crit!(ctx.logger, "Could not initialize serenity client";
                "reason" => format!("{:?}", why)
            );
            return Err(why.into());
        }
    };
    debug!(ctx.logger, "Serenity client initialized";
        "event handler" => "crate::Handler",
        "framework" => "StandardFramework");

    thread::scope(move |scope| -> UResult {
        let runtime = tokio::runtime::Runtime::new()?;

        let logger = ctx.logger.clone();
        let client_thread = runtime.spawn(async move {
            if let Err(why) = client.start().await {
                crit!(logger, "A critical error occured while running serenity client"; "reason" => format!("{:?}", why));
                return UResult::Err(why.into());
            }
            UResult::Ok(())
        });

        let _ = scope.spawn(move || -> UResult {
            runtime.block_on(client_thread)??;
            Ok(())
        });

        let _ = scope.spawn(move || -> UResult {
            let cmd_server = bootstrap_command_server(&ctx);
            Ok(())
        });
        Ok(())
    })?;

    Ok(())
}
