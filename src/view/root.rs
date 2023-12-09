#![allow(unused)]

use std::time::{Duration, Instant};

use cursive::{
    event::{Event, EventResult, Key},
    view::ViewWrapper,
    views::{DummyView, NamedView, ResizedView, TextView},
    Vec2, View, XY,
};
use cursive_flexi_logger_view::FlexiLoggerView;
use log::{log, Level};

use super::{playing::Playing, titlebar::Titlebar};
use crate::global_cursive;

enum EventMode {
    Pass,
    Input,
}

pub struct Root {
    // Child views
    titlebar: ResizedView<Titlebar>,
    content: Vec<Box<dyn View>>,
    playing: ResizedView<Playing>,

    // State
    selected: usize,
    input: Option<String>,
    mode: EventMode,
    last_tick: Instant,
}

impl Root {
    pub fn new() -> Self {
        Self {
            titlebar: ResizedView::with_fixed_height(2, Titlebar::new("Title".into())),
            content: vec![
                Box::new(FlexiLoggerView::new()),
                Box::new(super::playlist::Playlist::new()),
            ],
            //content: vec![Box::new(TextView::new(""))],
            playing: ResizedView::with_fixed_height(2, Playing::new()),

            selected: 0,
            input: None,
            mode: EventMode::Pass,
            last_tick: Instant::now(),
        }
    }

    fn pass_event(&mut self, e: Event) -> EventResult {
        match self.content[self.selected].on_event(e.clone()) {
            EventResult::Ignored => self.playing.on_event(e),
            r => r,
        }
    }
}

impl View for Root {
    // 'Root' is functionally a vertical linear layout
    fn draw(&self, printer: &cursive::Printer) {
        let mut title_printer = printer.clone();
        let mut content_printer = printer.clone();
        let mut playing_printer = printer.clone();
        title_printer.output_size.y = 2;
        title_printer.size.y = 2;

        content_printer.output_size.y -= 4;
        content_printer.size.y -= 4;
        content_printer.offset.y += 2;

        playing_printer.output_size.y = 2;
        playing_printer.size.y = 2;
        playing_printer.offset.y += title_printer.size.y + content_printer.size.y;

        self.titlebar.draw(&title_printer);
        self.content[self.selected].draw(&content_printer);
        self.playing.draw(&playing_printer);
    }

    fn layout(&mut self, size: XY<usize>) {
        // log!(Level::Debug, "Layout");
        self.titlebar.layout(XY { x: size.x, y: 2 });
        self.playing.layout(XY { x: size.x, y: 2 });
        self.content[self.selected].layout(XY {
            x: size.x,
            y: size.y - 4,
        })
    }

    fn needs_relayout(&self) -> bool {
        self.titlebar.needs_relayout()
            || self.content[self.selected].needs_relayout()
            || self.playing.needs_relayout()
    }

    fn call_on_any(&mut self, s: &cursive::view::Selector, e: cursive::event::AnyCb) {
        // log!(Level::Debug, "Call On Any");
        self.titlebar.call_on_any(s, e);
        self.content.iter_mut().for_each(|v| v.call_on_any(s, e));
        self.playing.call_on_any(s, e);
    }

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        XY {
            x: constraint.x,
            y: 4 + self
                .content
                .iter_mut()
                .map(|v| {
                    v.required_size(XY {
                        x: constraint.x,
                        y: constraint.y - 4, // reserve for title bar & playing
                    })
                    .y
                })
                .max()
                .unwrap_or(1),
        }
    }

    fn on_event(&mut self, e: Event) -> EventResult {
        // handle refresh event seperately
        if let Some(&Event::Refresh) = Some(&e) {
            let cur_time = Instant::now();
            let dt = cur_time.duration_since(self.last_tick);
            self.last_tick = cur_time;
            self.titlebar
                .get_inner_mut()
                .set_title(format!("{:.2} tps", 1.0 / dt.as_secs_f64()));

            if let Some(i) = &self.input {
                self.playing.get_inner_mut().lock_title(i.clone());
            } else {
                self.playing.get_inner_mut().unlock_title();
            }
            self.pass_event(e);
            return EventResult::Ignored; // Always mark refresh as ignored
        }

        // handle other events
        match self.mode {
            EventMode::Pass => match e {
                Event::Char(':') => {
                    self.input = Some(String::from(':'));
                    self.mode = EventMode::Input;
                    EventResult::Consumed(None)
                }
                Event::Char('q') => {
                    global_cursive().quit();
                    EventResult::Consumed(None)
                }

                Event::Char('1') => {
                    self.selected = 0;
                    EventResult::Consumed(None)
                }
                Event::Char('2') => {
                    self.selected = 1;
                    EventResult::Consumed(None)
                }
                _ => self.pass_event(e),
            },
            EventMode::Input => match e {
                Event::Char(c) => {
                    self.input.iter_mut().for_each(|s| s.push(c));
                    EventResult::Consumed(None)
                }
                Event::CtrlChar('c') => {
                    self.input.take();
                    self.mode = EventMode::Pass;
                    EventResult::Consumed(None)
                }
                Event::Key(Key::Del) | Event::Key(Key::Backspace) => {
                    self.input.iter_mut().for_each(|s| {
                        s.pop();
                    });
                    EventResult::Consumed(None)
                }
                Event::Key(Key::Enter) => {
                    let command = match &self.input {
                        Some(i) => i,
                        None => unreachable!(),
                    };
                    log!(Level::Info, "Command: {}", command);
                    self.input.take();
                    self.mode = EventMode::Pass;
                    EventResult::Consumed(None)
                }
                e => self.pass_event(e),
            },
        }
    }
}
