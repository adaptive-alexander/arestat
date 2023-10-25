use std::collections::VecDeque;
use std::time::Duration;

use reqwest::{Client, Error};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use tokio::time::{Instant, sleep};

use crate::cli::{HttpHeaders, HttpMethod};

pub async fn dispatch_requests(http_method: HttpMethod, headers: Option<HttpHeaders>, requests: usize, req_rate: Option<u16>, threads: usize) -> Result<Vec<u128>, Error> {
    let client = Client::new();
    let mut results = VecDeque::with_capacity(requests);

    let mut header_map = HeaderMap::new();

    if let Some(headers) = headers {
        for (h, v) in headers.0 {
            header_map.insert(h.parse::<HeaderName>().expect("Failed converting key to HeaderName"),
                              v.parse::<HeaderValue>().expect("Failed converting value to a HeaderValue"));
        }
    }

    let req_call = match http_method {
        HttpMethod::Get(arg) => { client.get(&arg.url).headers(header_map) }
        HttpMethod::Post(arg) => { client.post(&arg.url).json(&arg.body).headers(header_map) }
        HttpMethod::Patch(arg) => { client.patch(&arg.url).json(&arg.body).headers(header_map) }
        HttpMethod::Put(arg) => { client.patch(&arg.url).json(&arg.body).headers(header_map) }
        HttpMethod::Delete(arg) => { client.get(&arg.url).headers(header_map) }
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
