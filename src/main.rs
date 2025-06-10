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
    /*
    let db_name = env::var("PG_DBNAME").unwrap();
    let db_host = env::var("PG_HOST").unwrap();
    let db_user = env::var("PG_USER").unwrap();
    let db_password = env::var("PG_PASSWORD").unwrap();
    */
    /*
    let url = env::var("START_URL")
        .expect("Crawlers need somewhere to start! Set this START_URL variable.");
    */
    let num_cpus = thread::available_parallelism().unwrap().get();
    let num_cpus = match env::var("NUM_CPUS") {
        Ok(s) => s.parse::<usize>().unwrap_or(num_cpus),
        _ => num_cpus,
    };

    /*
    let database_url = format!(
        "postgres://{}:{}@{}/{}",
        db_user, db_password, db_host, db_name
    );

    let pool = PgPoolOptions::new()
        .max_connections(num_cpus as u32)
        .connect(&database_url)
        .await?;
    */

    log("Successfully connected to database."
        .to_string()
        .green()
        .bold());
    //log(format!("Beginning search starting at URL: {}", url).green());

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

    // TODO: fetch all data and put into a zip file
    // probably want to put a cap on how much data can be in the db
    // then also want to emit some sort of file encoding what URLs are visited and how often
    // probably not more complicated than simply writing a json string
}
