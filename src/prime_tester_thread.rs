use std::{
    iter::Iterator, 
    sync::mpsc::{
        channel,
        TryRecvError,
        Sender, 
        Receiver
    }, 
    thread::{spawn, JoinHandle},
};
use std::collections::BTreeSet;
use std::time::Instant;
use indicatif::ProgressBar;
use crate::prime::{PotentialPrimesGenerator, check_if_prime};


/// this represents a command that can be sent to a prime tester thread.
enum PrimeTestThreadCommand {
    Test(u64),
    Shutdown
}

/// this represents what state the prime tester thread is in.
#[derive(Debug, Clone, Copy)]
enum PrimeTestThreadState {
    Idle,
    Testing
}
impl PrimeTestThreadState {
    pub fn is_testing(&self) -> bool {
        if let Self::Testing = self { true } else { false }
    }
}

/// this represents a result of a primality test on the prime tester thread.
#[derive(Debug, Clone, Copy)]
struct PrimeTestThreadResult {
    pub number : u64,
    pub is_prime : bool,
}

/// This is a smart thread that will test numbers for primality in a background thread.
struct PrimeTesterThread {
    command_chan : Sender<PrimeTestThreadCommand>,
    state_chan : Receiver<PrimeTestThreadState>,
    prime_result_chan : Receiver<PrimeTestThreadResult>,
    join_handle : Option<JoinHandle<()>>
}
impl PrimeTesterThread {
    /// this will start a prime tester thread. this thread will receive a number to test for primality and it will send the result to a given channel.
    /// this thread will block waiting for a number to test until a number is received or a shut down command is issued through the shuttdown method.
    pub fn new() -> Self {
        let (command_send_chan, command_recv_chan) = channel::<PrimeTestThreadCommand>();
        let (prime_result_send_chan, prime_result_recv_chan) = channel::<PrimeTestThreadResult>();
        let (state_send_chan, state_recv_chan) = channel::<PrimeTestThreadState>();
        let join_handle = spawn(move || Self::prime_tester_thread_task(command_recv_chan, prime_result_send_chan, state_send_chan));
        Self {
            state_chan : state_recv_chan,
            command_chan: command_send_chan,
            prime_result_chan : prime_result_recv_chan,
            join_handle : Some(join_handle)
        }
    }

    /// this will instruct a thread to test a number for primality. the result will be sent through the result channel.
    pub fn test_prime(&mut self, n : u64) {
        self.command_chan.send(PrimeTestThreadCommand::Test(n)).unwrap();
    }
    
    pub fn try_get_result(&mut self) -> Result<PrimeTestThreadResult, TryRecvError> {
        self.prime_result_chan.try_recv()
    }

    pub fn try_get_state(&mut self) -> Result<PrimeTestThreadState, TryRecvError> {
        self.state_chan.try_recv()
    }

    pub fn shutdown(&mut self) {
        if self.join_handle.is_some() {
            self.command_chan.send(PrimeTestThreadCommand::Shutdown).unwrap();
            self.join_handle.take().unwrap().join().unwrap()
        }
    }

    fn prime_tester_thread_task(command_recv_chan : Receiver<PrimeTestThreadCommand>, result_send_chan : Sender<PrimeTestThreadResult>, state_chan : Sender<PrimeTestThreadState>) {
        loop {
            match command_recv_chan.recv() {
                Ok(PrimeTestThreadCommand::Test(number)) => {
                    state_chan.send(PrimeTestThreadState::Testing).unwrap();
                    let result = PrimeTestThreadResult {
                        number,
                        is_prime : check_if_prime(number)
                    };

                    result_send_chan.send(result).unwrap();
                    state_chan.send(PrimeTestThreadState::Idle).unwrap();
                },
                Ok(PrimeTestThreadCommand::Shutdown) => break,
                Err(_) => {}, 
            }
        }
    }
}
impl Drop for PrimeTesterThread {
    fn drop(&mut self) {
        self.shutdown()
    }
}

pub fn n_prime_cli(n : usize, n_threads : usize) -> u64 {
    println!("calculating...");
    let progress_bar = ProgressBar::new(n as u64);
    let start_time = Instant::now();
    // setup the found primes buffer and also skip the second prime
    if n == 1 { return 2 }
    let mut p_prime_gen = PotentialPrimesGenerator::new();
    let mut found_primes : BTreeSet<u64> = BTreeSet::from([p_prime_gen.next().unwrap()]);
    progress_bar.inc(1);
    
    // spawn the threads and give them a prime to process
    let mut threads : Vec<PrimeTesterThread> = (0..n_threads)
        .into_iter()
        .map(|_| {
            let mut thread = PrimeTesterThread::new();
            thread.test_prime(p_prime_gen.next().unwrap());
            thread
        })
        .collect();

    // test new primes while the prime hasn't been found and also while there are still threads active.
    while found_primes.len() < n || threads.iter_mut().any(|t| t.try_get_state().is_ok_and(|r| r.is_testing())) {
        for thread in threads.iter_mut() {
            if let Ok(result) = thread.try_get_result() {
                if result.is_prime {
                    found_primes.insert(result.number);
                    progress_bar.inc(1);
                }
                if found_primes.len() < n {
                    thread.test_prime(p_prime_gen.next().unwrap())
                }
            }
        }
    }
    progress_bar.finish();
    let result = *found_primes.iter().nth(n - 1).unwrap();

    println!("number : {:?}, time elapsed : {:?}", result, start_time.elapsed());

    result
}