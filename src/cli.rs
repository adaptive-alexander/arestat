use clap::{Args, Parser, Subcommand};

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
    #[arg(short, long, default_value_t = 1)]
    pub threads: usize,
    #[arg(short, long, default_value_t = 50)]
    pub requests: usize,
    #[arg(long)]
    pub req_rate: Option<u16>,
    #[arg(long)]
    pub headers: Option<String>,
}
