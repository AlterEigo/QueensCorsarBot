use crate::prelude::*;
use qcproto::prelude::*;
use serenity::prelude::TypeMapKey;
use serenity::utils::MessageBuilder;
use serenity::{framework::StandardFramework, model::prelude::*, Client};
use slog::{crit, debug, info, Logger};
use std::collections::HashMap;
use std::sync::Arc;
use serde_json::{json, Value};
use std::thread;
use std::sync::RwLock;

use std::sync::mpsc;

const CRATE_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const TOKEN_ENV: &'static str = "QUEENSCORSAR_TOKEN";

#[derive(Clone)]
pub struct BootstrapRequirements {
    pub logger: slog::Logger,
    // pub config: config::Config,
}

#[derive(Debug)]
pub struct Pipe<ST = Value, RT = ST>
    where ST: Send + Sync,
          RT: Send + Sync
{
    r: RwLock<mpsc::Receiver<RT>>,
    s: RwLock<mpsc::Sender<ST>>
}

unsafe impl<ST, RT> Sync for Pipe<ST, RT>
    where ST: Send + Sync,
          RT: Send + Sync {}

impl<ST, RT> Pipe<ST, RT>
    where ST: Send + Sync,
          RT: Send + Sync
{
    pub fn channel() -> (Self, Pipe<RT, ST>) {
        let (s1, r1) = mpsc::channel::<ST>();
        let (s2, r2) = mpsc::channel::<RT>();
        (
            Self { r: RwLock::new(r2), s: RwLock::new(s1) },
            Pipe { s: RwLock::new(s2), r: RwLock::new(r1) }
        )
    }

    pub fn send(&self, data: ST) -> UResult
    {
        let lock = self.s.write();
        if let Err(why) = lock {
            return Err("RwLock on sender seems to be poisoned".into());
        }
        let lock = lock.unwrap();

        if let Err(why) = lock.send(data) {
            Err("Receiver doesn't seem to exist anymore".into())
        } else {
            Ok(())
        }
    }

    pub fn recv(&self) -> UResult<RT> {
        let lock = self.r.write();
        if let Err(why) = lock {
            return Err("RwLock on receiver seems to be poisoned".into());
        }
        let lock = lock.unwrap();

        let res = lock.recv();
        if let Err(ref why) = res {
            Err("Sender doesn't seem to exist anymore".into())
        } else {
            Ok(res.unwrap())
        }
    }
}

struct PipeKey;
impl TypeMapKey for PipeKey
{
    type Value = HashMap<String, Pipe>;
}

struct CommandHandler {
    logger: Logger
}

pub fn bootstrap_command_server(ctx: &BootstrapRequirements, comm: Pipe) -> UResult {
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
        let (cs_side, ds_side) = Pipe::channel();

        let logger = ctx.logger.clone();
        let client_thread = runtime.spawn(async move {
            {
                let mut data = client.data.write().await;
                let mut pipes = data.entry::<PipeKey>().or_insert(HashMap::new());
                pipes.insert("cmdserver".to_owned(), ds_side);
            }
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
            let cmd_server = bootstrap_command_server(&ctx, cs_side);
            Ok(())
        });
        Ok(())
    })?;

    Ok(())
}
