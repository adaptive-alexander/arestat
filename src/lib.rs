use crate::cli::{HttpHeaders, HttpMethod};
use crate::reqs::dispatch_requests;
use crate::stats::chunk_reqs;

pub mod cli;
pub mod reqs;
pub mod stats;

pub struct ReqRunner {
    threads: usize,
    requests: usize,
    method: HttpMethod,
    headers: Option<HttpHeaders>,
    req_rate: Option<u16>,
}

impl ReqRunner {
    pub fn new(
        threads: usize,
        requests: usize,
        method: HttpMethod,
        headers: Option<HttpHeaders>,
    ) -> Self {
        Self {
            threads,
            requests,
            method,
            headers,
            req_rate: None,
        }
    }
    pub fn with_req_rate(&mut self, req_rate: Option<u16>) {
        self.req_rate = req_rate;
    }
    pub async fn run_requests(self) -> (u128, Vec<u128>) {
        let mut tasks = Vec::with_capacity(self.threads);

        // --- Spawn tasks
        for i in 0..self.threads {
            let reqs = chunk_reqs(self.requests, self.threads, i);
            tasks.push(tokio::spawn(dispatch_requests(
                self.method.clone(),
                self.headers.clone(),
                reqs,
                self.req_rate,
                self.threads,
            )))
        }

        // Await to make sure tasks complete
        let mut timers = Vec::with_capacity(self.requests);
        for task in tasks {
            timers.push(task.await.unwrap().unwrap());
        }

        // Output
        let total_time = timers.iter().map(|v| v.iter().sum()).max().unwrap();
        let timers = timers.into_iter().flatten().collect::<Vec<u128>>();
        (total_time, timers)
    }
}
