use crate::prelude::*;
use qcproto::prelude::*;
use serenity::prelude::{TypeMapKey, Context};
use serenity::utils::MessageBuilder;
use serenity::{framework::StandardFramework, model::prelude::*, Client};
use slog::{crit, debug, info, Logger};
use std::collections::HashMap;
use std::future::Future;
use std::marker::PhantomData;
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

pub struct SendersKey;
impl TypeMapKey for SendersKey {
    type Value = HashMap<String, CommandSender>;
}


pub struct PipesKey<T>
    where for<'x> T: 'x + Send + Sync
{
    phantom_t: PhantomData<T>
}
impl<T> TypeMapKey for PipesKey<T>
    where for<'x> T: 'x + Send + Sync
{
    type Value = HashMap<String, Pipe<T>>;
}

struct DsCommandHandler {
    logger: Logger,
    ds_context: Context,
    async_runtime: tokio::runtime::Runtime
}

impl DsCommandHandler {
    pub fn new(logger: Logger, ds_context: Context, async_runtime: tokio::runtime::Runtime) -> Self {
        Self { logger, ds_context, async_runtime }
    }
}

impl CommandHandler for DsCommandHandler {
    fn forward_message(&self, msg: Command) -> UResult {
        info!(self.logger, "Forwarding the message: {:#?}", msg);
        let ds_context = self.ds_context.clone();
        self.as_sync(async move {
            let guild = ds_context.http.get_guild(1032941443058241546).await?;
            let channel_id = ChannelId(1032942368015515708);
            let channels = guild.channels(&ds_context.http).await?;
            let channel = channels.get(&channel_id).unwrap();
            if let CommandKind::ForwardMessage { from, to: _, content } = msg.kind {
                let msg_content = MessageBuilder::new()
                    .push_bold_safe(from.name)
                    .push_line_safe(" пишет:")
                    .push_safe(content)
                    .build();
                channel.send_message(&ds_context.http, |m| {
                    m.content(msg_content)
                }).await?;
                UResult::Ok(())
            } else {
                UResult::Err("".into())
            }
        })??;
        // let _ = self.ds_context.http.send_message("1034419827525296191".parse::<u64>(), "");
        Ok(())
    }
}

impl DsCommandHandler {
    fn as_sync<F>(&self, f: F) -> UResult<<F as Future>::Output>
        where F: Future + Send + 'static,
              F::Output: Send + 'static
    {
        let t = self.async_runtime.spawn(f);
        Ok(self.async_runtime.block_on(t)?)
    }
}

pub fn bootstrap_command_server(ctx: &BootstrapRequirements, comm: Pipe<Context>) -> UResult<CommandServer> {
    let srv_addr = format!(
        "{}",
        "/tmp/qcorsar.discord.sock".to_owned()
        // ctx.config.general.sock_addr.to_string_lossy().into_owned()
    );

    let ds_context = comm.recv()?;
    let runtime = tokio::runtime::Runtime::new()?;
    let command_handler = Arc::new(DsCommandHandler::new(ctx.logger.clone(), ds_context, runtime));
    let command_dispatcher = Arc::new(DefaultCommandDispatcher::new(
        command_handler,
        ctx.logger.clone(),
    ));
    let stream_handler = Arc::new(DefaultUnixStreamHandler::new(
        command_dispatcher,
        ctx.logger.clone(),
    ));
    CommandServer::new()
        .logger(ctx.logger.clone())
        .server_addr(&srv_addr)
        .stream_handler(stream_handler)
        .build()
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
        let (cs_side, ds_side) = Pipe::<Context>::channel();

        let logger = ctx.logger.clone();
        let client_thread = runtime.spawn(async move {
            {
                let mut data = client.data.write().await;
                let pipes = data.entry::<PipesKey<Context>>().or_insert(HashMap::new());
                pipes.insert("cmdserver".to_owned(), ds_side);

                let senders = data.entry::<SendersKey>().or_insert(HashMap::new());
                let sender = CommandSender::new("/tmp/qcorsar.tg.sock".to_owned());
                senders.insert("tgsender".to_owned(), sender);
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

        let logger = ctx.logger.clone();
        let _ = scope.spawn(move || -> UResult {
            let cmd_server = bootstrap_command_server(&ctx, cs_side)?;
            if let Err(why) = cmd_server.listen() {
                crit!(logger, "Command server failed to run correctly; reason: {:#?}", why);
                Err(why.into())
            } else {
                Ok(())
            }
        });
        Ok(())
    })?;

    Ok(())
}
