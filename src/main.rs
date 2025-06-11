use clap::Parser;
use colored::{ColoredString, Colorize};

use std::{env, thread};

mod crawl;
use crawl::{DfsState, run_dfs};

fn log(s: ColoredString) {
    println!("LOG: {}", s)
}

#[derive(Parser)]
#[command(version("0.1.0"), about = "A webscraper", long_about = None)]
struct Cli {
    #[clap(
        short,
        action,
        help = "Give verbose output at runtime about which URLs are visited, whether or not responses were received, etc"
    )]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let url = env::var("START_URL")
        .expect("Crawlers need somewhere to start! Set this START_URL variable.");
    let num_workers = thread::available_parallelism().unwrap().get();
    let num_workers = match env::var("NUM_WORKERS") {
        Ok(s) => s.parse::<usize>().unwrap_or(num_workers),
        _ => num_workers,
    };

    log("Successfully connected to database."
        .to_string()
        .green()
        .bold());

    let mut tasks = Vec::with_capacity(num_workers);
    let mut state = DfsState::new();
    state.append_url(url);
    for _ in 0..num_workers {
        let threads_state = state.clone();
        tasks.push(tokio::spawn(async move {
            run_dfs(threads_state).await;
        }));
    }
    for task in tasks {
        task.await.unwrap();
    }

    // TODO: fetch all data and put into a zip file
    // probably want to put a cap on how much data can be in the db
    // then also want to emit some sort of file encoding what URLs are visited and how often
    // probably not more complicated than simply writing a json string
}
