// #![allow(unused)]

mod commands;
mod core;
mod handler;
mod logger;
mod prelude;
mod utility;
mod sender;

use crate::prelude::*;
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
