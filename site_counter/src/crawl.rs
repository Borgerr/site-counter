use num_bigint::BigUint;
use lazy_static::lazy_static;

use std::collections::HashMap;

type Url = String;

struct DfsState {
    pub visited: HashMap<Url, BigUint>, // maps URLs to the number of times they've been visited
    pub queue: Vec<Url>,    // queue of URLs to visit
}

lazy_static! {
    static ref DFSSTATE: DfsState = DfsState {
        visited: HashMap::new(),
        queue: Vec::new(),
    };
}

pub async fn run_dfs() {
    println!("Run_dfs...");
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
}

