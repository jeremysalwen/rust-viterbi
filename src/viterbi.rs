use std;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::iter::IntoIterator;

use std::ops::Deref;

use symbol_map;
use symbol_map::SymbolId;
use symbol_map::indexing::Indexing;
use symbol_map::indexing::Insertion;

#[derive(Eq, Ord, PartialEq, PartialOrd, Default, Hash, Copy, Clone, Debug)]
struct StateId(usize);

type InputIter = usize;

impl SymbolId for StateId {
    fn next(&self) -> Self {
        return StateId(self.0.next());
    }
    fn as_usize(&self) -> usize {
        return self.0.as_usize();
    }
}

pub trait State:
    std::marker::Sized + Clone + std::cmp::Eq + std::hash::Hash + std::fmt::Debug
{
    type Cost: std::cmp::Ord
        + std::ops::Add<Output = Self::Cost>
        + std::marker::Copy
        + std::fmt::Debug;
    type InputSymbol;
    type ChildrenIterator: Iterator<Item = (Self, Self::Cost)>;
    fn emission(&self, &[Self::InputSymbol]) -> Option<(usize, Self::Cost)>;
    fn children(&self) -> Self::ChildrenIterator;
}

fn children_with_emission<'a, 'b, 'c, S: State>(
    s: &'a S,
    input: &'b [S::InputSymbol],
) -> impl Iterator<Item = (S, S::Cost, usize)> + 'c
where
    'a: 'c,
    'b: 'c,
{
    s.children().filter_map(move |(state, transition_cost)| {
        state
            .emission(input)
            .map(|(offset, emission_cost)| (state, transition_cost + emission_cost, offset))
    })
}

#[derive(Clone, Copy, Debug)]
struct StateInfo<C> {
    input_idx: InputIter,
    cost: C,
    //Note that this references the state from the previous layer.
    parent_idx: Option<StateId>,
}

impl<C> StateInfo<C>
where
    C: std::cmp::Ord + std::ops::Add + std::marker::Copy,
{
    fn update(&mut self, other: StateInfo<C>) {
        if other.cost < self.cost {
            *self = other;
        }
    }
}

#[derive(Debug)]
struct ViterbiStepIter<'a, S: State + 'a> {
    step: &'a ViterbiStep<S>,
    idx_iterator: std::collections::hash_map::Iter<'a, StateId, StateInfo<S::Cost>>,
}

impl<'a, S> Iterator for ViterbiStepIter<'a, S>
where
    S: State,
{
    type Item = (&'a StateId, &'a S, &'a StateInfo<S::Cost>);
    fn next(&mut self) -> Option<(&'a StateId, &'a S, &'a StateInfo<S::Cost>)> {
        match self.idx_iterator.next() {
            Some((idx, stateinfo)) => {
                let ref state = &self.step.state_table.get_symbol(idx).unwrap().data();
                Some((idx, state, stateinfo))
            }
            None => return None,
        }
    }
}

#[derive(Debug)]
struct ViterbiStep<S: State> {
    state_table: symbol_map::indexing::HashIndexing<S, StateId>,
    state_info: HashMap<StateId, StateInfo<S::Cost>>,
}

fn emit<S, Input>(
    state: S,
    stateinfo: StateInfo<S::Cost>,
    input: &Input,
) -> Option<(S, StateInfo<S::Cost>)>
where
    S: State,
    Input: Deref<Target = [S::InputSymbol]>,
{
    match state.emission(input) {
        Some((emission_offset, emission_cost)) => Some((
            state,
            StateInfo::<S::Cost> {
                input_idx: stateinfo.input_idx + emission_offset,
                cost: stateinfo.cost + emission_cost,
                parent_idx: stateinfo.parent_idx,
            },
        )),
        None => None,
    }
}

impl<S> ViterbiStep<S>
where
    S: State,
{
    fn from_iter<T>(initial: T) -> Result<ViterbiStep<S>, String>
    where
        T: IntoIterator<Item = (S, StateInfo<S::Cost>)>,
    {
        let mut result = Self::new();
        for (state, stateinfo) in initial {
            let state_id = match result.state_table.get_or_insert(state) {
                Insertion::New(entry) => entry.id(),
                Insertion::Present(entry) => return Err(String::from("Duplicate start state. ")),
            };

            result.state_info.insert(*state_id, stateinfo);
        }
        return Ok(result);
    }
    fn new() -> ViterbiStep<S> {
        ViterbiStep::<S> {
            state_table: symbol_map::indexing::HashIndexing::<S, StateId>::default(),
            state_info: HashMap::new(),
        }
    }
    fn iter(&self) -> ViterbiStepIter<S> {
        ViterbiStepIter {
            step: self,
            idx_iterator: self.state_info.iter(),
        }
    }
    fn update(&mut self, state: S, stateinfo: StateInfo<S::Cost>) {
        let id = self.state_table.get_or_insert(state).unwrap().id();
        match self.state_info.entry(*id) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().update(stateinfo);
            }
            Entry::Vacant(entry) => {
                entry.insert(stateinfo);
            }
        };
    }
    //TODO
    fn sort(&mut self) {}
    //TODO
    fn cull(&mut self, max: usize) {}
}

