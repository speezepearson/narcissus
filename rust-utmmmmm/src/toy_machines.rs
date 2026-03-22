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
    One,
}
pub static ACCEPT_IMMEDIATELY_SPEC: LazyLock<SimpleTuringMachineSpec<AccImmState, AccImmSymbol>> =
    LazyLock::new(|| SimpleTuringMachineSpec {
        initial: AccImmState::Init,
        blank: AccImmSymbol::Blank,
        accepting: HashSet::from([AccImmState::Init]),
        transitions: HashMap::from([(
            (AccImmState::Init, AccImmSymbol::Blank),
            (AccImmState::Init, AccImmSymbol::One, Dir::Right),
        )]),
        all_states: vec![AccImmState::Init],
        all_symbols: vec![AccImmSymbol::Blank, AccImmSymbol::One],
    });

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RejImmState {
    Init,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RejImmSymbol {
    Blank,
    One,
}
pub static REJECT_IMMEDIATELY_SPEC: LazyLock<SimpleTuringMachineSpec<RejImmState, RejImmSymbol>> =
    LazyLock::new(|| SimpleTuringMachineSpec {
        initial: RejImmState::Init,
        blank: RejImmSymbol::Blank,
        accepting: HashSet::new(),
        transitions: HashMap::from([(
            (RejImmState::Init, RejImmSymbol::Blank),
            (RejImmState::Init, RejImmSymbol::One, Dir::Right),
        )]),
        all_states: vec![RejImmState::Init],
        all_symbols: vec![RejImmSymbol::Blank, RejImmSymbol::One],
    });

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckPalindromeState {
    Start,
    Accept,
    SeekL,
    SeekRA,
    SeekRB,
    SeekRC,
    CheckA,
    CheckB,
    CheckC,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckPalindromeSymbol {
    Blank,
    A,
    B,
    C,
}

pub static CHECK_PALINDROME_SPEC: LazyLock<
    SimpleTuringMachineSpec<CheckPalindromeState, CheckPalindromeSymbol>,
> = LazyLock::new(|| {
    use CheckPalindromeState::*;
    use CheckPalindromeSymbol::*;

    let letter_triplets = [
        (A, CheckA, SeekRA),
        (B, CheckB, SeekRB),
        (C, CheckC, SeekRC),
    ];

    let mut transitions: HashMap<
        (CheckPalindromeState, CheckPalindromeSymbol),
        (CheckPalindromeState, CheckPalindromeSymbol, Dir),
    > = HashMap::new();

    // start rules
    transitions.insert((Start, Blank), (Accept, Blank, Dir::Right));
    transitions.insert((Start, A), (SeekRA, Blank, Dir::Right));
    transitions.insert((Start, B), (SeekRB, Blank, Dir::Right));
    transitions.insert((Start, C), (SeekRC, Blank, Dir::Right));

    for (_letter, check, seek) in letter_triplets {
        transitions.insert((seek, Blank), (check, Blank, Dir::Left));
        for (l2, _, _) in letter_triplets {
            transitions.insert((seek, l2), (seek, l2, Dir::Right));
        }
    }

    // check_x rules
    for (letter, check, _seek) in letter_triplets {
        transitions.insert((check, Blank), (Accept, Blank, Dir::Right));
        transitions.insert((check, letter), (SeekL, Blank, Dir::Left));
    }

    // seekL rules
    transitions.insert((SeekL, Blank), (Start, Blank, Dir::Right));
    for (letter, _, _) in letter_triplets {
        transitions.insert((SeekL, letter), (SeekL, letter, Dir::Left));
    }

    SimpleTuringMachineSpec {
        initial: CheckPalindromeState::Start,
        accepting: HashSet::from([CheckPalindromeState::Accept]),
        blank: CheckPalindromeSymbol::Blank,
        transitions: HashMap::from(transitions),
        all_states: vec![
            CheckPalindromeState::Start,
            CheckPalindromeState::Accept,
            CheckPalindromeState::SeekL,
            CheckPalindromeState::SeekRA,
            CheckPalindromeState::SeekRB,
            CheckPalindromeState::SeekRC,
            CheckPalindromeState::CheckA,
            CheckPalindromeState::CheckB,
            CheckPalindromeState::CheckC,
        ],
        all_symbols: vec![
            CheckPalindromeSymbol::Blank,
            CheckPalindromeSymbol::A,
            CheckPalindromeSymbol::B,
            CheckPalindromeSymbol::C,
        ],
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
        tm.tape = vec![Blank, One];
        let result = run_tm(&mut tm, 1000, None);
        assert!(matches!(result, Ok(HaltReason::Accepted { .. })));
    }

    #[test]
    fn reject_immediately() {
        use RejImmSymbol::*;
        let mut tm = RunningTuringMachine::new(&*REJECT_IMMEDIATELY_SPEC);
        tm.tape = vec![Blank, One];
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
        use CheckPalindromeSymbol::*;
        let mut tm = RunningTuringMachine::new(&*CHECK_PALINDROME_SPEC);
        tm.tape = vec![A, B, C, B, A];
        let result = run_tm(&mut tm, 1000, None);
        assert!(matches!(result, Ok(HaltReason::Accepted { .. })));
    }

    #[test]
    fn check_palindrome_accepts_even_length_palindrome() {
        use CheckPalindromeSymbol::*;
        let mut tm = RunningTuringMachine::new(&*CHECK_PALINDROME_SPEC);
        tm.tape = vec![A, B, B, A];
        let result = run_tm(&mut tm, 1000, None);
        assert!(matches!(result, Ok(HaltReason::Accepted { .. })));
    }

    #[test]
    fn check_palindrome_rejects_non_palindrome() {
        use CheckPalindromeSymbol::*;
        let mut tm = RunningTuringMachine::new(&*CHECK_PALINDROME_SPEC);
        tm.tape = vec![A, B, C, A];
        let result = run_tm(&mut tm, 1000, None);
        assert!(matches!(result, Ok(HaltReason::Rejected { .. })));
    }
}
