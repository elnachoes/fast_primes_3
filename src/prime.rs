use std::iter::Iterator;

#[derive(Clone, Copy, Debug)]
pub enum CheckIfPrimeState {
    InitialCheck,
    DivisibilityChecks,
    Completed(bool)
}
impl CheckIfPrimeState {
    pub fn is_completed(&self) -> bool {
        if let Self::Completed(_) = self { true } else { false }
    }
}

/// check if prime function I found on wikipedia that is the best 100% accurate primeality test.
/// TODO lets turn this into a iterator that can be canceled :D
pub fn check_if_prime(n : u64) -> bool {
    if n == 2 || n == 3 { return true }
    if n % 2 == 0 || n % 3 == 0 { return false }
    for i in (5..).step_by(6).take_while(|i| i * i <= n) {
        if n % i == 0 || n % (i + 2) == 0 { return false }
    }
    true
}

/// this struct is used as an iterator for generating all possible numbers that could be prime.
pub struct PotentialPrimesGenerator {
    current : u64,
}
impl PotentialPrimesGenerator {
    pub fn new() -> Self {
        Self { current : 2 }
    }
}
impl Iterator for PotentialPrimesGenerator {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let divisibility_filters : &'static[u64] = &[3,5,9,7,11];
        if self.current == 2 {
            let val = self.current; 
            self.current += 1;
            Some(val)
        }
        else if divisibility_filters.contains(&self.current) {
            let val = self.current; 
            self.current += 2;
            Some(val)
        }
        else {
            if divisibility_filters.iter().any(|filter| self.current % filter == 0) {
                while divisibility_filters.iter().any(|filter| self.current % filter == 0) {
                    self.current += 2
                }
                let val = self.current; 
                self.current += 2;
                Some(val)
            } else {
                let val = self.current; 
                self.current += 2;
                Some(val)
            }
        }
    }
}