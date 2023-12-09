#![allow(unused)]

use std::fs;
use std::time::Instant;

use cursive::views::ResizedView;
use cursive::{Cursive, CursiveExt};
use flexi_logger::Logger;
use log::{log, Level};

use mpcursive::view::root::Root;
use mpcursive::{global_cursive, mpd_util::*};

fn main() {
    mpcursive::init();
    let mut siv = global_cursive();

    siv.set_fps(30);
    siv.set_autorefresh(true);

    Logger::try_with_env_or_str("debug,cursive=info")
        .expect("Couldn't create logger")
        .log_to_file_and_writer(
            flexi_logger::FileSpec::default()
                .directory("logs")
                .suppress_timestamp(),
            cursive_flexi_logger_view::cursive_flexi_logger(&siv),
        )
        .format(flexi_logger::colored_with_thread)
        .start()
        .expect("Failed to initialize logger");

    siv.add_fullscreen_layer(ResizedView::with_full_screen(Root::new()));

    siv.load_toml(fs::read_to_string("themes/dark.toml").unwrap().as_str())
        .unwrap();

    log!(Level::Debug, "Starting run");
    siv.run_termion().unwrap();
    log!(Level::Debug, "End");
}
