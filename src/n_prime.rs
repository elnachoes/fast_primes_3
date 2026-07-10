use std::collections::BTreeSet;
use std::time::Instant;
use indicatif::ProgressBar;
use crate::args::Args;
use crate::prime::PotentialPrimesGenerator;
use crate::prime_tester_thread::PrimeTesterThread;

pub fn n_prime(args : &Args) -> u64 {
    let complete_result = |result : u64, start_time : Option<Instant>| {
        if !args.quiet {
            println!("number : {:?}, time elapsed : {:?}", result, start_time.unwrap().elapsed());
        }
        result
    };

    if !args.quiet {
        println!("calculating the {}th prime with {} worker threads...", args.n, args.threads);
    }
    let progress_bar =  if args.quiet { None } else {
        let bar = ProgressBar::new(args.n);
        bar.set_style(indicatif::ProgressStyle::with_template("[{elapsed_precise}][primes per second : {per_sec}] {bar:100.cyan/green}")
            .unwrap()
            .progress_chars("++-"));
        Some(bar)
    };

    let start_time = if args.quiet { None } else { Some(Instant::now()) };
    // setup the found primes buffer and also skip the second prime
    if args.n == 1 { return complete_result(2, start_time) }

    // setup the generator for making possible primes
    let mut p_prime_gen = PotentialPrimesGenerator::new();

    // set the found primes list to a bst set of just [2] and increment the progress bar once if quiet mode is disabled
    let mut found_primes : BTreeSet<u64> = BTreeSet::from([p_prime_gen.next().unwrap()]);
    progress_bar.as_ref().and_then(|bar| Some(bar.inc(1)));

    // spawn the threads and give them a prime to process
    let mut threads : Vec<PrimeTesterThread> = (0..args.threads)
        .into_iter()
        .map(|_| {
            let mut thread = PrimeTesterThread::new();
            thread.test_prime(p_prime_gen.next().unwrap());
            thread
        })
        .collect();

    // test new primes while the prime hasn't been found and also while there are still threads active.
    while found_primes.len() < args.n as usize || threads.iter_mut().any(|t| t.try_get_state().is_ok_and(|r| r.is_testing())) {
        for thread in threads.iter_mut() {
            if let Ok(result) = thread.try_get_result() {
                if result.is_prime {
                    found_primes.insert(result.number);
                    progress_bar.as_ref().and_then(|bar| Some(bar.inc(1)));
                }
                if found_primes.len() < args.n as usize {
                    thread.test_prime(p_prime_gen.next().unwrap())
                }
            }
        }
    }
    // complete progress if not quiet and return the result
    progress_bar.and_then(|bar| Some(bar.finish()));
    complete_result(*found_primes.iter().nth((args.n - 1) as usize).unwrap(), start_time)
}