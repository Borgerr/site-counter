use colored::{ColoredString, Colorize};
//use sqlx::postgres::PgPoolOptions;

use std::{env, thread};

mod crawl;
use crawl::{DfsState, run_dfs};

fn log(s: ColoredString) {
    println!("LOG: {}", s)
}

#[tokio::main]
async fn main() {
    let url = env::var("START_URL")
        .expect("Crawlers need somewhere to start! Set this START_URL variable.");
    let num_cpus = thread::available_parallelism().unwrap().get();
    let num_cpus = match env::var("NUM_CPUS") {
        Ok(s) => s.parse::<usize>().unwrap_or(num_cpus),
        _ => num_cpus,
    };

    log("Successfully connected to database."
        .to_string()
        .green()
        .bold());

    /*
    let mut tasks = Vec::with_capacity(num_cpus);
    let state = DfsState::new();
    for _ in 0..num_cpus {
        let threads_state = state.clone();
        tasks.push(tokio::spawn(async move {
            run_dfs(threads_state).await;
        }));
    }
    for task in tasks {
        task.await.unwrap();
    }
    */

    // TODO: fetch all data and put into a zip file
    // probably want to put a cap on how much data can be in the db
    // then also want to emit some sort of file encoding what URLs are visited and how often
    // probably not more complicated than simply writing a json string
}
