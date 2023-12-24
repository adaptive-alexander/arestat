use std::cmp::min;
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Error};
use tokio::time::MissedTickBehavior::Delay;
use tokio::time::{interval, Instant};
use crate::BasicAuth;

use crate::cli::{HttpHeaders, HttpMethod};

pub async fn dispatch_requests(
    http_method: HttpMethod,
    headers: Option<HttpHeaders>,
    requests: usize,
    req_rate: Option<u16>,
    threads: usize,
    auth: Option<BasicAuth>,
) -> Result<Vec<u128>, Error> {
    let client = Client::new();
    let mut results = Vec::with_capacity(requests);

    let mut header_map = HeaderMap::new();

    if let Some(headers) = headers {
        for (h, v) in headers.0 {
            header_map.insert(
                h.parse::<HeaderName>()
                    .expect("Failed converting key to HeaderName"),
                v.parse::<HeaderValue>()
                    .expect("Failed converting value to a HeaderValue"),
            );
        }
    }

    // todo!("Remake into a macro")
    let req_call = match (http_method, auth.is_some()) {
        (HttpMethod::Get(arg), false) => client.get(&arg.url),
        (HttpMethod::Post(arg), false) => client.post(&arg.url).json(&arg.body),
        (HttpMethod::Patch(arg), false) => client.patch(&arg.url).json(&arg.body),
        (HttpMethod::Put(arg), false) => client.patch(&arg.url).json(&arg.body),
        (HttpMethod::Delete(arg), false) => client.get(&arg.url),
        (HttpMethod::Get(arg), true) => client.get(&arg.url).basic_auth(auth.as_ref().unwrap().username.clone(), auth.as_ref().unwrap().password.clone()),
        (HttpMethod::Post(arg), true) => client.post(&arg.url).basic_auth(auth.as_ref().unwrap().username.clone(), auth.as_ref().unwrap().password.clone()),
        (HttpMethod::Patch(arg), true) => client.patch(&arg.url).json(&arg.body).basic_auth(auth.as_ref().unwrap().username.clone(), auth.as_ref().unwrap().password.clone()),
        (HttpMethod::Put(arg), true) => client.patch(&arg.url).json(&arg.body).basic_auth(auth.as_ref().unwrap().username.clone(), auth.as_ref().unwrap().password.clone()),
        (HttpMethod::Delete(arg), true) => client.get(&arg.url).basic_auth(auth.as_ref().unwrap().username.clone(), auth.as_ref().unwrap().password.clone()),
    };

    let req_call = req_call.headers(header_map);

    req_call
        .try_clone()
        .expect("Failed cloning RequestBuilder, irrecoverable")
        .send()
        .await?;

    match req_rate {
        None => {
            for _ in 0..requests {
                let start = Instant::now();
                req_call
                    .try_clone()
                    .expect("Failed cloning RequestBuilder, irrecoverable")
                    .send()
                    .await?;
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
                    req_call
                        .try_clone()
                        .expect("Failed cloning RequestBuilder, irrecoverable")
                        .send()
                        .await?;
                    results.push(start.elapsed().as_nanos())
                }
                reqs_run += req_rate_thread;
                if reqs_run >= requests {
                    break;
                }
            }
        }
    }
    Ok(results)
}
