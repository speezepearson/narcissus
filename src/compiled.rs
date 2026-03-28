use std::collections::HashMap;

use crate::gen_utm::UtmSpec;
use crate::tm::{Dir, RunningTuringMachine, TuringMachineSpec};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CState(pub u8);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CSymbol(pub u8);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledTuringMachineSpec<'a, Guest: TuringMachineSpec> {
    pub n_states: u8,
    pub n_symbols: u8,
    pub initial: CState,
    pub accept: Vec<CState>,
    pub blank: CSymbol,
    pub transitions: [Option<(CState, CSymbol, Dir)>; 1 << 16],

    pub guest: &'a Guest,
    pub original_states: Vec<Guest::State>,
    pub original_symbols: Vec<Guest::Symbol>,
}

impl<'a, Guest: TuringMachineSpec> TuringMachineSpec for CompiledTuringMachineSpec<'a, Guest> {
    type State = CState;
    type Symbol = CSymbol;
    fn initial(&self) -> Self::State {
        self.initial
    }
    fn blank(&self) -> Self::Symbol {
        self.blank
    }
    fn is_accepting(&self, state: Self::State) -> bool {
        self.accept.contains(&state)
    }
    fn get_transition(
        &self,
        state: Self::State,
        symbol: Self::Symbol,
    ) -> Option<(Self::State, Self::Symbol, Dir)> {
        self.transitions[((state.0 as usize) << 8) | (symbol.0 as usize)]
    }
    fn iter_states(&self) -> impl Iterator<Item = Self::State> {
        (0..self.n_states).map(|i| CState(i as u8))
    }
    fn iter_symbols(&self) -> impl Iterator<Item = Self::Symbol> {
        (0..self.n_symbols).map(|i| CSymbol(i as u8))
    }
    fn iter_rules(
        &self,
    ) -> impl Iterator<Item = (Self::State, Self::Symbol, Self::State, Self::Symbol, Dir)> {
        self.transitions
            .iter()
            .enumerate()
            .filter_map(|(i, rule)| match rule {
                Some((nst, sym, dir)) => Some((i, (nst, sym, dir))),
                None => None,
            })
            .map(|(i, (&nst, &sym, &dir))| {
                (
                    CState((i >> 8) as u8),
                    CSymbol((i & 0xFF) as u8),
                    nst,
                    sym,
                    dir,
                )
            })
        // (0..1 << 16).map(|i| {
        //     let state = CState(i >> 8);
        //     let symbol = CSymbol(i & 0xFF);
        //     let next_state = CState((i >> 8) & 0xFF);
        //     let next_symbol = CSymbol(i & 0xFF);
        //     let dir = if i & 0x8000 == 0 {
        //         Dir::Left
        //     } else {
        //         Dir::Right
        //     };
        //     (state, symbol, next_state, next_symbol, dir)
        // })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    TooManyStates,
    TooManySymbols,
}
impl<'a, Guest: TuringMachineSpec> CompiledTuringMachineSpec<'a, Guest> {
    pub fn compile(guest: &'a Guest) -> Result<CompiledTuringMachineSpec<'a, Guest>, CompileError> {
        let original_states: Vec<Guest::State> = guest.iter_states().collect();
        if original_states.len() > 256 {
            return Err(CompileError::TooManyStates);
        }
        let state_to_idx = original_states
            .iter()
            .enumerate()
            .map(|(i, state)| (*state, CState(i as u8)))
            .collect::<HashMap<_, _>>();

        let original_symbols: Vec<Guest::Symbol> = guest.iter_symbols().collect();
        if original_symbols.len() > 256 {
            return Err(CompileError::TooManySymbols);
        }
        let sym_to_idx = original_symbols
            .iter()
            .enumerate()
            .map(|(i, symbol)| (*symbol, CSymbol(i as u8)))
            .collect::<HashMap<_, _>>();

        let mut transitions = [None; 1 << 16];
        for (state, symbol, next_state, next_symbol, dir) in guest.iter_rules() {
            let idx = ((state_to_idx[&state].0 as usize) << 8) | (sym_to_idx[&symbol].0 as usize);
            transitions[idx] = Some((state_to_idx[&next_state], sym_to_idx[&next_symbol], dir));
        }

        Ok(CompiledTuringMachineSpec {
            n_states: state_to_idx.len() as u8,
            n_symbols: sym_to_idx.len() as u8,
            initial: state_to_idx[&guest.initial()],
            accept: guest
                .iter_states()
                .filter(|s| guest.is_accepting(*s))
                .map(|s| state_to_idx[&s])
                .collect(),
            blank: sym_to_idx[&guest.blank()],
            transitions,

            guest: guest,
            original_states,
            original_symbols,
        })
    }

    pub fn compile_state(&self, state: Guest::State) -> CState {
        CState(
            self.original_states
                .iter()
                .position(|&s| s == state)
                .expect("every state should be in the compiled state list") as u8,
        )
    }
    pub fn compile_symbol(&self, symbol: Guest::Symbol) -> CSymbol {
        CSymbol(
            self.original_symbols
                .iter()
                .position(|&s| s == symbol)
                .expect("every symbol should be in the compiled symbol list") as u8,
        )
    }
    pub fn decompile_symbol(&self, symbol: CSymbol) -> Guest::Symbol {
        self.original_symbols[symbol.0 as usize]
    }
    pub fn decompile_state(&self, state: CState) -> Guest::State {
        self.original_states[state.0 as usize]
    }

    pub fn decompile(&self, tm: &RunningTuringMachine<Self>) -> RunningTuringMachine<'_, Guest> {
        RunningTuringMachine {
            spec: self.guest,
            state: self.original_states[tm.state.0 as usize],
            pos: tm.pos,
            tape: tm
                .tape
                .iter()
                .map(|s| self.original_symbols[s.0 as usize])
                .collect(),
        }
    }
}

impl<'a, Guest: UtmSpec> UtmSpec for CompiledTuringMachineSpec<'a, Guest> {
    fn encode<Inner: TuringMachineSpec>(
        &self,
        tm: &RunningTuringMachine<Inner>,
    ) -> Vec<CSymbol> {
        let guest_tape = self.guest.encode(tm);
        guest_tape
            .iter()
            .map(|&s| self.compile_symbol(s))
            .collect()
    }

