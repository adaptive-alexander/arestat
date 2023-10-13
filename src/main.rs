use clap::Parser;
use tokio::time::Instant;

use arestat::cli::Cli;
use arestat::reqs::run_requests;
use arestat::stats::{chunk_reqs, print_stats};

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let mut tasks = Vec::with_capacity(args.threads);
    let start = Instant::now();
    for i in 0..args.threads {
        let reqs = chunk_reqs(args.requests, args.threads, i);

        // --- Spawn tasks
        tasks.push(tokio::spawn(run_requests(args.method.clone(), reqs)))
    }

    // Await to make sure tasks complete
    let mut timers = Vec::with_capacity(args.requests);
    for task in tasks {
        timers.push(task.await.unwrap().unwrap());
    }

    // Output
    let timers = timers.into_iter().flatten().collect::<Vec<u128>>();
    print_stats(start.elapsed().as_nanos(), timers, args.requests);
}
