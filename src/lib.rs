#![feature(retain_hash_collection)]
#![feature(conservative_impl_trait)]

extern crate permutation;
extern crate symbol_map;

mod viterbi;
mod windowiter;
#[cfg(test)]
mod test;

pub use viterbi::*;
