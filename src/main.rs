#![allow(unused)]

mod commands;
mod core;
mod handler;
mod logger;
mod prelude;
mod utility;

use crate::prelude::*;
use qcproto::prelude::*;
use serenity::{framework::StandardFramework, model::prelude::*, Client};
use slog::{crit, debug, info, Logger};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use crate::core::application;

#[tokio::main]
async fn main() -> UResult {
    let logger = logger::configure_compact_root()?;
    let reqs = application::BootstrapRequirements {
        logger
    };

    application::bootstrap_application(reqs).await?;

    Ok(())
}
