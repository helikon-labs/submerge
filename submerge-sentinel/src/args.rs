use clap::Parser;

#[derive(Parser, Clone, Debug)]
pub struct Args {
    #[arg(env, long)]
    /// Merkle Science API key
    pub merkle_science_api_key: String,

    #[arg(long, default_value = "20")]
    /// API request timeout in seconds
    pub request_timeout_secs: u64,
}
