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
}
