use chrono::Local;
use clap::ValueEnum;
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ValueEnum)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

pub fn setup_logger(log_level: LogLevel) {
    let level_filter = match log_level {
        LogLevel::Off => LevelFilter::Off,
        LogLevel::Error => LevelFilter::Error,
        LogLevel::Warn => LevelFilter::Warn,
        LogLevel::Info => LevelFilter::Info,
        LogLevel::Debug => LevelFilter::Debug,
        LogLevel::Trace => LevelFilter::Trace,
    };

    env_logger::Builder::new()
        .filter_level(level_filter)
        .format(|buf, record| {
            let now = Local::now();
            let timestamp = now.format("%Y-%m-%d %H:%M:%S%.6f");
            writeln!(buf, "{} [{:>5}] {}", timestamp, record.level(), record.args())
        })
        .init();

    // pretty_env_logger::env_logger::Builder::new()
    //     .filter_level(level_filter)
    //     .format(|buf, record| {
    //         let now = Local::now();
    //         let timestamp = now.format("%Y-%m-%d %H:%M:%S%.6f");
    //         writeln!(buf, "{} [{:>5}] - {}", timestamp, record.level(), record.args())
    //     })
    //     .init();

    //pretty_env_logger::init();
}

#[macro_export]
macro_rules! info_buf {
    ($buf:expr, $($arg:tt)+) => {{
        // Formátujeme správu len raz
        let msg = format!($($arg)+);
        // Logujeme do konzoly
        log::info!("{}", msg);
        // Ukladáme do bufferu
        $buf.push(msg);
    }};
}

// macro_rules! info_buf {
//     // Prvý argument je buffer, druhý je formátovací reťazec, za ním nasledujú voliteľné named args
//     ($buf:expr, $fmt:expr $(, $name:ident = $val:expr)* $(,)?) => {{
//         // Formátovanie správy pomocou format! a named args
//         let msg = format!($fmt $(, $name = $val)*);
//         // Logovanie
//         log::info!("{}", msg);
//         // Pridanie do bufferu
//         $buf.push(msg);
//     }};
// }

// eof
