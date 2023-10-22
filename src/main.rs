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

    let mut req_runner = ReqRunner::new(args.threads, args.requests, args.method);

    if let Some(req_rate) = args.req_rate {
        println!("Requests per second: {}", req_rate);
        req_runner.with_req_rate(args.req_rate);
    }

    let (total_time, timers) = req_runner.run_requests().await;

    let stats = Stats::new(total_time, timers, args.requests);
    stats.pretty_print();
}
