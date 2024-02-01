use std::{
    iter::Iterator, 
    sync::mpsc::{
        self, 
        Sender, 
        Receiver
    }, 
    thread::{
        spawn, 
        JoinHandle
    },
};

use crate::{models::TestedNumbersState, prime::{PotentialPrimesGenerator, check_if_prime}};


/// this represents a command that can be sent to a prime tester thread.
enum PrimeTestThreadCommand {
    Test(u64),
    Shutdown
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
    prime_result_chan : Receiver<PrimeTestThreadResult>, 
    join_handle : Option<JoinHandle<()>>
}
impl PrimeTesterThread {
    /// this will start a prime tester thread. this thread will receive a number to test for primality and it will send the result to a given channel.
    /// this thread will block waiting for a number to test until a number is received or a shut down command is issued through the shuttdown method.
    pub fn new() -> Self {
        let (command_send_chan, command_recv_chan) = mpsc::channel::<PrimeTestThreadCommand>();
        let (prime_result_send_chan, prime_result_recv_chan) = mpsc::channel::<PrimeTestThreadResult>();
        let join_handle = spawn(move || Self::prime_tester_thread_task(command_recv_chan, prime_result_send_chan));
        Self {
            command_chan: command_send_chan,
            prime_result_chan : prime_result_recv_chan,
            join_handle : Some(join_handle)
        }
    }

    /// this will instruct a thread to test a number for primality. the result will be sent through the result channel.
    pub fn test_prime(&mut self, n : u64) {
        self.command_chan.send(PrimeTestThreadCommand::Test(n)).unwrap();
    }
    
    pub fn try_get_result(&mut self) -> Result<PrimeTestThreadResult, mpsc::TryRecvError> {
        self.prime_result_chan.try_recv()
    }

    pub fn shutdown(&mut self) {
        if self.join_handle.is_some() {
            self.command_chan.send(PrimeTestThreadCommand::Shutdown).unwrap();
            self.join_handle.take().unwrap().join().unwrap()
        }
    }

    fn prime_tester_thread_task(command_recv_chan : Receiver<PrimeTestThreadCommand>, result_send_chan : Sender<PrimeTestThreadResult>) {
        loop {
            match command_recv_chan.recv() {
                Ok(PrimeTestThreadCommand::Test(number)) => {
                    let result = PrimeTestThreadResult {
                        number : number,
                        is_prime : check_if_prime(number)
                    };

                    result_send_chan.send(result).unwrap()
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

pub fn n_prime(n : usize, n_threads : usize) -> u64 {
    // setup the found primes buffer and also skip the second prime
    if n == 1 { return 2 }
    let mut p_prime_gen = PotentialPrimesGenerator::new();
    let mut found_primes : Vec<u64> = vec![p_prime_gen.next().unwrap()];

    // spawn the threads and give them a prime to process
    let mut threads : Vec<PrimeTesterThread> = (0..n_threads)
        .into_iter()
        .map(|_| {
            let mut thread = PrimeTesterThread::new();
            thread.test_prime(p_prime_gen.next().unwrap());
            thread
        })
        .collect();

    while found_primes.len() < n {
        for thread in threads.iter_mut() {
            if let Ok(result) = thread.try_get_result() {
                if result.is_prime {
                    found_primes.push(result.number);
                }
                thread.test_prime(p_prime_gen.next().unwrap())
            }
        }
    }

    found_primes.sort();

    return *found_primes.iter().nth(n - 1).unwrap()
}

// pub fn n_prime(n : usize, n_threads : usize) -> u64 {
//     // setup the found primes buffer and also skip the second prime
//     if n == 1 { return 2 }
//     let tested_numbers = TestedNumbersState::new();

//     // spawn the threads and give them a prime to process
//     let mut threads : Vec<PrimeTesterThread> = (0..n_threads)
//         .into_iter()
//         .map(|_| {
//             let mut thread = PrimeTesterThread::new();
//             thread.test_prime(tested_numbers.test_new_number());
//             thread
//         })
//         .collect();

//     while tested_numbers.found_primes_count() < n {
//         for thread in threads.iter_mut() {
//             if let Ok(result) = thread.try_get_result() {
//                 if result.is_prime {
//                     tested_numbers.complete_tested_number(number, is_prime);
//                 }
//                 thread.test_prime(p_prime_gen.next().unwrap())
//             }
//         }
//     }

//     // found_primes.sort();

//     // return *found_primes.iter().nth(n - 1).unwrap()
//     0
// }
