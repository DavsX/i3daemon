#[macro_use]
extern crate lazy_static;
extern crate log;
extern crate simplelog;

use simplelog::{ConfigBuilder, LevelFilter, SimpleLogger};
use std::{collections::HashSet, env, process::Command};
use time::macros::format_description;

mod daemon;
mod tree;
mod window;

use daemon::I3Daemon;

lazy_static! {
    static ref ALLOWED_COMMANDS: HashSet<&'static str> =
        HashSet::from(["status", "stop", "start", "restart", "mask", "unmask"]);
}

fn main() {
    init_logging();

    let args: Vec<String> = env::args().collect();
    if let Some(arg) = args.get(1) {
        if ALLOWED_COMMANDS.contains(arg.as_str()) {
            Command::new("systemctl")
                .arg("--user")
                .arg(arg)
                .arg("i3daemon")
                .status()
                .expect("Command failed");
        }
    } else {
        // https://patorjk.com/software/taag/#p=display&f=Big&t=i3daemon
        log::info!(r"                                                  ");
        log::info!(r"  _ ____      _                                   ");
        log::info!(r" (_)___ \    | |                                  ");
        log::info!(r"  _  __) | __| | __ _  ___ _ __ ___   ___  _ __   ");
        log::info!(r" | ||__ < / _` |/ _` |/ _ \ '_ ` _ \ / _ \| '_ \  ");
        log::info!(r" | |___) | (_| | (_| |  __/ | | | | | (_) | | | | ");
        log::info!(r" |_|____/ \__,_|\__,_|\___|_| |_| |_|\___/|_| |_| ");
        log::info!(r"                                                  ");

        I3Daemon::new().run();
    }
}

fn init_logging() {
    let config = ConfigBuilder::new()
        .set_time_format_custom(format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second]"
        ))
        .build();
    SimpleLogger::init(LevelFilter::Info, config).expect("Failed to init logging");
}
