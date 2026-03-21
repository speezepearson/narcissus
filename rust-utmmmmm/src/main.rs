use std::{rc::Rc, sync::LazyLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dir {
    Left,
    Right,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct State(u8);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Symbol(u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TuringMachineSpec {
    initial: State,
    blank: Symbol, // not actually used in the infinite-tape case, but the UTM *should* be able to
    accept: State,
    transition_matrix: [Option<(State, Symbol, Dir)>; 1 << 16],
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunningTuringMachine<'a> {
    spec: &'a TuringMachineSpec,
    state: State,
    pos: usize,
    tape: Vec<Symbol>,
}

const UTM_SPEC: LazyLock<&'static TuringMachineSpec> = LazyLock::new(|| {
    todo!("equivalent to the TypeScript `myUtmSpec`");
});

fn encode_tape(tape: &Vec<Symbol>) -> Vec<Symbol> {
    todo!("roughly equivalent to the TypeScript `makeEncodeTapeOverlayBackground`")
}

fn build_running_utm(m: &RunningTuringMachine) -> RunningTuringMachine<'static> {
    let mut tape = build_header(&m.spec, vec![]);
    tape.extend(encode_tape(&m.tape));
    RunningTuringMachine {
        spec: (*UTM_SPEC),
        state: m.spec.initial,
        pos: 0,
        tape,
    }
}

fn build_header(spec: &TuringMachineSpec, optimization_hints: Vec<(State, Symbol)>) -> Vec<Symbol> {
    // equivalent to the TypeScript `buildHeader(myUtmSpec, myUtmSpec.initial, optimizationHints)`
    todo!()
}

const INFINITE_UTM_TAPE_HEADER: LazyLock<&Vec<Symbol>> = LazyLock::new(|| {
    todo!("see TypeScript infiniteUtmTapeBackground");
});
fn extend_infinite_utm_tape(tape: &mut Vec<Symbol>) {
    // Do not overwrite any existing values.
    todo!(
        "see TypeScript `infiniteUtmTapeBackground`; except increase the tape's length by a good chunk, not just one symbol at a time."
    )
}

fn step_turing_machine(
    m: &mut RunningTuringMachine,
    extend_tape: impl Fn(&mut Vec<Symbol>, Symbol),
) -> bool {
    while m.pos >= m.tape.len() {
        extend_tape(&mut m.tape, m.spec.blank);
    }
    let symbol = m.tape[m.pos];
    if let Some((next_state, next_symbol, dir)) =
        m.spec.transition_matrix[((m.state.0 as usize) << 8) | (symbol.0 as usize)]
    {
        m.state = next_state;
        m.tape[m.pos] = next_symbol;
        m.pos = match dir {
            Dir::Left => m.pos.saturating_sub(1),
            Dir::Right => m.pos + 1,
        };
        true
    } else {
        false
    }
}

fn main() {
    let mut machine = RunningTuringMachine {
        spec: *UTM_SPEC,
        state: UTM_SPEC.initial,
        pos: 0,
        tape: vec![],
    };
    let mut steps = 0;
    let extend_tape = |tape: &mut Vec<Symbol>, _blank: Symbol| {
        extend_infinite_utm_tape(tape);
    };
    loop {
        steps += 1;
        if !step_turing_machine(&mut machine, extend_tape) {
            break;
        }
    }
    println!(
        "halted in state {:?} ({}) after {steps} steps",
        machine.state,
        if machine.state == machine.spec.accept {
            "accept"
        } else {
            "reject"
        }
    );
}
