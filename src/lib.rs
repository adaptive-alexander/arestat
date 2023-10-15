use tokio::time::Instant;

use crate::cli::HttpMethod;
use crate::reqs::dispatch_requests;
use crate::stats::chunk_reqs;

pub mod reqs;
pub mod stats;
pub mod cli;

pub async fn run_requests(threads: usize, requests: usize, method: HttpMethod) -> (u128, Vec<u128>) {
    let mut tasks = Vec::with_capacity(threads);
    let start = Instant::now();
    for i in 0..threads {
        let reqs = chunk_reqs(requests, threads, i);

        // --- Spawn tasks
        tasks.push(tokio::spawn(dispatch_requests(method.clone(), reqs)))
    }

    // Await to make sure tasks complete
    let mut timers = Vec::with_capacity(requests);
    for task in tasks {
        timers.push(task.await.unwrap().unwrap());
    }

    // Output
    let timers = timers.into_iter().flatten().collect::<Vec<u128>>();
    (start.elapsed().as_nanos(), timers)
}
