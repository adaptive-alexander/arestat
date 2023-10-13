use reqwest::{Client, Error};
use tokio::time::Instant;

use crate::cli::HttpMethod;

pub async fn run_requests(http_method: HttpMethod, requests: usize) -> Result<Vec<u128>, Error> {
    let client = Client::new();
    let mut results = Vec::with_capacity(requests);
    let requests = requests + 1;
    match http_method {
        HttpMethod::Get(args) => {
            for _ in 0..requests {
                let start = Instant::now();
                client.get(&args.url).send().await?;
                results.push(start.elapsed().as_nanos())
            }
        }
        HttpMethod::Post(args) => {
            for _ in 0..requests {
                let start = Instant::now();
                client.post(&args.url).json(&args.body).send().await?;
                results.push(start.elapsed().as_nanos())
            }
        }
        HttpMethod::Patch(args) => {
            for _ in 0..requests {
                let start = Instant::now();
                client.patch(&args.url).json(&args.body).send().await?;
                results.push(start.elapsed().as_nanos())
            }
        }
        HttpMethod::Put(args) => {
            for _ in 0..requests {
                let start = Instant::now();
                client.put(&args.url).json(&args.body).send().await?;
                results.push(start.elapsed().as_nanos())
            }
        }
        HttpMethod::Delete(args) => {
            for _ in 0..requests {
                let start = Instant::now();
                client.delete(&args.url).send().await?;
                results.push(start.elapsed().as_nanos())
            }
        }
    };
    Ok(results[1..].to_owned())
}
