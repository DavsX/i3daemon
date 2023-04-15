#[macro_use]
extern crate lazy_static;

mod daemon;
mod tree;
mod window;

use daemon::I3Daemon;

fn main() {
    // https://patorjk.com/software/taag/#p=display&f=Big&t=i3daemon
    println!(r"                                                  ");
    println!(r"  _ ____      _                                   ");
    println!(r" (_)___ \    | |                                  ");
    println!(r"  _  __) | __| | __ _  ___ _ __ ___   ___  _ __   ");
    println!(r" | ||__ < / _` |/ _` |/ _ \ '_ ` _ \ / _ \| '_ \  ");
    println!(r" | |___) | (_| | (_| |  __/ | | | | | (_) | | | | ");
    println!(r" |_|____/ \__,_|\__,_|\___|_| |_| |_|\___/|_| |_| ");
    println!(r"                                                  ");

    I3Daemon::new().run();
}
