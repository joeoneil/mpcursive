#![allow(unused)]

use cursive::{
    theme::{Effect, Style, StyleType},
    utils::{
        markup::ansi::{self, Parser},
        span::{SpannedStr, SpannedString},
    },
    View, XY,
};
use log::{log, Level};
use mpd::Song;

use crate::mpd_util::MPD;

enum ColumnKey {
    Album,
    AlbumArtist,
    Artist,
    Disc,
    Duration,
    Title,
    Track,
}

struct Column {
    header: String,
    min_width: usize,
    ratio: f64,
    key: ColumnKey,
    format: &'static str,
}

impl Column {
    fn normalize(columns: &mut Vec<Column>) {
        let sum: f64 = columns.iter().map(|c| c.ratio).sum();
        columns.iter_mut().for_each(|c| c.ratio /= sum);
    }

    fn get(&self, song: &Song) -> Option<String> {
        match self.key {
            ColumnKey::Album => song
                .tags
                .iter()
                .find(|t| t.0.eq("Album"))
                .map(|t| t.1.clone()),
            ColumnKey::AlbumArtist => song
                .tags
                .iter()
                .find(|t| t.0.eq("AlbumArtist"))
                .map(|t| t.1.clone()),
            ColumnKey::Artist => song.artist.clone(),
            ColumnKey::Disc => song
                .tags
                .iter()
                .find(|t| t.0.eq("Disc"))
                .map(|t| t.1.clone()),
            ColumnKey::Duration => song
                .duration
                .map(|d| format!("{:02}:{:02}", d.as_secs() / 60, d.as_secs() % 60)),
            ColumnKey::Title => song.title.clone(),
            ColumnKey::Track => song
                .tags
                .iter()
                .find(|t| t.0.eq("Track"))
                .map(|t| t.1.clone())
                .map(|t| format!("{:02}", t.parse::<usize>().unwrap())),
        }
    }
}

pub struct Playlist {
    view_size: XY<usize>,
    offset: usize,
    selected: Option<usize>,
    columns: Vec<Column>,
}

impl Playlist {
    pub fn new() -> Self {
        Self {
            view_size: XY::zero(),
            offset: 0,
            selected: Some(0),
            columns: Playlist::default_columns(),
        }
    }

    fn default_columns() -> Vec<Column> {
        let mut cols = vec![
            Column {
                header: "Artist".into(),
                min_width: 6,
                ratio: 0.2,
                key: ColumnKey::AlbumArtist,
                format: "\x1b[33m",
            },
            Column {
                header: "Track".into(),
                min_width: 5,
                ratio: 0.0,
                key: ColumnKey::Track,
                format: "\x1b[32m",
            },
            Column {
                header: "Title".into(),
                min_width: 5,
                ratio: 0.6,
                key: ColumnKey::Title,
                format: "\x1b[37m",
            },
            Column {
                header: "Album".into(),
                min_width: 5,
                ratio: 0.2,
                key: ColumnKey::Album,
                format: "\x1b[36m",
            },
            Column {
                header: "Time".into(),
                min_width: 6,
                ratio: 0.0,
                key: ColumnKey::Duration,
                format: "\x1b[35m",
            },
        ];
        Column::normalize(&mut cols);
        cols
    }

    fn format_song(&self, song: &Song) -> String {
        let mut out = String::new();
        for (col, width) in self.column_widths() {
            out.push_str(
                format!(
                    "{0}{1:2$} {3}",
                    col.format,
                    col.get(song)
                        .unwrap_or("\x1b[90mUnknown".into())
                        .chars()
                        .take(width - 1)
                        .collect::<String>(),
                    width - 1,
                    "\x1b[0m"
                )
                .as_str(),
            )
        }
        out
    }

    fn column_widths(&self) -> Vec<(&Column, usize)> {
        let mut widths = vec![];
        let max_width = self.view_size.x;
        let mut width = 0;
        for col in &self.columns {
            if width + col.min_width > max_width {
                return widths;
            }
            widths.push((col, col.min_width));
        }
        widths.clear();
        // all columns fit with at least min_width
        let rem_width = max_width - self.columns.iter().map(|c| c.min_width).sum::<usize>();
        for col in &self.columns {
            // this is not entirely correct
            widths.push((col, col.min_width + (col.ratio * rem_width as f64) as usize));
        }
        widths
    }
}

impl View for Playlist {
    fn draw(&self, printer: &cursive::Printer) {
        let q = MPD::queue();
        if !q.is_some() {
            printer.print(XY { x: 0, y: 0 }, "Queue was None");
            return;
        }
        let q = q.unwrap();
        let current = MPD::now_playing().unwrap_or_default();
        let count = (q.len() - self.offset).min(printer.size.y - 1);
        for row in 0..count {
            let line = self.format_song(&q[row + self.offset]);
            let mut spanstr = ansi::parse(line);
            if (q[row + self.offset].eq(&current)) {
                spanstr
                    .spans_raw_attr_mut()
                    .for_each(|span| span.attr.effects |= Effect::Bold)
            }
            printer.print_styled(XY { x: 0, y: row + 2 }, &spanstr);
        }
    }

    fn layout(&mut self, size: cursive::Vec2) {
        self.view_size = size;
    }
}
