use crate::tm::{RunningTuringMachine, TuringMachineSpec};

pub trait Encoder<'a, UtmSymbol, Guest: 'a + TuringMachineSpec> {
    fn encode(&self, tm: &RunningTuringMachine<Guest>) -> Vec<UtmSymbol>;
    fn decode(&self, tape: &[UtmSymbol]) -> Result<RunningTuringMachine<'a, Guest>, String>;
}

pub trait UtmSpec: TuringMachineSpec {
    type Encoder<'a, Guest: 'a + TuringMachineSpec>: Encoder<'a, Self::Symbol, Guest>;

    fn encoder<'a, Guest: 'a + TuringMachineSpec>(&self, tm: &'a Guest)
        -> Self::Encoder<'a, Guest>;

    /// Returns true once per completed inner step. Decoding the tape at a
    /// tick should yield a valid snapshot of the guest machine after one
    /// more step than at the previous tick.
    fn is_tick_boundary(&self, prev_state: Self::State, new_state: Self::State) -> bool;
}
