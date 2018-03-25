extern crate ordered_float;

use std;
use viterbi;
use viterbi::*;

use self::ordered_float::NotNaN;

#[derive(Debug)]
struct DummyIt {}
impl Iterator for DummyIt {
    type Item = (DummyState, u32);
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
struct DummyState {}

impl viterbi::State for DummyState {
    type Cost = u32;
    type InputSymbol = u8;
    type ChildrenIterator = DummyIt;
    fn emission(&self, input: &[Self::InputSymbol]) -> Option<(usize, Self::Cost)> {
        Some((0, 0u32))
    }
    fn children(&self) -> DummyIt {
        return DummyIt {};
    }
}
#[test]
fn test_instantiation() {
    let inital_states = vec![(DummyState {}, 12)];
    let mut s = viterbi::Viterbi::<DummyState>::new(None, None);
    s.compute(inital_states, &vec![0, 4]).unwrap();
    println!("{:?}", s);
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum HealthObservation {
    Dizzy,
    Cold,
    Normal,
}
use self::HealthObservation::*;
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum HealthState {
    Healthy,
    Fever,
}
use self::HealthState::*;
fn transition_cost(instate: HealthState, outstate: HealthState) -> NotNaN<f32> {
    let prob: f32 = match (instate, outstate) {
        (Healthy, Healthy) => 0.7,
        (Healthy, Fever) => 0.3,
        (Fever, Fever) => 0.6,
        (Fever, Healthy) => 0.4,
    };
    let r = NotNaN::new(-prob.ln()).unwrap();
    println!("{:?} {:?} {:?} ", instate, outstate, r);
    r
}
fn emission_cost(state: HealthState, emission: HealthObservation) -> NotNaN<f32> {
    let prob: f32 = match (state, emission) {
        (Healthy, Dizzy) => 0.1,
        (Healthy, Cold) => 0.4,
        (Healthy, Normal) => 0.5,
        (Fever, Dizzy) => 0.6,
        (Fever, Cold) => 0.3,
        (Fever, Normal) => 0.1,
    };
    let r = NotNaN::new(-prob.ln()).unwrap();

    println!("{:?} {:?} {:?} ", state, emission, r);
    r
}

impl viterbi::State for HealthState {
    type Cost = NotNaN<f32>;
    type InputSymbol = HealthObservation;
    type ChildrenIterator = <Vec<(Self, Self::Cost)> as std::iter::IntoIterator>::IntoIter;
    fn emission(&self, input: &[Self::InputSymbol]) -> Option<(usize, Self::Cost)> {
        match input.first() {
            Some(emission) => Some((1, emission_cost(*self, *emission))),
            None => None,
        }
    }
    fn children(&self) -> Self::ChildrenIterator {
        let a: Vec<(HealthState, NotNaN<f32>)> = [Healthy, Fever]
            .iter()
            .map(|s| (*s, transition_cost(*self, *s)))
            .collect();
        return a.into_iter();
    }
}

#[test]
fn test_wiki_example() {
    let inital_states = vec![
        (Healthy, NotNaN::new(0.0).unwrap()),
        (Fever, NotNaN::new(0.0).unwrap()),
    ];
    let mut s = viterbi::Viterbi::<HealthState>::new(None, None);
    s.compute(inital_states, &vec![Normal, Cold, Dizzy])
        .unwrap();
    let best_path = s.best_path();

    println!("{:#?}", s);
    println!("{:#?}", best_path);
    assert!(best_path == Ok(vec![Healthy, Healthy, Fever]));
}
