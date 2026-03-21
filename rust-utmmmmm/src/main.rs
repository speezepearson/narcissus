#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dir {
    Left,
    Right,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct State(u8);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Symbol(u8);

struct TuringMachineSpec {
    initial: State,
    blank: Symbol,  // not actually used in the infinite-tape case, but the UTM *should* be able to 
    accept: State,
    transition_matrix: [Option<(State, Symbol, Dir)>; 1<<16],
}

fn build_utm_spec() -> TuringMachineSpec {
    // equivalent to the TypeScript `myUtmSpec`
    todo!()
}


fn build_header(spec: TuringMachineSpec, optimization_hints: Vec<(State, Symbol)>) -> Vec<Symbol> {
    // equivalent to the TypeScript `buildHeader(myUtmSpec, myUtmSpec.initial, optimizationHints)`
    todo!()
}

fn extend_infinite_utm_tape(tape: &mut Vec<Symbol>) {
    let header = build_header(build_utm_spec(), todo!("see TypeScript infiniteUtmTapeBackground")); // ideally we would just compute this once

    // Equivalent to the TypeScript `infiniteUtmTapeBackground`.
    // Increase the tape's length by some substantial amount.
    // Do not overwrite any existing values.
    todo!()
}


fn main() {
    let spec = build_utm_spec();
    let mut state = spec.initial;
    let mut pos: usize = 0;
    let mut tape: Vec<Symbol> = vec![];
    let mut steps = 0;
    loop {
        steps += 1;
        while pos >= tape.len() {
            extend_infinite_utm_tape(&mut tape);
        }

        let symbol = tape[pos];
        if let Some((next_state, next_symbol, dir)) = spec.transition_matrix[((state.0 as usize) << 8) | (symbol.0 as usize)] {
            state = next_state;
            tape[pos] = next_symbol;
            pos = match dir {
                Dir::Left => pos.saturating_sub(1),
                Dir::Right => pos + 1,
            };
        } else {
            break;
        }
    }
    println!("halted in state {state:?} ({}) after {steps} steps", if state == spec.accept { "accept" } else { "reject" });
}
