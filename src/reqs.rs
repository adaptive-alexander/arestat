use std::collections::VecDeque;
use std::time::Duration;

use reqwest::{Client, Error};
use tokio::time::{Instant, sleep};

use crate::cli::HttpMethod;

pub async fn dispatch_requests(http_method: HttpMethod, requests: usize, req_rate: Option<u16>, threads: usize) -> Result<Vec<u128>, Error> {
    let client = Client::new();
    let mut results = VecDeque::with_capacity(requests);

    let req_call = match http_method {
        HttpMethod::Get(arg) => { client.get(&arg.url) }
        HttpMethod::Post(arg) => { client.post(&arg.url).json(&arg.body) }
        HttpMethod::Patch(arg) => { client.patch(&arg.url).json(&arg.body) }
        HttpMethod::Put(arg) => { client.patch(&arg.url).json(&arg.body) }
        HttpMethod::Delete(arg) => { client.get(&arg.url) }
    };

    match req_rate {
        None => {
            // Add one for warm-up
            for _ in 0..requests + 1 {
                let start = Instant::now();
                req_call.try_clone().expect("Failed cloning RequestBuilder, irrecoverable").send().await?;
                results.push_back(start.elapsed().as_nanos())
            }
        }
        Some(_) => {
            let mut limit_timer = Instant::now();
            for i in 0..requests {
                // Limit if i is not first or last (excluding warm-up), and is divisible by req_rate per thread
                if i % (req_rate.unwrap() as usize / threads) == 0 && i != 0 && i != requests - 1 {
                    // Only limit if less than a second has passed
                    if limit_timer.elapsed().as_secs() < 1 {
                        // Limit using blocking sleep
                        sleep(Duration::from_millis((1000 - limit_timer.elapsed().as_millis()) as u64)).await;
                        limit_timer = Instant::now();
                    }
                }
                let start = Instant::now();
                req_call.try_clone().expect("Failed cloning RequestBuilder, irrecoverable").send().await?;
                results.push_back(start.elapsed().as_nanos())
            }
        }
    }
    // Remove warm-up timer
    results.pop_front();
    Ok(results.into_iter().collect::<Vec<_>>())
}
