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
        let val = self.current;
        if val == 2 {
            self.current += 1
        } else {
            self.current += 2
        }
        Some(val)
    }
}