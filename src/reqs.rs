use std::cmp::min;
use std::time::Duration;

use reqwest::{Client, Error};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use tokio::time::{Instant, interval};
use tokio::time::MissedTickBehavior::Delay;

use crate::cli::{HttpHeaders, HttpMethod};

pub async fn dispatch_requests(http_method: HttpMethod, headers: Option<HttpHeaders>, requests: usize, req_rate: Option<u16>, threads: usize) -> Result<Vec<u128>, Error> {
    let client = Client::new();
    let mut results = Vec::with_capacity(requests);

    let mut header_map = HeaderMap::new();

    if let Some(headers) = headers {
        for (h, v) in headers.0 {
            header_map.insert(h.parse::<HeaderName>().expect("Failed converting key to HeaderName"),
                              v.parse::<HeaderValue>().expect("Failed converting value to a HeaderValue"));
        }
    }

    let req_call = match http_method {
        HttpMethod::Get(arg) => { client.get(&arg.url) }
        HttpMethod::Post(arg) => { client.post(&arg.url).json(&arg.body) }
        HttpMethod::Patch(arg) => { client.patch(&arg.url).json(&arg.body) }
        HttpMethod::Put(arg) => { client.patch(&arg.url).json(&arg.body) }
        HttpMethod::Delete(arg) => { client.get(&arg.url) }
    };

    let req_call = req_call.headers(header_map);

    req_call.try_clone().expect("Failed cloning RequestBuilder, irrecoverable").send().await?;

    match req_rate {
        None => {
            for _ in 0..requests {
                let start = Instant::now();
                req_call.try_clone().expect("Failed cloning RequestBuilder, irrecoverable").send().await?;
                results.push(start.elapsed().as_nanos())
            }
        }
        Some(_) => {
            let mut interval = interval(Duration::from_secs(1));
            interval.set_missed_tick_behavior(Delay);

            let req_rate_thread = req_rate.unwrap() as usize / threads;
            let mut reqs_run = 0;

            loop {
                interval.tick().await;
                let batch = min(req_rate_thread, requests - reqs_run);
                for _ in 0..batch {
                    let start = Instant::now();
                    req_call.try_clone().expect("Failed cloning RequestBuilder, irrecoverable").send().await?;
                    results.push(start.elapsed().as_nanos())
                }
                reqs_run += req_rate_thread;
                if reqs_run >= requests {
                    break;
                }
            }
        }
    }
    // Remove warm-up timer
    Ok(results)
}
