// ════════════════════════════════════════════════════════════════════
// Toy machines for testing the UTM
// ════════════════════════════════════════════════════════════════════

use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use crate::tm::{Dir, SimpleTuringMachineSpec};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccImmState {
    Init,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccImmSymbol {
    Blank,
}
pub static ACCEPT_IMMEDIATELY_SPEC: LazyLock<SimpleTuringMachineSpec<AccImmState, AccImmSymbol>> =
    LazyLock::new(|| SimpleTuringMachineSpec {
        initial: AccImmState::Init,
        blank: AccImmSymbol::Blank,
        accepting: HashSet::from([AccImmState::Init]),
        transitions: HashMap::new(),
        all_states: vec![AccImmState::Init],
        all_symbols: vec![AccImmSymbol::Blank],
    });

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RejImmState {
    Init,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RejImmSymbol {
    Blank,
}
pub static REJECT_IMMEDIATELY_SPEC: LazyLock<SimpleTuringMachineSpec<RejImmState, RejImmSymbol>> =
    LazyLock::new(|| SimpleTuringMachineSpec {
        initial: RejImmState::Init,
        blank: RejImmSymbol::Blank,
        accepting: HashSet::new(),
        transitions: HashMap::new(),
        all_states: vec![RejImmState::Init],
        all_symbols: vec![RejImmSymbol::Blank],
    });

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub     enum Letter {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
}
fn all_letters() -> [Letter; 26] {
    [
        Letter::A,
        Letter::B,
        Letter::C,
        Letter::D,
        Letter::E,
        Letter::F,
        Letter::G,
        Letter::H,
        Letter::I,
        Letter::J,
        Letter::K,
        Letter::L,
        Letter::M,
        Letter::N,
        Letter::O,
        Letter::P,
        Letter::Q,
        Letter::R,
        Letter::S,
        Letter::T,
        Letter::U,
        Letter::V,
        Letter::W,
        Letter::X,
        Letter::Y,
        Letter::Z,
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckPalindromeState {
    Start,
    Accept,
    SeekL,
    SeekR(Letter),
    Check(Letter),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckPalindromeSymbol {
    Blank,
    Letter(Letter),
}

pub static CHECK_PALINDROME_SPEC: LazyLock<
    SimpleTuringMachineSpec<CheckPalindromeState, CheckPalindromeSymbol>,
> = LazyLock::new(|| {
    use CheckPalindromeState::*;
    use CheckPalindromeSymbol::*;

    let mut transitions: HashMap<
        (CheckPalindromeState, CheckPalindromeSymbol),
        (CheckPalindromeState, CheckPalindromeSymbol, Dir),
    > = HashMap::new();

    // start rules
    transitions.insert((Start, Blank), (Accept, Blank, Dir::Right));
    for letter in all_letters() {
        transitions.insert((Start, Letter(letter)), (SeekR(letter), Blank, Dir::Right));
    }

    for letter in all_letters() {
        transitions.insert((SeekR(letter), Blank), (Check(letter), Blank, Dir::Left));
        for l2 in all_letters() {
            transitions.insert((SeekR(letter), Letter(l2)), (SeekR(letter), Letter(l2), Dir::Right));
        }
    }

    // check_x rules
    for letter in all_letters() {
        transitions.insert((Check(letter), Blank), (Accept, Blank, Dir::Right));
        transitions.insert((Check(letter), Letter(letter)), (SeekL, Blank, Dir::Left));
    }

    // seekL rules
    transitions.insert((SeekL, Blank), (Start, Blank, Dir::Right));
    for letter in all_letters() {
        transitions.insert((SeekL, Letter(letter)), (SeekL, Letter(letter), Dir::Left));
    }

    SimpleTuringMachineSpec {
        initial: CheckPalindromeState::Start,
        accepting: HashSet::from([CheckPalindromeState::Accept]),
        blank: CheckPalindromeSymbol::Blank,
        transitions: HashMap::from(transitions),
        all_states: {
            let mut v = vec![
            CheckPalindromeState::Start,
            CheckPalindromeState::Accept,
            CheckPalindromeState::SeekL,
            ];
            for letter in all_letters() {
                v.push(CheckPalindromeState::SeekR(letter));
                v.push(CheckPalindromeState::Check(letter));
            }
            v
        },
        all_symbols: {
            let mut v = vec![
            CheckPalindromeSymbol::Blank,
            ];
            for letter in all_letters() {
                v.push(CheckPalindromeSymbol::Letter(letter));
            }
            v
        },
    }
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DoubleXState {
    Start,
    FindX,
    GoRight,
    GoBack,
    CleanL,
    CleanR,
    Done,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DoubleXSymbol {
    Blank,
    Dollar,
    X,
    Y,
    Z,
}
pub static DOUBLE_X_SPEC: LazyLock<SimpleTuringMachineSpec<DoubleXState, DoubleXSymbol>> =
    LazyLock::new(|| {
        use DoubleXState::*;
        use DoubleXSymbol::*;
        SimpleTuringMachineSpec {
            initial: DoubleXState::Start,
            accepting: HashSet::from([DoubleXState::Done]),
            blank: DoubleXSymbol::Blank,
            transitions: HashMap::from([
                ((Start, Dollar), (FindX, Dollar, Dir::Right)),
                ((FindX, X), (GoRight, Y, Dir::Right)),
                ((FindX, Y), (FindX, Y, Dir::Right)),
                ((FindX, Z), (CleanL, Z, Dir::Left)),
                ((FindX, Blank), (Done, Blank, Dir::Left)),
                ((GoRight, X), (GoRight, X, Dir::Right)),
                ((GoRight, Z), (GoRight, Z, Dir::Right)),
                ((GoRight, Blank), (GoBack, Z, Dir::Left)),
                ((GoBack, X), (GoBack, X, Dir::Left)),
                ((GoBack, Y), (GoBack, Y, Dir::Left)),
                ((GoBack, Z), (GoBack, Z, Dir::Left)),
                ((GoBack, Dollar), (FindX, Dollar, Dir::Right)),
                ((CleanL, Y), (CleanL, X, Dir::Left)),
                ((CleanL, Dollar), (CleanR, Dollar, Dir::Right)),
                ((CleanR, X), (CleanR, X, Dir::Right)),
                ((CleanR, Z), (CleanR, X, Dir::Right)),
                ((CleanR, Blank), (Done, Blank, Dir::Left)),
            ]),
            all_states: vec![Start, FindX, GoRight, GoBack, CleanL, CleanR, Done],
            all_symbols: vec![Blank, Dollar, X, Y, Z],
        }
    });

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlipBitsState {
    Flip,
    Done,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlipBitsSymbol {
    Blank,
    Zero,
    One,
}
pub static FLIP_BITS_SPEC: LazyLock<SimpleTuringMachineSpec<FlipBitsState, FlipBitsSymbol>> =
    LazyLock::new(|| {
        use FlipBitsState::*;
        use FlipBitsSymbol::*;
        SimpleTuringMachineSpec {
            initial: Flip,
            accepting: HashSet::from([Done]),
            blank: Blank,
            transitions: HashMap::from([
                ((Flip, Zero), (Flip, One, Dir::Right)),
                ((Flip, One), (Flip, Zero, Dir::Right)),
                ((Flip, Blank), (Done, Blank, Dir::Left)),
            ]),
            all_states: vec![Flip, Done],
            all_symbols: vec![Blank, Zero, One],
        }
    });

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tm::{run_tm, HaltReason, RunningTuringMachine};

    #[test]
    fn accept_immediately() {
        use AccImmSymbol::*;
        let mut tm = RunningTuringMachine::new(&*ACCEPT_IMMEDIATELY_SPEC);
        tm.tape = vec![Blank];
        let result = run_tm(&mut tm, 1000, None);
        assert!(matches!(result, Ok(HaltReason::Accepted { .. })));
    }

    #[test]
    fn reject_immediately() {
        use RejImmSymbol::*;
        let mut tm = RunningTuringMachine::new(&*REJECT_IMMEDIATELY_SPEC);
        tm.tape = vec![Blank];
        let result = run_tm(&mut tm, 1000, None);
        assert!(matches!(result, Ok(HaltReason::Rejected { .. })));
    }

    #[test]
    fn double_x() {
        use DoubleXSymbol::*;
        let mut tm = RunningTuringMachine::new(&*DOUBLE_X_SPEC);
        tm.tape = vec![Dollar, X, X, X];
        let result = run_tm(&mut tm, 1000, None);
        assert!(matches!(result, Ok(HaltReason::Accepted { .. })));
        while tm.tape.last() == Some(&Blank) {
            tm.tape.pop();
        }
        assert_eq!(tm.tape, vec![Dollar, X, X, X, X, X, X]);
    }

    #[test]
    fn check_palindrome_accepts_odd_length_palindrome() {
        use super::Letter::*;
        use CheckPalindromeSymbol::*;
        let mut tm = RunningTuringMachine::new(&*CHECK_PALINDROME_SPEC);
        tm.tape = [A, B, C, B, A].map(Letter).to_vec();
        let result = run_tm(&mut tm, 1000, None);
        assert!(matches!(result, Ok(HaltReason::Accepted { .. })));
    }

    #[test]
    fn check_palindrome_accepts_even_length_palindrome() {
        use super::Letter::*;
        use CheckPalindromeSymbol::*;
        let mut tm = RunningTuringMachine::new(&*CHECK_PALINDROME_SPEC);
        tm.tape = [A, B, B, A].map(Letter).to_vec();
        let result = run_tm(&mut tm, 1000, None);
        assert!(matches!(result, Ok(HaltReason::Accepted { .. })));
    }

    #[test]
    fn check_palindrome_rejects_non_palindrome() {
        use super::Letter::*;
        use CheckPalindromeSymbol::*;
        let mut tm = RunningTuringMachine::new(&*CHECK_PALINDROME_SPEC);
        tm.tape = [A, B, C, A].map(Letter).to_vec();
        let result = run_tm(&mut tm, 1000, None);
        assert!(matches!(result, Ok(HaltReason::Rejected { .. })));
    }
}
