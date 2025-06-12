use clap::Parser;
use colored::{ColoredString, Colorize};
use lazy_static::lazy_static;
use tempfile::{TempDir, tempdir};

use tokio::time::{sleep, Duration};

use std::thread;

mod crawl;
use crawl::{DfsState, run_dfs};

lazy_static! {
    pub static ref TEMPDIR: TempDir = tempdir().unwrap();
}

fn log(s: ColoredString) {
    println!("LOG: {}", s)
}

type Url = String; // Rust probably has a better type for this

#[derive(Parser)]
#[command(version("0.1.0"), about = "A webscraper", long_about = None)]
struct Args {
    #[arg(help = "URL to start off with. Must include protocol, URL, and any optional path.")]
    start_url: Url,
    #[clap(
        short,
        action,
        help = "Give verbose output at runtime about which URLs are visited, whether or not responses were received, etc"
    )]
    verbose: bool,
    #[clap(
        short,
        long,
        value_name = "NUM_WORKERS",
        help = "Number of maximum worker threads."
    )]
    num_workers: Option<usize>,
    #[clap(
        short,
        long,
        value_name = "ARCHIVE_SIZE",
        help = "Maximum size of the produced archive, in MB."
    )]
    tmpfs_size: Option<usize>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let verbosity = args.verbose;

    let start_url = args.start_url;
    let num_workers = thread::available_parallelism().unwrap().get();
    let num_workers = args.num_workers.unwrap_or(num_workers);
    let tmpfs_size = args.tmpfs_size.unwrap_or(200);

    // TODO: setup background task checking for tmpfs size
    let _ = tokio::task::spawn(async {
        println!("waiting 2 seconds...");
        sleep(Duration::from_millis(200)).await;
    });

    /*
    let mut tasks = Vec::with_capacity(num_workers);
    let mut state = DfsState::new();
    state.append_url(start_url, verbosity);
    for _ in 0..num_workers {
        let threads_state = state.clone();
        tasks.push(tokio::spawn(async move {
            run_dfs(threads_state, verbosity).await;
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
