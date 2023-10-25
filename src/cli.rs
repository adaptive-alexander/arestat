use std::collections::HashMap;
use std::str::FromStr;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Clone)]
pub struct HttpHeaders(pub(crate) HashMap<String, String>);

impl FromStr for HttpHeaders {
    type Err = clap::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut map = HashMap::new();

        for pair in s.split(' ') {
            let parts: Vec<&str> = pair.split(':').collect();
            if parts.len() != 2 {
                return Err(clap::Error::new(clap::error::ErrorKind::InvalidValue));
            }

            map.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
        }

        Ok(Self(map))
    }
}

#[derive(Args, Debug, Clone)]
pub struct HttpArg {
    pub url: String,
}

#[derive(Args, Debug, Clone)]
pub struct HttpArgBody {
    pub url: String,
    pub body: String,
}

#[derive(Subcommand, Debug, Clone)]
pub enum HttpMethod {
    Get(HttpArg),
    Post(HttpArgBody),
    Patch(HttpArgBody),
    Put(HttpArgBody),
    Delete(HttpArg),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub method: HttpMethod,
    #[arg(short, long, default_value_t = 10)]
    pub threads: usize,
    #[arg(short, long, default_value_t = 1000)]
    pub requests: usize,
    #[arg(long, help = "Attempts to limit total requests per second")]
    pub req_rate: Option<u16>,
    #[arg(long, help = "Format: header:value header:value... (note spaces)")]
    pub headers: Option<HttpHeaders>,
}
