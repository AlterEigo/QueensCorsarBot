use serenity::prelude::*;
use std::collections::HashMap;
use std::fmt::Display;

/// Универсальное возвращаемое значение с возможностью типизирования параметра
pub type UResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Перечисление стандартных возможных ошибок бота
#[derive(Debug)]
pub enum BotError {
    TimedOut,
    NotInGuild,
    RulesRefused,
    MessageTooLong,
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
            Self::MessageTooLong => {
                write!(f, "Tryied to send the message bigger than Discord allows")?
            }
        }
        Ok(())
    }
}

pub struct LoggersKey;
impl TypeMapKey for LoggersKey {
    type Value = HashMap<String, slog::Logger>;
}

pub use crate::commands::*;
pub use crate::core::*;
pub use crate::handler::*;
pub use crate::utility::*;
