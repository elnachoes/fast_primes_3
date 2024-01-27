use std::{
    env, 
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
    time::Instant
};

/// check if prime function I found on wikipedia that is the best 100% accurate primeality test.
fn check_if_prime(n : u64) -> bool {
    if n == 2 || n == 3 { return true }
    if n % 2 == 0 || n % 3 == 0 { return false }
    for i in (5..).step_by(6).take_while(|i| i * i <= n) {
        if n % i == 0 || n % (i + 2) == 0 { return false }
    }
    true
}

/// this struct is used as an iterator for generating all possible numbers that could be prime.
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
        self.command_chan.send(PrimeTestThreadCommand::Shutdown).unwrap();
        if self.join_handle.is_some() {
            self.join_handle.take().unwrap().join().unwrap()
        }
    }

    fn prime_tester_thread_task(command_recv_chan : Receiver<PrimeTestThreadCommand>, result_send_chan : Sender<PrimeTestThreadResult>) {
        loop  {
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

fn n_prime(n : usize, n_threads : usize) -> u64 {
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

    threads
        .iter_mut()
        .for_each(|thread| {
            while let Ok(result) = thread.try_get_result() {
                if result.is_prime {
                    found_primes.push(result.number)
                }
            }
        });

    found_primes.sort();

    return *found_primes.iter().nth(n - 1).unwrap()
}

fn main() -> Result<(), String> {
    let args : Vec<String> = env::args().collect();
    if args.len() != 3 { return Err("expected 2 args for x prime and y worker threads".to_string()) }
    let n : usize = args[1].parse().or(Err("missing first arg : expected positive whole number to test for primality".to_string()))?;
    let n_threads : usize = args[2].parse().or(Err("missing second arg : expected positive whole number of worker threads".to_string()))?;
    println!("calculating...");
    let start_time = Instant::now();
    println!("number : {:?}, time elapsed : {:?}", n_prime(n, n_threads), start_time.elapsed());
    Ok(())
}