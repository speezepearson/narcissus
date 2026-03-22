use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

// ── Direction ──
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Left,
    Right,
}

pub trait TuringMachineSpec {
    type State: Copy + PartialEq + Eq + Hash;
    type Symbol: Copy + PartialEq + Eq + Hash;
    fn initial(&self) -> Self::State;
    fn blank(&self) -> Self::Symbol;
    fn is_accepting(&self, state: Self::State) -> bool;
    fn get_transition(
        &self,
        state: Self::State,
        symbol: Self::Symbol,
    ) -> Option<(Self::State, Self::Symbol, Dir)>;

    fn iter_states(&self) -> impl Iterator<Item = Self::State>;
    fn iter_symbols(&self) -> impl Iterator<Item = Self::Symbol>;
    fn iter_rules(
        &self,
    ) -> impl Iterator<Item = (Self::State, Self::Symbol, Self::State, Self::Symbol, Dir)>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleTuringMachineSpec<
    State: Copy + PartialEq + Eq + Hash,
    Symbol: Copy + PartialEq + Eq + Hash,
> {
    pub initial: State,
    pub accepting: HashSet<State>,
    pub blank: Symbol,
    pub transitions: HashMap<(State, Symbol), (State, Symbol, Dir)>,
    pub all_states: Vec<State>,
    pub all_symbols: Vec<Symbol>,
}

impl<State: Copy + PartialEq + Eq + Hash, Symbol: Copy + PartialEq + Eq + Hash> TuringMachineSpec
    for SimpleTuringMachineSpec<State, Symbol>
{
    type State = State;
    type Symbol = Symbol;
    fn initial(&self) -> Self::State {
        self.initial
    }
    fn blank(&self) -> Self::Symbol {
        self.blank
    }
    fn is_accepting(&self, state: Self::State) -> bool {
        self.accepting.contains(&state)
    }
    fn get_transition(
        &self,
        state: Self::State,
        symbol: Self::Symbol,
    ) -> Option<(Self::State, Self::Symbol, Dir)> {
        self.transitions.get(&(state, symbol)).cloned()
    }
    fn iter_states(&self) -> impl Iterator<Item = Self::State> {
        self.all_states.iter().copied()
    }
    fn iter_symbols(&self) -> impl Iterator<Item = Self::Symbol> {
        self.all_symbols.iter().copied()
    }
    fn iter_rules(
        &self,
    ) -> impl Iterator<Item = (Self::State, Self::Symbol, Self::State, Self::Symbol, Dir)> {
        self.transitions
            .iter()
            .map(|((state, symbol), (next_state, next_symbol, dir))| {
                (*state, *symbol, *next_state, *next_symbol, *dir)
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunningTuringMachine<'a, Spec: TuringMachineSpec> {
    pub spec: &'a Spec,
    pub state: Spec::State,
    pub pos: usize,
    pub tape: Vec<Spec::Symbol>,
}

impl<'a, Spec: TuringMachineSpec> RunningTuringMachine<'a, Spec> {
    pub fn new(spec: &'a Spec) -> Self {
        Self {
            spec,
            state: spec.initial(),
            pos: 0,
            tape: Vec::new(),
        }
    }
}

// ════════════════════════════════════════════════════════════════════
// Run a TM
// ════════════════════════════════════════════════════════════════════

#[allow(dead_code)]
pub enum RunningTMStatus {
    Accepted,
    Rejected,
    Running,
}

#[allow(dead_code)]
pub fn step<Spec: TuringMachineSpec>(machine: &mut RunningTuringMachine<Spec>) -> RunningTMStatus {
    let sym = machine.tape[machine.pos];
    if let Some((ns, nsym, dir)) = machine.spec.get_transition(machine.state, sym) {
        machine.state = ns;
        machine.tape[machine.pos] = nsym;
        machine.pos = match dir {
            Dir::Left => machine.pos.saturating_sub(1),
            Dir::Right => machine.pos + 1,
        };
        RunningTMStatus::Running
    } else {
        if machine.spec.is_accepting(machine.state) {
            RunningTMStatus::Accepted
        } else {
            RunningTMStatus::Rejected
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HaltReason {
    Accepted { num_steps: usize },
    Rejected { num_steps: usize },
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartialRunReason {
    StepLimit,
}

#[allow(dead_code)]
pub fn run_tm<Spec: TuringMachineSpec>(
    tm: &mut RunningTuringMachine<Spec>,
    max_steps: usize,
    mut extender: Option<&mut dyn TapeExtender<Spec::Symbol>>,
) -> Result<HaltReason, PartialRunReason> {
    for step_count in 0..max_steps {
        if tm.pos >= tm.tape.len() {
            match extender.as_deref_mut() {
                None => tm.tape.resize(tm.pos + 1, tm.spec.blank()),
                Some(extender) => extender.extend(&mut tm.tape, tm.pos),
            }
        }
        match step(tm) {
            RunningTMStatus::Running => continue,
            RunningTMStatus::Accepted => {
                return Ok(HaltReason::Accepted {
                    num_steps: step_count,
                })
            }
            RunningTMStatus::Rejected => {
                return Ok(HaltReason::Rejected {
                    num_steps: step_count,
                })
            }
        }
    }
    Err(PartialRunReason::StepLimit)
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunUntilResult {
    Accepted { num_steps: usize },
    Rejected { num_steps: usize },
    StepLimit,
}

/// Run the TM until it enters `target_state`, taking at least one step.
/// Returns Ok(num_steps) if the target state is reached, or Err with the reason
/// if the machine halts or hits the step limit first.
#[allow(dead_code)]
pub fn run_until_enters_state<Spec: TuringMachineSpec>(
    tm: &mut RunningTuringMachine<Spec>,
    target_state: Spec::State,
    max_steps: usize,
    mut extender: Option<&mut dyn TapeExtender<Spec::Symbol>>,
) -> Result<usize, RunUntilResult> {
    for step_count in 1..=max_steps {
        if tm.pos >= tm.tape.len() {
            match extender.as_deref_mut() {
                None => tm.tape.resize(tm.pos + 1, tm.spec.blank()),
                Some(extender) => extender.extend(&mut tm.tape, tm.pos + 1),
            }
        }
        match step(tm) {
            RunningTMStatus::Running => {
                if tm.state == target_state {
                    return Ok(step_count);
                }
            }
            RunningTMStatus::Accepted => {
                if tm.state == target_state {
                    return Ok(step_count);
                }
                return Err(RunUntilResult::Accepted {
                    num_steps: step_count,
                });
            }
            RunningTMStatus::Rejected => {
                if tm.state == target_state {
                    return Ok(step_count);
                }
                return Err(RunUntilResult::Rejected {
                    num_steps: step_count,
                });
            }
        }
    }
    Err(RunUntilResult::StepLimit)
}

pub trait TapeExtender<Symbol> {
    fn extend(&mut self, tape: &mut Vec<Symbol>, min_size: usize);
}
