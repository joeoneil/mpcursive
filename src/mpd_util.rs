#![allow(unused)]

use anyhow::Result;
use lazy_static::lazy_static;
use log::{log, Level};
use mpd::{Client, Song, State, Status};

use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
    time::{Duration, Instant},
};

use crate::view::playing::Playing;

lazy_static! {
    static ref CLIENT: RwLock<Client> = RwLock::new(Client::connect("127.0.0.1:6600").unwrap());
    static ref CACHE: RwLock<Cache> = RwLock::new(Cache::new());
}

struct Cache {
    queue: CacheItem<Vec<Song>>,
    status: CacheItem<Status>,
}

impl Cache {
    fn new() -> Self {
        Self {
            queue: CacheItem::new(
                Duration::from_millis(5000),
                Box::new(|| CLIENT.write().unwrap().queue().ok()),
                String::from("Queue"),
            ),
            status: CacheItem::new(
                Duration::from_millis(1000),
                Box::new(|| CLIENT.write().unwrap().status().ok()),
                String::from("Status"),
            ),
        }
    }
}

struct CacheItem<T>
where
    T: Clone,
{
    data: Option<T>,
    fetched: Instant,
    ttl: Duration,
    fetch: Box<dyn Fn() -> Option<T> + Send + Sync>,
    debug_name: String,
}

impl<T> CacheItem<T>
where
    T: Clone,
{
    fn new(
        ttl: Duration,
        fetch: Box<dyn Fn() -> Option<T> + Send + Sync>,
        debug_name: String,
    ) -> Self {
        let data = fetch();
        let fetched = Instant::now();
        Self {
            data,
            fetched,
            ttl,
            fetch,
            debug_name,
        }
    }

    fn expired(&self) -> bool {
        Instant::now().duration_since(self.fetched) > self.ttl
    }

    fn update_get(&mut self) -> Option<&T> {
        if self.expired() || self.data.is_none() {
            self.update();
        }
        self.data.as_ref()
    }

    fn get(&self) -> Option<&T> {
        if self.expired() {
            None
        } else {
            self.data.as_ref()
        }
    }

    fn update(&mut self) {
        if let Some(d) = (self.fetch)() {
            self.data = Some(d);
            self.fetched = Instant::now();
        } else {
            log!(
                Level::Warn,
                "Failed to update cache for {}",
                self.debug_name
            );
        }
    }

    fn invalidate(&mut self) {
        self.data = None;
    }
}

pub struct MPD;

impl MPD {
    pub fn queue() -> Option<Vec<Song>> {
        let mut cache = CACHE.write().unwrap();
        cache.queue.update_get().cloned()
    }

    pub fn status() -> Option<Status> {
        let mut cache = CACHE.write().unwrap();
        cache.status.update_get().cloned()
    }

    pub fn now_playing() -> Option<Song> {
        let q = MPD::queue()?;
        let s = MPD::status()?;
        Some(q[s.song?.pos as usize].clone())
    }

    pub fn elapsed() -> Option<Duration> {
        let mut cache = CACHE.write().unwrap();
        let s = cache.status.update_get()?;
        Some(match s.state {
            State::Play => (s.elapsed? + Instant::now().duration_since(cache.status.fetched)),
            _ => s.elapsed?,
        })
    }

    pub fn current_time() -> Option<(Duration, Duration)> {
        MPD::status()?.time
    }

    pub fn set_repeat(repeat: bool) -> Result<()> {
        CACHE.write().unwrap().status.invalidate();
        Ok(CLIENT.write().unwrap().repeat(repeat)?)
    }
}
