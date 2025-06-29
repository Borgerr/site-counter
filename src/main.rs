use clap::Parser;
use fs_extra::dir::get_size;
use lazy_static::lazy_static;
use serde_json::json;
use tempfile::{TempDir, tempdir};
use tokio::{
    task::JoinHandle,
    time::{Duration, sleep},
};

// taken from zip-rs examples
use anyhow::Context;
use walkdir::{DirEntry, WalkDir};
use zip::{result::ZipError, write::SimpleFileOptions};

use std::{
    fs::File,
    io::{Read, Seek, Write},
    path::{Path, PathBuf},
    thread,
};

mod crawl;
use crawl::{DfsState, Worker};

lazy_static! {
    pub static ref TEMPDIR: TempDir =
        tempdir().expect("couldn't instantiate a temporary directory");
}

type Url = String; // Rust probably has a better type for this

#[derive(Parser)]
#[command(version("0.1.0"), about = "A webscraper that fetches websites and counts their references from others visited, and outputs all this data into an archive.", long_about = None)]
struct Args {
    #[arg(help = "URL to start off with. Must include protocol, URL, and any optional path.")]
    start_url: Url,
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
        help = "Maximum size of the produced archive, in KB."
    )]
    tmpfs_size: Option<u64>,
    #[clap(
        short,
        long,
        value_name = "DESTINATION_ZIPFILE",
        help = "Where to place the result archive."
    )]
    destination: Option<PathBuf>,
    #[clap(
        long,
        short,
        action,
        help = "Crawling algorithm is breadth first instead of depth first when set."
    )]
    is_bfs: bool,
    #[clap(
        short,
        action,
        help = "Give verbose output at runtime about which URLs are visited, whether or not responses were received, etc."
    )]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let verbosity = args.verbose;

    let start_url = args.start_url;
    let num_workers = thread::available_parallelism().unwrap().get();
    let num_workers = args.num_workers.unwrap_or(num_workers);
    let tmpfs_size = args.tmpfs_size.unwrap_or(4) << 10; // << 10 -> KiB
    let dst_file = args.destination.unwrap_or("./archive.xz".into());
    let is_bfs = args.is_bfs;

    verbosity.then(|| println!("TEMPDIR is {}", TEMPDIR.path().display()));

    let mut tasks = Vec::with_capacity(num_workers);
    let mut state = DfsState::new(num_workers, is_bfs);
    state.append_url(start_url, verbosity);

    tokio::select!(
        _ = wait_for_size(tmpfs_size, verbosity) => (),
        _ = wait_for_workers(num_workers, state.clone(), &mut tasks, verbosity) => ()
    );
    for task in &tasks {
        // TODO: should probably abort more gracefully
        verbosity.then(|| println!("Killing tasks..."));
        task.abort();
    }

    let stats_path = TEMPDIR.path().join("stats.json");
    let mut stats_file = File::create(stats_path).unwrap();
    write!(stats_file, "{}", json!(*state.visited));

    // TODO: zip TEMPDIR and put into an accessible archive
    doit(TEMPDIR.path(), &dst_file, zip::CompressionMethod::Xz).unwrap();
}

fn currentsize_tmpfs() -> u64 {
    get_size(TEMPDIR.path()).unwrap()
}

async fn wait_for_workers(
    num_workers: usize,
    state: DfsState,
    tasks: &mut Vec<JoinHandle<()>>,
    verbosity: bool,
) {
    for _ in 0..num_workers {
        let worker = Worker::new(state.clone(), verbosity);
        tasks.push(tokio::spawn(async move {
            worker.crawl().await;
        }));
    }
    for task in tasks {
        task.await.unwrap();
    }
    verbosity.then(|| println!("returning from wait_for_workers..."));
}

async fn wait_for_size(tmpfs_size: u64, verbosity: bool) {
    loop {
        verbosity.then(|| {
            println!(
                "tmpfs_size: {}, currentsize_tmpfs: {}",
                tmpfs_size,
                currentsize_tmpfs()
            )
        });
        if currentsize_tmpfs() >= tmpfs_size {
            break;
        }
        sleep(Duration::from_millis(200)).await;
    }
    verbosity.then(|| println!("returning from wait_for_size..."));
}

// compression taken from zip-rs examples
// https://github.com/zip-rs/zip2/blob/0a40f7183959e311b3131d3e1f7392d166c26b2a/examples/write_dir.rs

fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &Path,
    writer: T,
    method: zip::CompressionMethod,
) -> anyhow::Result<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = SimpleFileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let prefix = Path::new(prefix);
    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(prefix).unwrap();
        let path_as_string = name
            .to_str()
            .map(str::to_owned)
            .with_context(|| format!("{name:?} Is a Non UTF-8 Path"))?;

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            println!("adding file {path:?} as {name:?} ...");
            zip.start_file(path_as_string, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            println!("adding dir {path_as_string:?} as {name:?} ...");
            zip.add_directory(path_as_string, options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

fn doit(src_dir: &Path, dst_file: &Path, method: zip::CompressionMethod) -> anyhow::Result<()> {
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound.into());
    }

    let path = Path::new(dst_file);
    let file = File::create(path).unwrap();

    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter();

    zip_dir(&mut it.filter_map(|e| e.ok()), src_dir, file, method)?;

    Ok(())
}