    fn decode<'b, Inner: TuringMachineSpec>(
        &self,
        inner: &'b Inner,
        tape: &[CSymbol],
    ) -> Result<RunningTuringMachine<'b, Inner>, String> {
        let guest_tape: Vec<Guest::Symbol> = tape
            .iter()
            .map(|&s| self.decompile_symbol(s))
            .collect();
        self.guest.decode(inner, &guest_tape)
    }

    fn at_tick(&self, state: CState, symbol: CSymbol) -> bool {
        self.guest
            .at_tick(self.decompile_state(state), self.decompile_symbol(symbol))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        optimization_hints::make_my_utm_self_optimization_hints, tm::run_tm,
        toy_machines::CHECK_PALINDROME_SPEC, utm::make_utm_spec,
    };

    use super::*;

    #[test]
    fn compile_fails_if_too_many_states() {}

    #[test]
    fn compile_run_decompile_is_same_as_run() {
        let base = &*CHECK_PALINDROME_SPEC;
        let mut base_tm = RunningTuringMachine::new(base);
        let expected_status = run_tm(&mut base_tm, 1000, None).unwrap();

        let compiled = CompiledTuringMachineSpec::compile(base).unwrap();
        let mut compiled_tm = RunningTuringMachine::new(&compiled);
        let actual_status = run_tm(&mut compiled_tm, 1000, None).unwrap();

        assert_eq!(actual_status, expected_status);
        assert_eq!(base_tm, compiled.decompile(&compiled_tm));
    }

    #[test]
    fn compiled_extender_run_matches_interpreted() {
        use crate::infinity::InfiniteTape;
        use crate::tm::step;

        let optimization_hints = make_my_utm_self_optimization_hints();
        let utm_spec = make_utm_spec();
        let compiled = CompiledTuringMachineSpec::compile(&utm_spec).unwrap();
        let background = InfiniteTape::new(&utm_spec, &optimization_hints);

        // Interpreted: run 1000 steps with InfiniteTape
        let mut interp_tm = RunningTuringMachine::new(&utm_spec);
        for _ in 0..1000 {
            if interp_tm.pos >= interp_tm.tape.len() {
                background.extend(&mut interp_tm.tape, interp_tm.pos + 1);
            }
            step(&mut interp_tm);
        }

        // Compiled: run 1000 steps with CompiledTapeExtender
        let mut compiled_tm = RunningTuringMachine::new(&compiled);
        for _ in 0..1000 {
            if compiled_tm.pos >= compiled_tm.tape.len() {
                background.extend_compiled(&mut compiled_tm.tape, compiled_tm.pos + 1, &compiled);
            }
            step(&mut compiled_tm);
        }

        let decompiled = compiled.decompile(&compiled_tm);
        assert_eq!(interp_tm.state, decompiled.state);
        assert_eq!(interp_tm.pos, decompiled.pos);
        assert_eq!(interp_tm.tape, decompiled.tape);
    }
}
