#![feature(conservative_impl_trait)]

extern crate permutation;
extern crate symbol_map;

#[cfg(test)]
mod test;
mod viterbi;
mod windowiter;

pub use viterbi::*;