#[derive(Debug)]
pub struct Viterbi<S: State> {
    steps: Vec<ViterbiStep<S>>,
    max_states: Option<usize>,
    max_cost: Option<S::Cost>,
}

impl<S> Viterbi<S>
where
    S: State,
{
    pub fn new(max_states: Option<usize>, max_cost: Option<S::Cost>) -> Viterbi<S> {
        Viterbi {
            steps: vec![],
            max_states: max_states,
            max_cost: max_cost,
        }
    }

    fn step<T>(&mut self, input: &T) -> Result<usize, String>
    where
        T: Deref<Target = [S::InputSymbol]>,
    {
        println!("Step! {:?}", self.steps.len());
        {
            self.steps.push(ViterbiStep::new());
            let len = self.steps.len();
            let (prev_steps, new_steps) = self.steps.as_mut_slice().split_at_mut(len - 1);
            let prev_step = try!(prev_steps.last().ok_or("Error, no starting state."));
            let ref mut new_step = new_steps[0];
            for (idx, state, stateinfo) in prev_step.iter() {
                let remaining_input = &input.split_at(stateinfo.input_idx).1;
                println!("Input offset {:?}", remaining_input.len());
                for (next_state, child_cost, input_offset) in
                    children_with_emission(state, remaining_input)
                {
                    println!("Child! {:?}", input_offset);
                    new_step.update(
                        next_state,
                        StateInfo {
                            input_idx: stateinfo.input_idx + input_offset,
                            cost: stateinfo.cost + child_cost,
                            parent_idx: Some(*idx),
                        },
                    );
                }
            }
            match self.max_states {
                Some(max) => new_step.cull(max),
                None => (),
            }
        }
        let result = Ok(self.steps.last().unwrap().state_info.len());
        if result == Ok(0) {
            self.steps.pop();
        }
        println!("Result {:?}", result);
        return result;
    }

    pub fn compute<Initial, Input>(
        &mut self,
        initial_states: Initial,
        input: &Input,
    ) -> Result<(), String>
    where
        Initial: IntoIterator<Item = (S, S::Cost)>,
        Input: Deref<Target = [S::InputSymbol]>,
    {
        let emitted_states = initial_states.into_iter().filter_map(|(state, cost)| {
            emit(
                state,
                StateInfo {
                    cost: cost,
                    input_idx: 0,
                    parent_idx: None,
                },
                input,
            )
        });
        self.steps = vec![try!(ViterbiStep::<S>::from_iter(emitted_states))];
        while try!(self.step(input)) != 0 {}
        Ok(())
    }
    fn last_step(&self) -> Result<&ViterbiStep<S>, String> {
        self.steps.last().ok_or(String::from("No viterbi steps."))
    }
    pub fn best_path(&self) -> Result<Vec<S>, String> {
        let last = try!(self.last_step());
        let start_state = try!(
            last.state_info
                .iter()
                .min_by_key(|&(K, V)| V.cost)
                .map(|(K, V)| K)
                .ok_or(String::from("No states in last step"))
        );
        let mut state: Option<StateId> = Some(*start_state);

        let mut result = Vec::<S>::new();
        for step in self.steps.iter().rev() {
            result.push(
                step.state_table
                    .get_symbol(&state.unwrap())
                    .unwrap()
                    .data()
                    .clone(),
            );
            let stateinfo = step.state_info.get(&state.unwrap()).unwrap();
            state = stateinfo.parent_idx;
        }
        result.reverse();
        return Ok(result);
    }
}
