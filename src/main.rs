use clap::Parser;

use arestat::cli::Cli;
use arestat::ReqRunner;
use arestat::stats::Stats;

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    println!("Running requests...");

    println!("Requests: {}", args.requests);
    println!("Threads: {}", args.threads);
    if args.req_rate.is_some() {
        println!("Requests per second: {}", args.req_rate.unwrap());
    }

    let mut req_runner = ReqRunner::new(args.threads, args.requests, args.method);

    if args.req_rate.is_some() {
        req_runner.with_req_rate(args.req_rate)
    }

    let (total_time, timers) = req_runner.run_requests().await;

    let stats = Stats::new(total_time, timers, args.requests);
    stats.print();
}
