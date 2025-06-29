use dashmap::DashMap;
use lazy_static::lazy_static;
use num_bigint::BigUint;
use num_traits::One;
use regex::Regex;

use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use std::{fs::File, io::Write};

use super::TEMPDIR;
use super::Url;

lazy_static! {
    static ref URL_RE: Regex =
        Regex::new(r"http[s]?:\/\/.(?:www\.)?[-a-zA-Z0-9@%._\+~#=]{2,256}\.[a-z]{2,6}\b(?:[-a-zA-Z0-9@:%_\+.~\/\/]*)")
            .unwrap();
    static ref PROTOCOL_RE: Regex = Regex::new(r"https?:\/\/").unwrap();
}

#[derive(Clone)]
pub struct DfsState {
    pub visited: Arc<DashMap<Url, BigUint>>, // maps URLs to the number of times they've been visited
    pub queue: Arc<Mutex<VecDeque<Url>>>,    // queue of URLs to visit
    pub working_threads: Arc<Mutex<usize>>,  // number of threads currently fetching something
    is_bfs: bool,
}

impl DfsState {
    pub fn new(num_workers: usize, is_bfs: bool) -> Self {
        DfsState {
            visited: Arc::new(DashMap::new()),
            queue: Arc::new(Mutex::new(VecDeque::new())),
            working_threads: Arc::new(Mutex::new(num_workers)),
            is_bfs,
        }
    }
    pub fn append_url(&mut self, url: Url, verbosity: bool) {
        if URL_RE.is_match(&url) {
            verbosity.then(|| println!("added URL: {}", url));
            self.queue.lock().push_back(url)
        } else {
            verbosity.then(|| println!("didn't add URL: {}", url));
        }
    }
    pub fn get_url(&mut self) -> Option<Url> {
        if self.is_bfs {
            self.queue.lock().pop_front()
        } else {
            self.queue.lock().pop_back()
        }
    }
    pub fn get_active_workers(&mut self) -> usize {
        *self.working_threads.lock()
    }
    pub fn mark_worker_inactive(&mut self) -> usize {
        let mut count = self.working_threads.lock();
        *count -= 1;
        *count
    }
    pub fn mark_worker_active(&mut self) {
        let mut count = self.working_threads.lock();
        *count += 1;
    }
    pub fn try_claim(&mut self, key: Url) -> bool {
        // TODO:
        // probably just want to keep the DfsState more simple
        // as in, a map of Url to bool, and keep the counter in the worker thread
        // just because we have that eventual idea of keeping this all generic for other things
        (*self // or_insert() returns a RefMut
            .visited
            .entry(key) // `entry` always consumes key.
            // in case of deadlocks, maybe change this entry grab to something else
            .and_modify(|val| *val += BigUint::one())
            .or_insert(BigUint::one()))
            == BigUint::one() // if the RefMut is == 1, worker claims as fetch
    }
}

pub struct Worker {
    state: DfsState,
    verbosity: bool,
    is_active: bool,
    // TODO: do we want a worker ID?
    // does tokio have a better style guide for this?
}

impl Worker {
    pub fn new(state: DfsState, verbosity: bool) -> Self {
        Self {
            state,
            verbosity,
            is_active: true,
        }
    }
    fn check_activity(&mut self) -> usize {
        if self.is_active {
            self.is_active = false;
            self.state.mark_worker_inactive()
        } else {
            self.state.get_active_workers()
        }
    }
    fn set_active(&mut self) {
        if !self.is_active {
            self.is_active = true;
            self.state.mark_worker_active()
        }
    }
    pub async fn crawl(mut self) {
        assert!(TEMPDIR.path().exists()); // TODO: is this necessary?
        loop {
            self.verbosity.then(|| println!("crawl loop..."));
            let url_res = self.state.get_url();
            match url_res {
                Some(url) => {
                    self.set_active();
                    self.verbosity
                        .then(|| println!("investigating url {}", url));
                    // TODO: revisit
                    // we need to ensure there's only one operation where we check if we've visited a
                    // site, otherwise we have multiple workers thinking they're the first
                    if self.state.try_claim(url.clone()) {
                        self.new_visit(url).await;
                    } else {
                        self.prev_visit(url).await;
                    }
                }
                _ => {
                    let working_threads = self.check_activity();
                    self.verbosity
                        .then(|| println!("working threads: {}", working_threads));
                    if working_threads == 0 {
                        self.verbosity.then(|| println!("returning from crawl..."));
                        return;
                    }
                }
            }
        }
    }

    async fn prev_visit(&mut self, url: Url) -> reqwest::Result<()> {
        // basically just functions as logging
        // crawl loop has already incremented count
        self.verbosity.then(|| println!("prev_visit, url: {}", url));
        Ok(())
    }

    async fn new_visit(&mut self, url: Url) -> reqwest::Result<()> {
        self.verbosity.then(|| println!("new_visit, url: {}", url));

        // TODO: this always just returns depending on if the GET request fails
        // do we want to add an option for when this fails?
        let resp = reqwest::get(url.clone()).await?.text().await?;
        let protocol = PROTOCOL_RE
            .find(url.as_str())
            .expect("URL_RE guarantees PROTOCOL_RE matches on url param. Investigate.");
        let file_path_str: Url = url
            .trim_start_matches(protocol.as_str())
            .chars()
            .map(|c| if c == '/' { '-' } else { c })
            .take(200) // some greedy bastards at google are eating up all my filename space
            .collect();
        let file_path = TEMPDIR.path().join(format!("{}.html", file_path_str));

        println!("DEBUG PRINT: file_path: {}", file_path.display());
        let mut file = File::create(file_path).unwrap();
        write!(file, "{}", resp).unwrap();

        let state = self.state.clone();
        URL_RE
            .find_iter(&resp)
            .map(|extracted_url| extracted_url.as_str().to_string())
            .filter(|extracted_url| !state.visited.contains_key(extracted_url))
            .for_each(|extracted_url| {
                let verbosity = self.verbosity;
                self.state.append_url(extracted_url.clone(), verbosity);
            });

        Ok(())
    }
}
