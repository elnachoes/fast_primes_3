use fast_primes_3::{
    args::Args, 
    n_prime::n_prime
};

fn main() -> Result<(), String> {
    let args = Args::parse();
    let result = n_prime(&args);
    if args.quiet { println!("{}", result) }
    Ok(())
}