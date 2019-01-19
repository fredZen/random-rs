use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::iter::IntoIterator;
use std::iter::Iterator;
use std::ops::Index;

#[derive(Default)]
pub struct Primes {
    primes: Vec<u32>,
    i: usize,
    composite_to_stride: HashMap<u32, u32>,
    sieving_prime: u32,
    sieving_square: u32,
    candidate: u32,
}

impl Primes {
    pub fn new() -> Primes {
        let mut res = Primes {
            primes: vec![2, 3],
            i: 1,
            ..Default::default()
        };
        res.next_sieving_prime();
        res.candidate = res.sieving_prime;
        res
    }

    fn remember_stride(&mut self, stride: u32) {
        let mut it = ((self.candidate + stride)..).step_by(stride as usize);
        loop {
            if let Entry::Vacant(v) = self.composite_to_stride.entry(it.next().unwrap()) {
                v.insert(stride);
                return;
            }
        }
    }

    fn next_sieving_prime(&mut self) {
        self.sieving_prime = self.primes[self.i];
        self.sieving_square = self.sieving_prime * self.sieving_prime;
        self.i += 1;
    }

    fn ensure(&mut self, size: u32) {
        while self.primes.len() < size as usize {
            self.candidate += 2;

            if let Some(stride) = self.composite_to_stride.remove(&self.candidate) {
                self.remember_stride(stride);
            } else if self.candidate != self.sieving_square {
                self.primes.push(self.candidate);
            } else {
                let stride = self.sieving_prime * 2;
                self.remember_stride(stride);
                self.next_sieving_prime();
            }
        }
    }

    pub fn get(&mut self, i: u32) -> u32 {
        self.ensure(i + 1);
        self[i]
    }
}

impl Index<u32> for Primes {
    type Output = u32;

    fn index(&self, i: u32) -> &Self::Output {
        &self.primes[i as usize]
    }
}

impl<'a> IntoIterator for &'a mut Primes {
    type Item = u32;
    type IntoIter = PrimesIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PrimesIterator { primes: self, i: 0 }
    }
}

pub struct PrimesIterator<'a> {
    primes: &'a mut Primes,
    i: u32,
}

impl Iterator for PrimesIterator<'_> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.i += 1;
        Some(self.primes.get(self.i - 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primes() {
        let mut primes = Primes::new();
        let prime_vec: Vec<_> = (primes).into_iter().take(15).collect();

        assert_eq!(
            prime_vec,
            vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47]
        );

        assert_eq!(47, primes[14]);
    }
}
