use dashmap::DashMap;
use lazy_static::lazy_static;
use num_bigint::BigUint;
use num_traits::One;
use tempfile::{TempDir, tempdir};

use std::sync::{Arc, Mutex};
use std::thread;

type Url = String;

#[derive(Clone)]
pub struct DfsState {
    pub visited: DashMap<Url, BigUint>, // maps URLs to the number of times they've been visited
    pub queue: Arc<Mutex<Vec<Url>>>,    // queue of URLs to visit
    pub working_threads: Arc<Mutex<usize>>, // number of threads currently fetching something
}

impl DfsState {
    pub fn new() -> Self {
        DfsState {
            visited: DashMap::new(),
            queue: Arc::new(Mutex::new(Vec::new())),
            working_threads: Arc::new(Mutex::new(thread::available_parallelism().unwrap().get())),
        }
    }
    pub fn append_url(&mut self, url: Url) {
        self.queue.lock().unwrap().push(url)
    }
    pub fn get_url(&mut self) -> Option<Url> {
        self.queue.lock().unwrap().pop()
    }
    pub fn get_working_threads(&mut self) -> usize {
        *self.working_threads.lock().unwrap()
    }
    pub fn decrement_working_threads(&mut self) {
        *self.working_threads.lock().unwrap() -= 1
    }
    pub fn increment_working_threads(&mut self) {
        *self.working_threads.lock().unwrap() += 1
    }
    pub fn increment_value(&mut self, key: Url) {
        self.visited.entry(key).and_modify(|v| *v += BigUint::one()).or_insert(BigUint::one());
    }
}

lazy_static! {
    static ref TEMPDIR: TempDir = tempdir().unwrap();
}

pub async fn run_dfs(mut dfs_state: DfsState) {
    assert!(TEMPDIR.path().exists());
    loop {
        let url_res = dfs_state.get_url();
        match url_res {
            Some(url) => {
                dfs_state.increment_working_threads();
                if dfs_state.visited.contains_key(&url) {
                    // increment and don't fetch
                    dfs_state.increment_value(url)
                } else {
                    // fetch page, extract URLs, and move on
                    fetch_and_extract(url, &mut dfs_state).await
                }
            },
            _ => {
                dfs_state.decrement_working_threads();
                if dfs_state.get_working_threads() == 0 {
                    return;
                }
            },
        }
    }
}

async fn fetch_and_extract(url: Url, dfs_state: &mut DfsState) {
    todo!("fetch webpage, place into tmpfs, and return extracted URLs")
}

