use clap::Parser;

/// This is a program that locates n prime with n worker threads
/// Example use : fast_primes_3 -n 50000 -t 5
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct RawArgs {
    /// Nth prime to find
    #[arg(short, long)]
    n: u64,

    /// Number of worker threads to use
    #[arg(short, long, default_value_t = 1)]
    threads: u64,
    
    /// This runs the app quietly and doesn't print anything but the result to stdout.
    #[arg(short, long, default_value_t = false)]
    quiet: bool,
}

pub struct Args {
    pub n: u64,
    pub threads: u64,
    pub quiet: bool,
}
impl Args {
    pub fn parse() -> Self {
        let raw = RawArgs::parse();
        Self {
            threads: raw.threads,
            n: raw.n,
            quiet: raw.quiet,
        }
    }
}
