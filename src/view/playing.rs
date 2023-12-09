use std::time::Duration;

use cursive::{
    event::{Event, EventResult},
    Printer, View, XY,
};
use log::{log, Level};
use mpd::{Song, State};

use crate::mpd_util::MPD;

#[derive(Debug)]
pub struct Playing {
    song: Option<Song>,
    time: Option<(Duration, Duration)>,
    message: Option<String>,
    tick_count: usize,
}

impl Playing {
    pub fn new() -> Self {
        let mut s = Self {
            song: None,
            time: None,
            message: None,
            tick_count: 0,
        };
        s.update();
        s
    }

    pub fn update(&mut self) {
        if self.tick_count % 25 == 0 {
            self.song = MPD::now_playing();
            self.time = MPD::current_time();
        }
        self.tick_count += 1;
    }

    pub(super) fn lock_title(&mut self, msg: String) {
        self.message = Some(msg);
    }

    pub(super) fn unlock_title(&mut self) {
        self.message.take();
    }

    fn format_title(&self) -> String {
        if let Some(s) = &self.message {
            return s.clone();
        }
        let mut out = String::from("\x1b[1m"); // bold
        out.push_str(match MPD::status() {
            Some(s) => match s.state {
                State::Stop => return String::from("\x1b[1mStopped\x1b[0m"),
                State::Pause => "Paused: ",
                State::Play => "Playing: ",
            },
            None => {
                log!(Level::Warn, "Song status not found");
                return String::new();
            }
        });
        out.push_str("\x1b[0m"); // reset
        out.push_str(
            match &self.song {
                None => String::from("Unknown"),
                Some(s) => {
                    let artist = s
                        .tags
                        .iter()
                        .find(|t| t.0.eq("AlbumArtist"))
                        .map(|t| t.1.clone())
                        .or(s.artist.clone());
                    let title = s.title.clone();
                    let date = s.tags.iter().find(|t| t.0.eq("Date")).map(|t| t.1.clone());
                    let album = s.tags.iter().find(|t| t.0.eq("Album")).map(|t| t.1.clone());
                    let r = match (artist, album, date, title) {
                        (_, _, _, None) => String::from("Unknown"),
                        (Some(aa), Some(a), Some(d), Some(t)) => {
                            format!("{} \"{}\" ({}) - {}", aa, a, d, t)
                        }
                        (Some(aa), Some(a), None, Some(t)) => format!("{} \"{}\" - {}", aa, a, t),
                        (Some(aa), None, _, Some(t)) => format!("{} - {}", aa, t),
                        (_, _, _, Some(t)) => format!("{}", t),
                    };
                    r
                }
            }
            .as_str(),
        );
        return out;
    }
}

impl View for Playing {
    fn draw(&self, printer: &Printer<'_, '_>) {
        printer.print(
            XY::from((0, printer.size.y - 1)),
            self.format_title().as_str(),
        );
        if let Some(time) = &self.time {
            let mut el = time.1;
            if let Some(e) = MPD::elapsed() {
                el = e;
            }
            let pct = el.as_secs_f64() / time.1.as_secs_f64();
            printer.print(
                XY::from((0, printer.size.y - 2)),
                format!(
                    "{}>",
                    "=".repeat(((printer.size.x as f64 * pct) as usize).max(1) - 1)
                )
                .as_str(),
            );
        }
    }

    fn required_size(&mut self, constraint: XY<usize>) -> XY<usize> {
        XY {
            x: constraint.x,
            y: constraint.y.min(2),
        }
    }

    fn on_event(&mut self, e: Event) -> EventResult {
        match e {
            Event::Refresh => {
                self.update();
                EventResult::Ignored
            }
            _ => EventResult::Ignored,
        }
    }
}
