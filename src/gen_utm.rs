use crate::tm::{RunningTuringMachine, TuringMachineSpec};

pub trait UtmSpec: TuringMachineSpec {
    fn encode<Guest: TuringMachineSpec>(
        &self,
        tm: &RunningTuringMachine<Guest>,
    ) -> Vec<Self::Symbol>;
    fn decode<'a, Guest: TuringMachineSpec>(
        &self,
        guest: &'a Guest,
        tape: &[Self::Symbol],
    ) -> Result<RunningTuringMachine<'a, Guest>, String>;

    /// Returns true when the UTM is at a "tick" boundary: once for a freshly
    /// created machine (before any steps), and once per completed inner step
    /// thereafter. Decoding the tape at a tick should yield a valid snapshot
    /// of the guest machine.
    fn at_tick(&self, state: Self::State, symbol: Self::Symbol) -> bool;
}
