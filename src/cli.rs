use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(short = "t", default_value = "100000")]
    pub total_requests: usize,
    #[structopt(short = "w", default_value = "256")]
    pub concurrent_workers: usize,
    #[structopt(default_value = "http://127.0.0.1:15000/")]
    pub url: String,
    #[structopt(short = "r")]
    pub file_root: String,
}
