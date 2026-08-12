//! Minimal leveled logging — no external crate, matches this crate's
//! zero-dependency `Cargo.toml`. Every line was previously a bare
//! `eprintln!("swarm-node: ...")`, indistinguishable from each other at
//! the terminal; this adds a level tag and a cheap env-var gate
//! (`SWARM_NODE_LOG=warn` to silence routine info lines) so operators
//! running a node for a while can tell noise from signal without reading
//! every line.

use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Level {
    Info,
    Warn,
    Error,
}

fn min_level() -> Level {
    static MIN: OnceLock<Level> = OnceLock::new();
    *MIN.get_or_init(|| match std::env::var("SWARM_NODE_LOG").as_deref() {
        Ok("error") => Level::Error,
        Ok("warn") => Level::Warn,
        _ => Level::Info,
    })
}

pub fn enabled(level: Level) -> bool {
    level >= min_level()
}

pub fn tag(level: Level) -> &'static str {
    match level {
        Level::Info => "info",
        Level::Warn => "warn",
        Level::Error => "error",
    }
}

macro_rules! log_info {
    ($($arg:tt)*) => {
        if $crate::log::enabled($crate::log::Level::Info) {
            eprintln!("swarm-node[{}]: {}", $crate::log::tag($crate::log::Level::Info), format!($($arg)*));
        }
    };
}
macro_rules! log_warn {
    ($($arg:tt)*) => {
        if $crate::log::enabled($crate::log::Level::Warn) {
            eprintln!("swarm-node[{}]: {}", $crate::log::tag($crate::log::Level::Warn), format!($($arg)*));
        }
    };
}
#[allow(unused_macros)]
macro_rules! log_error {
    ($($arg:tt)*) => {
        if $crate::log::enabled($crate::log::Level::Error) {
            eprintln!("swarm-node[{}]: {}", $crate::log::tag($crate::log::Level::Error), format!($($arg)*));
        }
    };
}

pub(crate) use log_info;
pub(crate) use log_warn;
#[allow(unused_imports)]
pub(crate) use log_error;
