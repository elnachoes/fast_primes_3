use std::collections::BTreeMap;
use crate::prime::PotentialPrimesGenerator;

#[derive(Debug, Clone, Copy)]
pub enum ProcessState {
    Processing,
    Completed
}
impl ProcessState {
    fn is_processing(&self) -> bool {
        if let Self::Processing = self { true } else { false }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NumberState {
    pub number : u64,
    pub process_state : ProcessState,
    pub is_prime : bool
}

pub struct TestedNumbersState {
    tested_numbers_map : BTreeMap<u64, NumberState>,
    p_primes_gen : PotentialPrimesGenerator,
    found_primes_count : usize,
}
impl TestedNumbersState {
    pub fn new() -> Self {
        Self {
            tested_numbers_map : BTreeMap::new(),
            p_primes_gen : PotentialPrimesGenerator::new(),
            found_primes_count : 0
        }
    }

    pub fn tested_numbers_map(&self) -> &BTreeMap<u64, NumberState> {
        &self.tested_numbers_map
    }

    pub fn test_new_number(&mut self) -> u64 {
        let next_number_to_test = self.p_primes_gen.next().unwrap();
        let new_number_state = NumberState {
            number : next_number_to_test,
            process_state : ProcessState::Processing,
            is_prime : false
        };
        self.tested_numbers_map.insert(next_number_to_test, new_number_state);
        next_number_to_test
    }

    pub fn complete_tested_number(&mut self, number : u64, is_prime : bool) {
        if is_prime { self.found_primes_count += 1 }
        self.tested_numbers_map
            .get_mut(&number)
            .and_then(|number_state| {
                number_state.is_prime = is_prime;
                number_state.process_state = ProcessState::Completed;
                Some(number_state)
            });
    }

    pub fn nth_prime(&self, n : usize) -> Option<u64> {
        self.tested_numbers_map
            .iter()
            .filter(|(_number, number_state)| number_state.is_prime)
            .nth(n - 1)
            .and_then(|(number, _number_state)| Some(*number))
    }

    pub fn numbers_processing(&self) -> bool {
        self.tested_numbers_map
            .iter()
            .any(|(_number, number_state)| number_state.process_state.is_processing())
    }

    pub fn found_primes_count(&self) -> usize {
        self.found_primes_count
    }
}
