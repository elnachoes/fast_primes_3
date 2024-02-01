use core::num;
use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display, io::Lines, net::Shutdown, sync::{
        mpsc::{
            self, Receiver, Sender
        }, 
        Mutex,
        Arc
    }, 
    thread::{JoinHandle, spawn},
};
use crate::prime::{self, PotentialPrimesGenerator};
use itertools::*; 
use crate::models::*;

struct AtomicTestedNumbersState {
    tested_numbers_state : Arc<Mutex<TestedNumbersState>>
}
impl AtomicTestedNumbersState {
    pub fn new() -> AtomicTestedNumbersState {
        Self {
            tested_numbers_state : Arc::new(Mutex::new(TestedNumbersState::new()))
        }
    }

    pub fn test_new_number(&mut self) -> u64 {
        self.tested_numbers_state.lock().unwrap().test_new_number()
    }

    pub fn complete_tested_number(&mut self, number : u64, is_prime : bool) {
        self.tested_numbers_state.lock().unwrap().complete_tested_number(number, is_prime)
    }

    pub fn nth_prime(&self, n : usize) -> Option<u64> {
        // println!("{}", self.tested_numbers_state.lock().unwrap());
        self.tested_numbers_state.lock().unwrap().nth_prime(n)
    }

    pub fn check_numbers_processing(&self) -> bool {
        self.tested_numbers_state.lock().unwrap().numbers_processing()
    }

    pub fn found_primes_count(&self) -> usize {
        self.tested_numbers_state.lock().unwrap().found_primes_count()
    }
}
impl Clone for AtomicTestedNumbersState {
    fn clone(&self) -> Self {
        Self {
            tested_numbers_state : Arc::clone(&self.tested_numbers_state)
        }
    }
}

impl Display for TestedNumbersState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("{{\n")?;
        f.write_fmt(format_args!("\tfound primes count : {}\n", self.found_primes_count()))?;
        f.write_fmt(format_args!("\tfound primes : [:\n"))?;
        for (_key, value) in self.tested_numbers_map().iter() {
            f.write_fmt(format_args!("\t\t{:?}\n", value))?;
        }
        f.write_fmt(format_args!("\t]:\n"))?;
        f.write_fmt(format_args!("}}:\n"))?;
        Ok(())
    }
}

/// this represents a command that can be sent to a prime tester thread.
enum PrimeTestThreadCommand {
    Shutdown
    // TODO : pause maybe?
}

struct PrimeTesterThread {
    command_chan : Sender<PrimeTestThreadCommand>,
    join_handle : Option<JoinHandle<()>>,
}
impl PrimeTesterThread {
    pub fn new(tested_numbers : AtomicTestedNumbersState, target_prime : usize) -> Self {
        let (command_send_chan, command_recv_chan) : (Sender<PrimeTestThreadCommand>, Receiver<PrimeTestThreadCommand>) = mpsc::channel();
        let join_handle = spawn(move || Self::prime_tester_thread_task(tested_numbers.clone(), command_recv_chan, target_prime));
        Self {
            command_chan : command_send_chan,
            join_handle : Some(join_handle)
        }
    }

    pub fn shutdown(&mut self) {
        if self.join_handle.is_some() {
            let _ = self.command_chan.send(PrimeTestThreadCommand::Shutdown);
            self.join_handle.take().unwrap().join().unwrap()
        }
    }

    fn prime_tester_thread_task(mut tested_numbers : AtomicTestedNumbersState, command_chan : Receiver<PrimeTestThreadCommand>, target_prime : usize) {
        loop {
            // probably have some more on here for shutting down the thread
            if let Ok(PrimeTestThreadCommand::Shutdown) = command_chan.try_recv() { break }
            if tested_numbers.found_primes_count() as usize >= target_prime { break }

            let new_test_number = tested_numbers.test_new_number();
            let is_prime = prime::check_if_prime(new_test_number);
            tested_numbers.complete_tested_number(new_test_number, is_prime)
        }
    }
}
impl Drop for PrimeTesterThread {
    fn drop(&mut self) {
        self.shutdown()
    }
}

pub fn n_prime(n : usize, n_threads : usize) -> u64 {
    let tested_numbers = AtomicTestedNumbersState::new();

    let _threads : Vec<PrimeTesterThread> = (0..n_threads)
        .into_iter()
        .map(|_| PrimeTesterThread::new(tested_numbers.clone(), n))
        .collect();

    while tested_numbers.found_primes_count() < n {}

    tested_numbers.nth_prime(n).unwrap()
}