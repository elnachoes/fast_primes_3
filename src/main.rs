use std::env;
use fast_primes_3::{self, prime_tester_thread, };

fn main() -> Result<(), String> {
    let args : Vec<String> = env::args().collect();
    if args.len() != 3 { return Err("expected 2 args for x prime and y worker threads".to_string()) }
    let n : usize = args[1].parse().or(Err("missing first arg : expected positive whole number to test for primality".to_string()))?;
    let n_threads : usize = args[2].parse().or(Err("missing second arg : expected positive whole number of worker threads".to_string()))?;
    prime_tester_thread::n_prime_cli(n, n_threads);
    Ok(())
}