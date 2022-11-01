use std::fmt::Display;
use std::io::Write;

pub type UResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
pub enum BotError {
    TimedOut,
    NotInGuild,
    RulesRefused,
}

unsafe impl Send for BotError {}
unsafe impl Sync for BotError {}
impl std::error::Error for BotError {}
impl Display for BotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::TimedOut => write!(f, "Operation timed out!")?,
            Self::NotInGuild => write!(f, "User is not a part of the current session's guild!")?,
            Self::RulesRefused => write!(f, "User explicitly declined the rules")?,
        }
        Ok(())
    }
}

pub use crate::core::*;
pub use crate::utility::*;
pub use crate::commands::*;
pub use crate::handler::*;
