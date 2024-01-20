use core::num;
use std::{
    thread::{spawn, Thread, ThreadId, JoinHandle, self},
    sync::mpsc::{self, Sender, Receiver},
    iter::Iterator
};

fn check_if_prime(n : u64) -> bool {
    if n == 2 || n == 3 { return true }
    if n % 2 == 0 || n % 3 == 0 { return false }
    for i in (5..).step_by(6).take_while(|i| i * i <= n) {
        if n % i == 0 || n % (i + 2) == 0 { return false }
    }
    true
}

struct PotentialPrimesGenerator {
    current : u64,
}
impl PotentialPrimesGenerator {
    fn new() -> Self {
        Self { current : 2 }
    }
}
impl Iterator for PotentialPrimesGenerator {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.current;
        if val == 2 {
            self.current += 1
        } else {
            self.current += 2
        }
        Some(val)
    }
}

enum PrimeTestThreadCommand {
    //TODO hey you know what we might consider trying testing a whole list
    Test(u64),
    Shutdown
}

struct PrimeTestThreadResult {
    pub thread_id : ThreadId,
    pub number : u64,
    pub is_prime : bool,
}

/// main thread -> PrimeTesterThreadCommand : prime tester thread
/// 
/// main thread <- (number, is_prime, thread_id)
/// 
/// TODO we should probably implement drop on this to cleanup
struct PrimeTesterThread {
    test_prime_send_chan : Sender<PrimeTestThreadCommand>,
    join_handle : Option<JoinHandle<()>>
}
impl PrimeTesterThread {
    /// this will start a prime tester thread. this thread will receive a number to test for primality and it will send the result to a given channel.
    /// this thread will block waiting for a number to test until a number is received or a shut down command is issued through the shuttdown method.
    pub fn start(result_send_chan : Sender<PrimeTestThreadResult>) -> Self {
        let (test_prime_send_chan, test_prime_recv_chan) = mpsc::channel::<PrimeTestThreadCommand>();
        let join_handle = spawn(move || Self::prime_tester_thread_task(test_prime_recv_chan, result_send_chan));
        Self {
            test_prime_send_chan : test_prime_send_chan,
            join_handle : Some(join_handle)
        }
    }

    pub fn test_potential_prime(&mut self, n : u64) {
        self.test_prime_send_chan.send(PrimeTestThreadCommand::Test(n)).unwrap()
    }

    pub fn shutdown(&mut self) {
        if self.join_handle.is_some() {
            self.join_handle.take().unwrap().join().unwrap()
        }
    }

    fn prime_tester_thread_task(test_prime_recv_chan : Receiver<PrimeTestThreadCommand>, result_send_chan : Sender<PrimeTestThreadResult>) {
        'threadloop: loop  {
            match test_prime_recv_chan.try_recv() {
                Ok(PrimeTestThreadCommand::Test(number)) => {
                    let result = PrimeTestThreadResult {
                        thread_id : thread::current().id(),
                        number : number,
                        is_prime : check_if_prime(number)
                    };

                    // NOTE might be some errors happnin here unsure just yet probably not
                    result_send_chan.send(result).unwrap()
                },
                Ok(PrimeTestThreadCommand::Shutdown) => break 'threadloop,
                Err(_) => {
                    // NOTE may be worth inspecting here if shit is going sideways
                }, 
            }
        }
    }
}
impl Drop for PrimeTesterThread {
    fn drop(&mut self) {
        self.test_prime_send_chan.send(PrimeTestThreadCommand::Shutdown).unwrap();
        self.shutdown()
    }
}

//TODO :
struct PrimeTesterThreadPool {
    threads : Vec<PrimeTesterThread>
}

fn main() {
    let mut p_prime_gen = PotentialPrimesGenerator::new();

    let (sender, receiver) = mpsc::channel::<(f64, bool)>();
}