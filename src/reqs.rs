use std::time::Duration;

use reqwest::{Client, Error};
use tokio::time::{Instant, sleep};

use crate::cli::HttpMethod;

pub async fn dispatch_requests(http_method: HttpMethod, requests: usize, req_rate: Option<u16>, threads: usize) -> Result<Vec<u128>, Error> {
    let client = Client::new();
    let mut results = Vec::with_capacity(requests);

    // Add one for warm-up
    let requests = requests + 1;
    let mut limit_timer = Instant::now();

    let req_call = match http_method {
        HttpMethod::Get(arg) => { client.get(&arg.url) }
        HttpMethod::Post(arg) => { client.post(&arg.url).json(&arg.body) }
        HttpMethod::Patch(arg) => { client.patch(&arg.url).json(&arg.body) }
        HttpMethod::Put(arg) => { client.patch(&arg.url).json(&arg.body) }
        HttpMethod::Delete(arg) => { client.get(&arg.url) }
    };

    match req_rate {
        None => {
            for _ in 0..requests {
                let start = Instant::now();
                req_call.try_clone().expect("Failed cloning RequestBuilder, irrecoverable").send().await?;
                results.push(start.elapsed().as_nanos())
            }
        }
        Some(_) => {
            for i in 0..requests {
                // Limit if i is not first or last (excluding warm-up), and is divisible by req_rate per thread
                if i % (req_rate.unwrap() as usize / threads) == 0 && i != 0 && i != requests - 1 {
                    if limit_timer.elapsed().as_secs() < 1 {
                        // Limit using blocking sleep
                        sleep(Duration::from_millis((1000 - limit_timer.elapsed().as_millis()) as u64)).await;
                        limit_timer = Instant::now();
                    }
                }
                let start = Instant::now();
                req_call.try_clone().expect("Failed cloning RequestBuilder, irrecoverable").send().await?;
                results.push(start.elapsed().as_nanos())
            }
        }
    }
    // Remove warm-up timer
    Ok(results[1..].to_owned())
}
