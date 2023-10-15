use clap::Parser;

use arestat::cli::Cli;
use arestat::run_requests;
use arestat::stats::Stats;

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    println!("Running requests...");

    let (total_time, timers) = run_requests(args.threads, args.requests, args.method).await;

    let stats = Stats::new(total_time, timers, args.requests);
    stats.print();
}
