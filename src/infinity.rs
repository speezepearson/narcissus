// ════════════════════════════════════════════════════════════════════
// Infinite UTM tape: encode a UTM simulating itself (fixed point)
//
// The tape is: HEADER ^cell0 ,cell1 ,cell2 ...
// where cell_k encodes tape[k]. Since the guest is the UTM itself,
// tape[k] for k < |HEADER| is the k-th header symbol, and for
// k >= |HEADER| it's a tape-section symbol (marker or bit).
// Each cell_k only depends on tape[k] which is already computed
// (since k < HEADER_LEN + k * CELL_WIDTH), so this is a well-defined
// fixed point we can compute left-to-right.
// ════════════════════════════════════════════════════════════════════

use std::cell::RefCell;
use std::collections::HashMap;

use crate::{
    compiled::{CSymbol, CompiledTuringMachineSpec},
    tm::{RunningTuringMachine, SimpleTuringMachineSpec},
    utm::{num_bits, MyUtmSpec, MyUtmSpecOptimizationHints, State, Symbol},
};

pub struct InfiniteTape {
    header: Vec<Symbol>,
    sym_to_idx: HashMap<Symbol, usize>,
    n_sym_bits: usize,
    cell_width: usize, // 1 (marker) + n_sym_bits
    realized: RefCell<Vec<Symbol>>,
}

impl InfiniteTape {
    pub fn new(
        spec: &MyUtmSpec,
        optimization_hints: &MyUtmSpecOptimizationHints<MyUtmSpec>,
    ) -> Self {
        // Compute the header: everything before the ^ in the encoded tape
        let dummy = spec.encode_optimized(&RunningTuringMachine::new(spec), optimization_hints);
        let caret_pos = dummy
            .iter()
            .position(|&s| s == Symbol::Caret)
            .expect("encoded tape should contain ^");
        let header = dummy[..caret_pos].to_vec();

        let sym_to_idx: HashMap<Symbol, usize> = optimization_hints
            .symbol_encodings
            .iter()
            .map(|(&s, &i)| (s, i))
            .collect();
        let n_sym_bits = num_bits(sym_to_idx.len());
        let cell_width = 1 + n_sym_bits;

        Self {
            header,
            sym_to_idx,
            n_sym_bits,
            cell_width,
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
        let mut cache = self.realized.borrow_mut();
        if index < cache.len() {
            return;
        }

        let header = &self.header;
        let header_len = header.len();
        let sym_to_idx = &self.sym_to_idx;
        let n_sym_bits = self.n_sym_bits;
        let cell_width = self.cell_width;

        while cache.len() <= index {
            let pos = cache.len();
            if pos < header_len {
                cache.push(header[pos]);
            } else {
                let offset = pos - header_len;
                if offset % cell_width == 0 {
                    // Marker position
                    let cell_index = offset / cell_width;
                    cache.push(if cell_index == 0 {
                        Symbol::Caret
                    } else {
                        Symbol::Comma
                    });
                } else {
                    // Bit position: encode cache[cell_index]
                    let cell_index = offset / cell_width;
                    let bit_offset = offset % cell_width - 1;

                    let sym = cache[cell_index]; // always available: cell_index < pos
                    let sym_idx = sym_to_idx[&sym];
                    let bit = (sym_idx >> (n_sym_bits - 1 - bit_offset)) & 1;
                    cache.push(if bit == 1 { Symbol::One } else { Symbol::Zero });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{gen_utm::UtmSpec, utm::make_utm_spec};

    use super::*;

    #[test]
    fn test_is_self_similar() {
        let spec = make_utm_spec();
        let optimization_hints = MyUtmSpecOptimizationHints::guess(&spec);
        let inf = InfiniteTape::new(&spec, &optimization_hints);

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
