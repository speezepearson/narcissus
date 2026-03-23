use std::collections::HashMap;

use crate::tm::{Dir, RunningTuringMachine, TapeExtender, TuringMachineSpec};

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

/// Wraps a `TapeExtender<Guest::Symbol>` to produce `CSymbol` values,
/// keeping a shadow tape in the original symbol type.
pub struct CompiledTapeExtender<Guest: TuringMachineSpec> {
    shadow: Vec<Guest::Symbol>,
    sym_to_csym: HashMap<Guest::Symbol, CSymbol>,
    inner: Box<dyn TapeExtender<Guest::Symbol>>,
}

impl<Guest: TuringMachineSpec> CompiledTapeExtender<Guest> {
    pub fn new(
        compiled: &CompiledTuringMachineSpec<Guest>,
        inner: Box<dyn TapeExtender<Guest::Symbol>>,
    ) -> Self {
        let sym_to_csym = compiled
            .original_symbols
            .iter()
            .enumerate()
            .map(|(i, &s)| (s, CSymbol(i as u8)))
            .collect();
        Self {
            shadow: Vec::new(),
            sym_to_csym,
            inner,
        }
    }

    pub fn extend(&mut self, tape: &mut Vec<CSymbol>, min_size: usize) {
        self.inner.extend(&mut self.shadow, min_size);
        tape.extend(self.shadow[tape.len()..].iter().map(|s| self.sym_to_csym[s]));
    }

    /// Get the shadow tape in original guest symbols.
    #[allow(dead_code)]
    pub fn shadow_tape(&self) -> &[Guest::Symbol] {
        &self.shadow
    }
}

impl<Guest: TuringMachineSpec> TapeExtender<CSymbol> for CompiledTapeExtender<Guest> {
    fn extend(&mut self, tape: &mut Vec<CSymbol>, min_size: usize) {
        self.inner.extend(&mut self.shadow, min_size);
        while tape.len() < self.shadow.len() {
            let sym = self.shadow[tape.len()];
            tape.push(self.sym_to_csym[&sym]);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{tm::run_tm, toy_machines::CHECK_PALINDROME_SPEC};

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
        use crate::infinity::InfiniteTapeExtender;
        use crate::tm::{step, TapeExtender};
        use crate::utm::UTM_SPEC;

        let utm = &*UTM_SPEC;
        let compiled = CompiledTuringMachineSpec::compile(utm).unwrap();

        // Interpreted: run 1000 steps with InfiniteTapeExtender
        let mut interp_tm = RunningTuringMachine::new(utm);
        InfiniteTapeExtender.extend(&mut interp_tm.tape, 1);
        for _ in 0..1000 {
            if interp_tm.pos >= interp_tm.tape.len() {
                InfiniteTapeExtender.extend(&mut interp_tm.tape, interp_tm.pos + 1);
            }
            step(&mut interp_tm);
        }

        // Compiled: run 1000 steps with CompiledTapeExtender
        let mut compiled_tm = RunningTuringMachine::new(&compiled);
        let mut extender = CompiledTapeExtender::new(&compiled, Box::new(InfiniteTapeExtender));
        extender.extend(&mut compiled_tm.tape, 1);
        for _ in 0..1000 {
            if compiled_tm.pos >= compiled_tm.tape.len() {
                extender.extend(&mut compiled_tm.tape, compiled_tm.pos + 1);
            }
            step(&mut compiled_tm);
        }

        let decompiled = compiled.decompile(&compiled_tm);
        assert_eq!(interp_tm.state, decompiled.state);
        assert_eq!(interp_tm.pos, decompiled.pos);
        assert_eq!(interp_tm.tape, decompiled.tape);
    }

    #[test]
    fn compiled_extender_matches_original() {
        use crate::infinity::InfiniteTapeExtender;
        use crate::tm::TapeExtender;
        use crate::utm::{Symbol, UTM_SPEC};

        let utm = &*UTM_SPEC;
        let compiled = CompiledTuringMachineSpec::compile(utm).unwrap();

        // Extend original tape to 1000
        let mut original_tape: Vec<Symbol> = Vec::new();
        InfiniteTapeExtender.extend(&mut original_tape, 1000);

        // Extend compiled tape to 1000
        let mut compiled_extender =
            CompiledTapeExtender::new(&compiled, Box::new(InfiniteTapeExtender));
        let mut compiled_tape: Vec<CSymbol> = Vec::new();
        compiled_extender.extend(&mut compiled_tape, 1000);

        // Shadow tape should match the original
        assert_eq!(compiled_extender.shadow_tape(), &original_tape[..]);

        // Decompiling each CSymbol should give back the original Symbol
        let decompiled: Vec<Symbol> = compiled_tape
            .iter()
            .map(|cs| compiled.original_symbols[cs.0 as usize])
            .collect();
        assert_eq!(decompiled, original_tape);
    }
}
