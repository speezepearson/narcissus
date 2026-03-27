// ════════════════════════════════════════════════════════════════════
// Infinite UTM tape: encode a UTM simulating itself (fixed point)
// ════════════════════════════════════════════════════════════════════

use std::{cell::RefCell,};

use crate::{
    compiled::{CSymbol, CompiledTuringMachineSpec}, gen_utm::UtmSpec as _, tm::{RunningTuringMachine, SimpleTuringMachineSpec}, utm::{MyUtmSpec, State, Symbol}
};

pub struct InfiniteTape<'a> {
    spec: &'a MyUtmSpec,
    realized: RefCell<Vec<Symbol>>,
}

impl<'a> InfiniteTape<'a> {
    pub fn new(spec: &'a MyUtmSpec) -> Self {
        Self {
            spec,
            realized: RefCell::new(Vec::new()),
        }
    }

    pub fn get(&self, index: usize) -> Symbol {
        self.extend_to(index);
        self.realized.borrow()[index]
    }

    pub fn iter_forever(&self) -> impl Iterator<Item = Symbol> + '_ {
        (0..).map(|i| self.get(i))
    }

    pub fn extend(&self, dst: &mut Vec<Symbol>, index: usize) {
        if dst.len() >= index {
            return;
        }
        self.extend_to(index);
        let cache = self.realized.borrow();
        dst.extend_from_slice(&cache[dst.len()..index]);
    }

    pub fn extend_compiled(
        &self,
        dst: &mut Vec<CSymbol>,
        index: usize,
        spec: &CompiledTuringMachineSpec<SimpleTuringMachineSpec<State, Symbol>>,
    ) {
        if dst.len() >= index {
            return;
        }
        self.extend_to(index);
        let cache = self.realized.borrow();
        dst.extend(
            cache[dst.len()..index]
                .iter()
                .map(|&s| spec.compile_symbol(s)),
        );
    }

    fn extend_to(&self, index: usize) {
        while index >= self.realized.borrow().len() {
            let mut rtm = RunningTuringMachine::new(self.spec);
            rtm.tape = self.realized.take();
            self.realized.replace(self.spec.encode(&rtm));
        }

    }
}

#[cfg(test)]
mod tests {

    use crate::utm::make_utm_spec;

    use super::*;

    #[test]
    fn test_is_self_similar() {
        let spec = make_utm_spec();
        let inf = InfiniteTape::new(&spec);

        let header_len = spec.encode(&RunningTuringMachine::new(&spec)).len() + 10;

        let mut l0_tape = vec![];
        inf.extend(&mut l0_tape, 200 * header_len);
        let l1 = spec.decode(&spec, &l0_tape).unwrap();
        assert_eq!(l1.tape[..header_len], l0_tape[..header_len]);
        let l2 = spec.decode(&spec, &l1.tape).unwrap();
        assert_eq!(l2.tape[..header_len], l1.tape[..header_len]);
        let l3 = spec.decode(&spec, &l2.tape).unwrap();
        assert_eq!(l3.tape[..header_len], l2.tape[..header_len]);
    }
}