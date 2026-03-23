// ════════════════════════════════════════════════════════════════════
// UTM core: types, constants, rule builder, encoding, infinite tape
// ════════════════════════════════════════════════════════════════════

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    sync::LazyLock,
};

use crate::tm::{Dir, RunningTuringMachine, SimpleTuringMachineSpec, TuringMachineSpec};

// ── Newtype wrappers for type safety ──
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    Accept,
    AcceptSeekHome,
    AccFinalHome,
    AccRestAcc,
    AccRestState,
    ApplyReadNst,
    ChkAccBack2acc,
    ChkAccC0,
    ChkAccC0Find,
    ChkAccC1,
    ChkAccC1Find,
    ChkAccDoRest,
    ChkAccDoRest2,
    ChkAccFailBit,
    ChkAccInit,
    ChkAccIntoAcc,
    ChkAccNextEntry,
    ChkAccOk,
    ChkAccOkAcc,
    ChkAccOkFind,
    ChkAccOkSkip,
    ChkAccRestState,
    CmpStC0,
    CmpStC0Find,
    CmpStC0Sk1,
    CmpStC1,
    CmpStC1Find,
    CmpStC1Sk1,
    CmpStFail,
    CmpStNextbit,
    CmpStOk,
    CmpStRead,
    CmpSymC0,
    CmpSymC0Fb,
    CmpSymC0Fh,
    CmpSymC0S1,
    CmpSymC0S2,
    CmpSymC0S3,
    CmpSymC1,
    CmpSymC1Fb,
    CmpSymC1Fh,
    CmpSymC1S1,
    CmpSymC1S2,
    CmpSymC1S3,
    CmpSymFail,
    CmpSymNb2,
    CmpSymNextbit,
    CmpSymOk,
    CmpSymRead,
    CpNstC0,
    CpNstC0S1,
    CpNstC0W,
    CpNstC1,
    CpNstC1S1,
    CpNstC1W,
    CpNstDone,
    CpNstNext,
    CpNstNext2,
    CpNstNext3,
    CpNstRestDo,
    CpNstRestNav,
    CpNstRestS1,
    CpNstRet,
    CpNsymC0,
    CpNsymC0Fb,
    CpNsymC0Fh,
    CpNsymC0S1,
    CpNsymC0S2,
    CpNsymC0S3,
    CpNsymC1,
    CpNsymC1Fb,
    CpNsymC1Fh,
    CpNsymC1S1,
    CpNsymC1S2,
    CpNsymC1S3,
    CpNsymDone,
    CpNsymFn2,
    CpNsymFn3,
    CpNsymFn4,
    CpNsymFnext,
    CpNsymNav,
    CpNsymNav2,
    CpNsymNav3,
    CpNsymRead,
    CpNsymRestNav,
    CpNsymRet,
    CpNsymRnDo,
    CpNsymRnFh,
    CpNsymRnS1,
    CpNsymRnS2,
    CpNsymRnS3,
    CpNsymSeek,
    DoneSeekHome,
    Init,
    InitSeekEnd,
    InitSkip,
    MarkRule,
    MarkRuleNoMatch,
    MlFindHead,
    MlMark,
    MlNav,
    MlRestore,
    MlS1,
    MlS2,
    MlS3,
    MoveLeft,
    MoveRight,
    MrExtBc0,
    MrExtBc1,
    MrExtBcNext,
    MrExtBcRet,
    MrExtendInit,
    MrExtH1,
    MrExtH2,
    MrExtH3,
    MrExtHome,
    MrExtReadBlank,
    MrExtRestBlank,
    MrExtToBlank,
    MrExtWriteHead,
    MrFindHead,
    MrNav,
    MrPlaceHead,
    MrS1,
    MrS2,
    MrS3,
    MrSkipCell,
    RdRead,
    RdSk2,
    RdSk3,
    RdSk4,
    RdSkipToDir,
    ReadDir,
    Reject,
    RejectSeekHome,
    RejFinalHome,
    RejRestAcc,
    RejRestState,
    SmcFh,
    SmcRestDone,
    SmcRestHead,
    SmcRestSym,
    SmcS1,
    SmcS2,
    SmcS3,
    SmcSkipSt,
    StfFindStar,
    StfGoPrev,
    StfRestoreRule,
    StfRestoreState,
    StfSkipRest,
    StMatchCleanup,
    StmBackToRule,
    StmGoLeft,
    StmGotoState,
    StmGsSk1,
    StmRestoreRule,
    StmRestoreState,
    SymfDeactivate,
    SymfRestHead,
    SymfRestSym,
    SymfSeekStar,
    SymfSkipRest,
    SymfSkipSt,
    SymMatchCleanup,
    SymSkipState,
}
const ALL_STATES: [State; 166] = [
    State::Accept,
    State::AcceptSeekHome,
    State::AccFinalHome,
    State::AccRestAcc,
    State::AccRestState,
    State::ApplyReadNst,
    State::ChkAccBack2acc,
    State::ChkAccC0,
    State::ChkAccC0Find,
    State::ChkAccC1,
    State::ChkAccC1Find,
    State::ChkAccDoRest,
    State::ChkAccDoRest2,
    State::ChkAccFailBit,
    State::ChkAccInit,
    State::ChkAccIntoAcc,
    State::ChkAccNextEntry,
    State::ChkAccOk,
    State::ChkAccOkAcc,
    State::ChkAccOkFind,
    State::ChkAccOkSkip,
    State::ChkAccRestState,
    State::CmpStC0,
    State::CmpStC0Find,
    State::CmpStC0Sk1,
    State::CmpStC1,
    State::CmpStC1Find,
    State::CmpStC1Sk1,
    State::CmpStFail,
    State::CmpStNextbit,
    State::CmpStOk,
    State::CmpStRead,
    State::CmpSymC0,
    State::CmpSymC0Fb,
    State::CmpSymC0Fh,
    State::CmpSymC0S1,
    State::CmpSymC0S2,
    State::CmpSymC0S3,
    State::CmpSymC1,
    State::CmpSymC1Fb,
    State::CmpSymC1Fh,
    State::CmpSymC1S1,
    State::CmpSymC1S2,
    State::CmpSymC1S3,
    State::CmpSymFail,
    State::CmpSymNb2,
    State::CmpSymNextbit,
    State::CmpSymOk,
    State::CmpSymRead,
    State::CpNstC0,
    State::CpNstC0S1,
    State::CpNstC0W,
    State::CpNstC1,
    State::CpNstC1S1,
    State::CpNstC1W,
    State::CpNstDone,
    State::CpNstNext,
    State::CpNstNext2,
    State::CpNstNext3,
    State::CpNstRestDo,
    State::CpNstRestNav,
    State::CpNstRestS1,
    State::CpNstRet,
    State::CpNsymC0,
    State::CpNsymC0Fb,
    State::CpNsymC0Fh,
    State::CpNsymC0S1,
    State::CpNsymC0S2,
    State::CpNsymC0S3,
    State::CpNsymC1,
    State::CpNsymC1Fb,
    State::CpNsymC1Fh,
    State::CpNsymC1S1,
    State::CpNsymC1S2,
    State::CpNsymC1S3,
    State::CpNsymDone,
    State::CpNsymFn2,
    State::CpNsymFn3,
    State::CpNsymFn4,
    State::CpNsymFnext,
    State::CpNsymNav,
    State::CpNsymNav2,
    State::CpNsymNav3,
    State::CpNsymRead,
    State::CpNsymRestNav,
    State::CpNsymRet,
    State::CpNsymRnDo,
    State::CpNsymRnFh,
    State::CpNsymRnS1,
    State::CpNsymRnS2,
    State::CpNsymRnS3,
    State::CpNsymSeek,
    State::DoneSeekHome,
    State::Init,
    State::InitSeekEnd,
    State::InitSkip,
    State::MarkRule,
    State::MarkRuleNoMatch,
    State::MlFindHead,
    State::MlMark,
    State::MlNav,
    State::MlRestore,
    State::MlS1,
    State::MlS2,
    State::MlS3,
    State::MoveLeft,
    State::MoveRight,
    State::MrExtBc0,
    State::MrExtBc1,
    State::MrExtBcNext,
    State::MrExtBcRet,
    State::MrExtendInit,
    State::MrExtH1,
    State::MrExtH2,
    State::MrExtH3,
    State::MrExtHome,
    State::MrExtReadBlank,
    State::MrExtRestBlank,
    State::MrExtToBlank,
    State::MrExtWriteHead,
    State::MrFindHead,
    State::MrNav,
    State::MrPlaceHead,
    State::MrS1,
    State::MrS2,
    State::MrS3,
    State::MrSkipCell,
    State::RdRead,
    State::RdSk2,
    State::RdSk3,
    State::RdSk4,
    State::RdSkipToDir,
    State::ReadDir,
    State::Reject,
    State::RejectSeekHome,
    State::RejFinalHome,
    State::RejRestAcc,
    State::RejRestState,
    State::SmcFh,
    State::SmcRestDone,
    State::SmcRestHead,
    State::SmcRestSym,
    State::SmcS1,
    State::SmcS2,
    State::SmcS3,
    State::SmcSkipSt,
    State::StfFindStar,
    State::StfGoPrev,
    State::StfRestoreRule,
    State::StfRestoreState,
    State::StfSkipRest,
    State::StMatchCleanup,
    State::StmBackToRule,
    State::StmGoLeft,
    State::StmGotoState,
    State::StmGsSk1,
    State::StmRestoreRule,
    State::StmRestoreState,
    State::SymfDeactivate,
    State::SymfRestHead,
    State::SymfRestSym,
    State::SymfSeekStar,
    State::SymfSkipRest,
    State::SymfSkipSt,
    State::SymMatchCleanup,
    State::SymSkipState,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    Blank,
    Zero,
    One,
    X,
    Y,
    Hash,
    Pipe,
    Semi,
    Comma,
    Caret,
    L,
    D,
    Dot,
    Star,
    Gt,
    Dollar,
}
impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Symbol::Blank => "_",
                Symbol::Zero => "0",
                Symbol::One => "1",
                Symbol::X => "X",
                Symbol::Y => "Y",
                Symbol::Hash => "#",
                Symbol::Pipe => "|",
                Symbol::Semi => ";",
                Symbol::Comma => ",",
                Symbol::Caret => "^",
                Symbol::L => "L",
                Symbol::D => "D",
                Symbol::Dot => ".",
                Symbol::Star => "*",
                Symbol::Gt => ">",
                Symbol::Dollar => "$",
            }
        )
    }
}
const ALL_SYMBOLS: [Symbol; 16] = [
    Symbol::Blank,
    Symbol::Zero,
    Symbol::One,
    Symbol::X,
    Symbol::Y,
    Symbol::Hash,
    Symbol::Pipe,
    Symbol::Semi,
    Symbol::Comma,
    Symbol::Caret,
    Symbol::L,
    Symbol::D,
    Symbol::Dot,
    Symbol::Star,
    Symbol::Gt,
    Symbol::Dollar,
];

pub trait UtmEncodingScheme {
    type State;
    type Symbol;
    fn encode<Guest: TuringMachineSpec>(spec: &RunningTuringMachine<Guest>) -> Vec<Self::Symbol>;
    fn decode<'a, Guest: TuringMachineSpec>(
        guest: &'a Guest,
        tape: &[Self::Symbol],
    ) -> Result<RunningTuringMachine<'a, Guest>, String>;
}

pub struct MyUtmEncodingScheme;
impl UtmEncodingScheme for MyUtmEncodingScheme {
    type State = State;
    type Symbol = Symbol;

    // ════════════════════════════════════════════════════════════════════
    // Encoding: encode an arbitrary TM + input into a UTM tape
    // ════════════════════════════════════════════════════════════════════

    /// Encode a guest TM spec into UTM tape symbols.
    /// Layout: $ RULES # ACCEPTSTATES # STATE # BLANK # TAPE $
    ///
    /// RULES: dot-separated entries, each = stateBits | symBits | newStateBits | newSymBits | dir
    /// ACCEPTSTATES: semicolon-separated state encodings
    /// STATE: current state bits
    /// BLANK: blank symbol bits
    /// TAPE: comma-separated cells, head cell prefixed with ^
    fn encode<Guest: TuringMachineSpec>(guest: &RunningTuringMachine<Guest>) -> Vec<Self::Symbol> {
        Self::encode_with_rule_order(guest, None)
    }

    // ════════════════════════════════════════════════════════════════════
    // Decoding: extract guest state from the UTM tape
    // ════════════════════════════════════════════════════════════════════

    /// Decode the UTM tape back into guest TM.
    fn decode<'a, Guest: TuringMachineSpec>(
        guest: &'a Guest,
        tape: &[Self::Symbol],
    ) -> Result<RunningTuringMachine<'a, Guest>, String> {
        let guest_states: Vec<Guest::State> = guest.iter_states().collect();
        let guest_symbols: Vec<Guest::Symbol> = guest.iter_symbols().collect();

        let n_state_bits = num_bits(guest_states.len());
        let n_sym_bits = num_bits(guest_symbols.len());

        // Find the sections separated by #
        // Layout: $ #[0] RULES #[1] ACC #[2] STATE #[3] BLANK #[4] TAPE $
        let mut hashes: Vec<usize> = Vec::new();
        for (i, &s) in tape.iter().enumerate() {
            if s == Symbol::Hash {
                hashes.push(i);
            }
        }

        if hashes.len() < 5 {
            return Err(format!(
                "expected at least 5 # delimiters, found {}",
                hashes.len()
            ));
        }

        let state_start = hashes[2] + 1;
        let state = guest_states[from_binary_at(tape, state_start, n_state_bits)];

        let tape_start = hashes[4] + 1;
        let tape_end = tape.len();

        let tape_section = &tape[tape_start..tape_end];
        let mut cells: Vec<usize> = Vec::new();
        let mut head_pos: usize = 0;
        let mut i = 0;
        let mut cell_idx = 0;
        while i < tape_section.len() {
            let s = tape_section[i];
            if s == Symbol::Blank || s == Symbol::Dollar {
                break;
            }
            if s == Symbol::Comma {
                i += 1;
                cell_idx += 1;
                continue;
            }
            if s == Symbol::Caret || s == Symbol::Gt {
                if s == Symbol::Caret {
                    head_pos = cell_idx;
                }
                i += 1;
                continue;
            }
            if i + n_sym_bits > tape_section.len() {
                break;
            }
            let val = from_binary_at(tape_section, i, n_sym_bits);
            cells.push(val);
            i += n_sym_bits;
        }

        Ok(RunningTuringMachine {
            spec: guest,
            state,
            pos: head_pos,
            tape: cells.iter().map(|&i| guest_symbols[i]).collect(),
        })
    }
}

impl MyUtmEncodingScheme {
    /// Encode a guest TM, optionally reordering rules so that `last_rules`
    /// appear at the end of the rules section (in the given order).
    /// Rules not in `last_rules` appear first (in `iter_rules` order).
    ///
    /// The UTM scans rules right-to-left, so placing frequently-used rules
    /// last reduces search time.
    pub fn encode_with_rule_order<Guest: TuringMachineSpec>(
        guest: &RunningTuringMachine<Guest>,
        last_rules: Option<&[(Guest::State, Guest::Symbol)]>,
    ) -> Vec<Symbol> {
        let guest_states: Vec<Guest::State> = guest.spec.iter_states().collect();
        let guest_symbols: Vec<Guest::Symbol> = guest.spec.iter_symbols().collect();

        let guest_st_to_idx = guest_states
            .iter()
            .enumerate()
            .map(|(i, s)| (*s, i))
            .collect::<HashMap<Guest::State, usize>>();
        let guest_sym_to_idx = guest_symbols
            .iter()
            .enumerate()
            .map(|(i, s)| (*s, i))
            .collect::<HashMap<Guest::Symbol, usize>>();

        let n_state_bits = num_bits(guest_states.len());
        let n_sym_bits = num_bits(guest_symbols.len());

        // Collect all rules
        type Rule<S, Y> = (S, Y, S, Y, Dir);
        let all_rules: Vec<Rule<Guest::State, Guest::Symbol>> = guest.spec.iter_rules().collect();

        // Reorder rules if last_rules is provided
        let ordered_rules: Vec<&Rule<Guest::State, Guest::Symbol>> = match last_rules {
            None => all_rules.iter().collect(),
            Some(last) => {
                let last_set: std::collections::HashSet<(Guest::State, Guest::Symbol)> =
                    last.iter().copied().collect();
                let mut front: Vec<&Rule<Guest::State, Guest::Symbol>> = all_rules
                    .iter()
                    .filter(|(st, sym, _, _, _)| !last_set.contains(&(*st, *sym)))
                    .collect();
                // Append last_rules in the specified order
                for &(lst, lsym) in last {
                    if let Some(rule) = all_rules
                        .iter()
                        .find(|(st, sym, _, _, _)| *st == lst && *sym == lsym)
                    {
                        front.push(rule);
                    }
                }
                front
            }
        };

        let mut tape: Vec<Symbol> = Vec::new();
        tape.push(Symbol::Dollar);

        // RULES section: # .rule1 ; .rule2 ; .rule3 ... #
        tape.push(Symbol::Hash);
        let mut first_rule = true;
        for &(st, sym, nst, nsym, dir) in &ordered_rules {
            if !first_rule {
                tape.push(Symbol::Semi);
            }
            first_rule = false;
            tape.push(Symbol::Dot);
            tape.extend_from_slice(&to_binary(guest_st_to_idx[&st], n_state_bits));
            tape.push(Symbol::Pipe);
            tape.extend_from_slice(&to_binary(guest_sym_to_idx[&sym], n_sym_bits));
            tape.push(Symbol::Pipe);
            tape.extend_from_slice(&to_binary(guest_st_to_idx[&nst], n_state_bits));
            tape.push(Symbol::Pipe);
            tape.extend_from_slice(&to_binary(guest_sym_to_idx[&nsym], n_sym_bits));
            tape.push(Symbol::Pipe);
            tape.push(match dir {
                Dir::Left => Symbol::L,
                Dir::Right => Symbol::D,
            });
        }

        tape.push(Symbol::Hash);
        for (i, state) in guest
            .spec
            .iter_states()
            .filter(|s| guest.spec.is_accepting(*s))
            .enumerate()
        {
            if i > 0 {
                tape.push(Symbol::Semi);
            }
            tape.extend_from_slice(&to_binary(guest_st_to_idx[&state], n_state_bits));
        }

        tape.push(Symbol::Hash);
        tape.extend_from_slice(&to_binary(guest_st_to_idx[&guest.state], n_state_bits));

        tape.push(Symbol::Hash);
        tape.extend_from_slice(&to_binary(
            guest_sym_to_idx[&guest.spec.blank()],
            n_sym_bits,
        ));

        tape.push(Symbol::Hash);

        let caret_pos = tape.len();
        let default_tape = [guest.spec.blank()];
        tape.extend_from_slice(&encode_tape(
            &guest_sym_to_idx,
            if guest.tape.is_empty() {
                &default_tape
            } else {
                guest.tape.as_slice()
            },
        ));
        tape[caret_pos] = Symbol::Caret;

        tape
    }
}

// ── Helpers ──
pub fn num_bits(count: usize) -> usize {
    1.max((count.max(2) as f64).log2().ceil() as usize)
}

pub fn to_binary(index: usize, width: usize) -> Vec<Symbol> {
    let mut bits = Vec::with_capacity(width);
    for i in (0..width).rev() {
        bits.push(if (index >> i) & 1 == 1 {
            Symbol::One
        } else {
            Symbol::Zero
        });
    }
    bits
}

fn from_binary_at(tape: &[Symbol], start: usize, width: usize) -> usize {
    let mut val = 0;
    for i in 0..width {
        let b = tape[start + i];
        val = val * 2
            + if b == Symbol::One || b == Symbol::Y {
                1
            } else if b == Symbol::Zero || b == Symbol::X {
                0
            } else {
                panic!("invalid binary symbol at {}: {:?}", start + i, b)
            };
    }
    val
}

// ── RuleSet: transition table + ordered list for encoding ──
struct RuleSet(HashMap<(State, Symbol), (State, Symbol, Dir)>);
impl RuleSet {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn add(&mut self, state: State, sym: Symbol, new_state: State, new_sym: Symbol, dir: Dir) {
        let key = (state, sym);
        if let Some(existing) = self.0.get(&key) {
            panic!(
                "Duplicate rule: {:?} -> {:?} vs {:?}",
                key,
                existing,
                (new_state, new_sym, dir),
            );
        }
        self.0.insert(key, (new_state, new_sym, dir));
    }
}

fn scan_right(m: &mut RuleSet, state: State, syms: &[Symbol]) {
    for &s in syms {
        m.add(state, s, state, s, Dir::Right);
    }
}

fn scan_left(m: &mut RuleSet, state: State, syms: &[Symbol]) {
    for &s in syms {
        m.add(state, s, state, s, Dir::Left);
    }
}

fn seek_home(m: &mut RuleSet, from: State, to: State) {
    scan_left(
        m,
        from,
        &[
            Symbol::Zero,
            Symbol::One,
            Symbol::X,
            Symbol::Y,
            Symbol::Hash,
            Symbol::Pipe,
            Symbol::Semi,
            Symbol::Comma,
            Symbol::Caret,
            Symbol::Dot,
            Symbol::Star,
            Symbol::Gt,
            Symbol::L,
            Symbol::D,
        ],
    );
    m.add(from, Symbol::Dollar, to, Symbol::Dollar, Dir::Right);
}

fn seek_star(m: &mut RuleSet, from: State, to: State) {
    scan_left(
        m,
        from,
        &[
            Symbol::Zero,
            Symbol::One,
            Symbol::X,
            Symbol::Y,
            Symbol::Hash,
            Symbol::Pipe,
            Symbol::Semi,
            Symbol::Comma,
            Symbol::Caret,
            Symbol::Dot,
            Symbol::L,
            Symbol::D,
        ],
    );
    m.add(from, Symbol::Star, to, Symbol::Star, Dir::Right);
}

// ════════════════════════════════════════════════════════════════════
// UTM Rule Builder
// ════════════════════════════════════════════════════════════════════

fn build_utm_rules() -> RuleSet {
    use State::*;
    use Symbol::*;
    let mut r = RuleSet::new();

    // Symbol groups
    let rule_internals: &[Symbol] = &[Zero, One, X, Y, Pipe, L, D];
    let rule_all: &[Symbol] = &[Zero, One, X, Y, Pipe, L, D, Semi, Dot, Star];
    let bits: &[Symbol] = &[Zero, One];
    let marked_bits: &[Symbol] = &[X, Y];
    let bits_and_marked: &[Symbol] = &[Zero, One, X, Y];

    // ══════════════════════════════════════════════════════════════
    // PHASE 0: INIT
    // ══════════════════════════════════════════════════════════════
    r.add(Init, Dollar, InitSkip, Dollar, Dir::Right);
    r.add(Init, Hash, InitSeekEnd, Hash, Dir::Right);
    r.add(InitSkip, Hash, InitSeekEnd, Hash, Dir::Right);
    {
        let s = InitSeekEnd;
        let mut syms: Vec<Symbol> = rule_internals.to_vec();
        syms.extend_from_slice(&[Semi, Dot]);
        scan_right(&mut r, s, &syms);
        r.add(s, Hash, MarkRule, Hash, Dir::Left);
    }

    // ══════════════════════════════════════════════════════════════
    // PHASE 1: MARK RULE (right-to-left search)
    // ══════════════════════════════════════════════════════════════
    {
        let mr = MarkRule;
        scan_left(&mut r, mr, rule_internals);
        r.add(mr, Semi, mr, Semi, Dir::Left);
        r.add(mr, Dot, CmpStRead, Star, Dir::Right);
        r.add(mr, Hash, MarkRuleNoMatch, Hash, Dir::Right);
    }
    {
        let nm = MarkRuleNoMatch;
        let mut syms: Vec<Symbol> = rule_internals.to_vec();
        syms.extend_from_slice(&[Semi, Dot]);
        scan_right(&mut r, nm, &syms);
        r.add(nm, Hash, ChkAccInit, Hash, Dir::Right);
    }

    // ══════════════════════════════════════════════════════════════
    // PHASE 2: COMPARE STATE BITS
    // ══════════════════════════════════════════════════════════════
    r.add(CmpStRead, Zero, CmpStC0, X, Dir::Right);
    r.add(CmpStRead, One, CmpStC1, Y, Dir::Right);
    r.add(CmpStRead, Pipe, StMatchCleanup, Pipe, Dir::Right);

    for (c_sym, carry, sk1, find) in [
        (Zero, CmpStC0, CmpStC0Sk1, CmpStC0Find),
        (One, CmpStC1, CmpStC1Sk1, CmpStC1Find),
    ] {
        scan_right(&mut r, carry, rule_all);
        r.add(carry, Hash, sk1, Hash, Dir::Right);

        let mut sk1_syms: Vec<Symbol> = bits_and_marked.to_vec();
        sk1_syms.push(Semi);
        scan_right(&mut r, sk1, &sk1_syms);
        r.add(sk1, Hash, find, Hash, Dir::Right);

        scan_right(&mut r, find, marked_bits);
        if c_sym == Zero {
            r.add(find, Zero, CmpStOk, X, Dir::Left);
            r.add(find, One, CmpStFail, Y, Dir::Left);
        } else {
            r.add(find, One, CmpStOk, Y, Dir::Left);
            r.add(find, Zero, CmpStFail, X, Dir::Left);
        }
    }

    // Bit matched -> return to * to read next bit
    {
        seek_star(&mut r, CmpStOk, CmpStNextbit);
        let nb = CmpStNextbit;
        scan_right(&mut r, nb, marked_bits);
        r.add(nb, Zero, CmpStC0, X, Dir::Right);
        r.add(nb, One, CmpStC1, Y, Dir::Right);
        r.add(nb, Pipe, StMatchCleanup, Pipe, Dir::Right);
    }

    // ══════════════════════════════════════════════════════════════
    // STATE MATCH CLEANUP
    // ══════════════════════════════════════════════════════════════
    {
        let smc = StMatchCleanup;
        r.add(smc, Zero, StmGoLeft, Zero, Dir::Left);
        r.add(smc, One, StmGoLeft, One, Dir::Left);
        r.add(smc, Pipe, StmGoLeft, Pipe, Dir::Left);
    }
    {
        let gl = StmGoLeft;
        r.add(gl, Pipe, StmRestoreRule, Pipe, Dir::Left);
        scan_left(&mut r, gl, bits);
    }
    {
        let rr = StmRestoreRule;
        r.add(rr, X, rr, Zero, Dir::Left);
        r.add(rr, Y, rr, One, Dir::Left);
        scan_left(&mut r, rr, bits);
        r.add(rr, Star, StmGotoState, Star, Dir::Right);
    }
    {
        let gs = StmGotoState;
        let mut syms: Vec<Symbol> = rule_internals.to_vec();
        syms.extend_from_slice(&[Semi, Dot]);
        scan_right(&mut r, gs, &syms);
        r.add(gs, Hash, StmGsSk1, Hash, Dir::Right);
    }
    {
        let sk = StmGsSk1;
        let mut syms: Vec<Symbol> = bits_and_marked.to_vec();
        syms.push(Semi);
        scan_right(&mut r, sk, &syms);
        r.add(sk, Hash, StmRestoreState, Hash, Dir::Right);
    }
    {
        let rs = StmRestoreState;
        r.add(rs, X, rs, Zero, Dir::Right);
        r.add(rs, Y, rs, One, Dir::Right);
        scan_right(&mut r, rs, bits);
        r.add(rs, Hash, StmBackToRule, Hash, Dir::Left);
    }
    {
        seek_star(&mut r, StmBackToRule, SymSkipState);
    }
    {
        let ss = SymSkipState;
        scan_right(&mut r, ss, bits);
        r.add(ss, Pipe, CmpSymRead, Pipe, Dir::Right);
    }

    // ══════════════════════════════════════════════════════════════
    // STATE MISMATCH
    // ══════════════════════════════════════════════════════════════
    {
        let sf = CmpStFail;
        scan_left(&mut r, sf, bits_and_marked);
        r.add(sf, Hash, StfRestoreState, Hash, Dir::Right);
    }
    {
        let rs = StfRestoreState;
        r.add(rs, X, rs, Zero, Dir::Right);
        r.add(rs, Y, rs, One, Dir::Right);
        scan_right(&mut r, rs, bits);
        r.add(rs, Hash, StfFindStar, Hash, Dir::Left);
    }
    {
        let mut syms: Vec<Symbol> = bits_and_marked.to_vec();
        syms.extend_from_slice(&[Semi, Hash, Pipe, Dot, L, D]);
        scan_left(&mut r, StfFindStar, &syms);
        r.add(StfFindStar, Star, StfRestoreRule, Dot, Dir::Right);
    }
    {
        let rr = StfRestoreRule;
        r.add(rr, X, rr, Zero, Dir::Right);
        r.add(rr, Y, rr, One, Dir::Right);
        scan_right(&mut r, rr, bits);
        r.add(rr, Pipe, StfGoPrev, Pipe, Dir::Left);
    }
    {
        let gp = StfGoPrev;
        scan_left(&mut r, gp, bits);
        r.add(gp, Dot, MarkRule, Dot, Dir::Left);
    }

    // ══════════════════════════════════════════════════════════════
    // PHASE 3: COMPARE SYMBOL BITS
    // ══════════════════════════════════════════════════════════════
    r.add(CmpSymRead, Zero, CmpSymC0, X, Dir::Right);
    r.add(CmpSymRead, One, CmpSymC1, Y, Dir::Right);
    r.add(CmpSymRead, Pipe, SymMatchCleanup, Pipe, Dir::Right);

    for c in [0u8, 1u8] {
        let (carry, s1, s2, s3, fh, fb) = if c == 0 {
            (
                CmpSymC0, CmpSymC0S1, CmpSymC0S2, CmpSymC0S3, CmpSymC0Fh, CmpSymC0Fb,
            )
        } else {
            (
                CmpSymC1, CmpSymC1S1, CmpSymC1S2, CmpSymC1S3, CmpSymC1Fh, CmpSymC1Fb,
            )
        };

        scan_right(&mut r, carry, rule_all);
        r.add(carry, Hash, s1, Hash, Dir::Right);

        // Skip ACC
        {
            let mut syms: Vec<Symbol> = bits_and_marked.to_vec();
            syms.push(Semi);
            scan_right(&mut r, s1, &syms);
            r.add(s1, Hash, s2, Hash, Dir::Right);
        }
        // Skip STATE
        scan_right(&mut r, s2, bits_and_marked);
        r.add(s2, Hash, s3, Hash, Dir::Right);
        // Skip BLANK
        scan_right(&mut r, s3, bits);
        r.add(s3, Hash, fh, Hash, Dir::Right);
        // Find ^
        {
            let mut syms: Vec<Symbol> = bits_and_marked.to_vec();
            syms.push(Comma);
            scan_right(&mut r, fh, &syms);
            r.add(fh, Caret, fb, Caret, Dir::Right);
        }
        // Find next unmarked bit in head cell
        scan_right(&mut r, fb, marked_bits);
        if c == 0 {
            r.add(fb, Zero, CmpSymOk, X, Dir::Left);
            r.add(fb, One, CmpSymFail, Y, Dir::Left);
        } else {
            r.add(fb, One, CmpSymOk, Y, Dir::Left);
            r.add(fb, Zero, CmpSymFail, X, Dir::Left);
        }
    }

    // Symbol bit matched -> return to * to read next bit
    {
        seek_star(&mut r, CmpSymOk, CmpSymNextbit);
        let nb = CmpSymNextbit;
        scan_right(&mut r, nb, bits);
        r.add(nb, Pipe, CmpSymNb2, Pipe, Dir::Right);
    }
    {
        let nb2 = CmpSymNb2;
        scan_right(&mut r, nb2, marked_bits);
        r.add(nb2, Zero, CmpSymC0, X, Dir::Right);
        r.add(nb2, One, CmpSymC1, Y, Dir::Right);
        r.add(nb2, Pipe, SymMatchCleanup, Pipe, Dir::Right);
    }

    // ══════════════════════════════════════════════════════════════
    // SYMBOL MISMATCH
    // ══════════════════════════════════════════════════════════════
    {
        let sf = CmpSymFail;
        scan_left(&mut r, sf, bits_and_marked);
        r.add(sf, Caret, SymfRestHead, Caret, Dir::Right);
    }
    {
        let rh = SymfRestHead;
        r.add(rh, X, rh, Zero, Dir::Right);
        r.add(rh, Y, rh, One, Dir::Right);
        scan_right(&mut r, rh, bits);
        r.add(rh, Comma, SymfSeekStar, Comma, Dir::Left);
        r.add(rh, Blank, SymfSeekStar, Blank, Dir::Left);
    }
    {
        seek_star(&mut r, SymfSeekStar, SymfSkipSt);
    }
    {
        let ss = SymfSkipSt;
        scan_right(&mut r, ss, bits);
        r.add(ss, Pipe, SymfRestSym, Pipe, Dir::Right);
    }
    {
        let rs = SymfRestSym;
        r.add(rs, X, rs, Zero, Dir::Right);
        r.add(rs, Y, rs, One, Dir::Right);
        scan_right(&mut r, rs, bits);
        r.add(rs, Pipe, SymfDeactivate, Pipe, Dir::Left);
    }
    {
        let da = SymfDeactivate;
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.push(Pipe);
        scan_left(&mut r, da, &syms);
        r.add(da, Star, MarkRule, Dot, Dir::Left);
    }

    // ══════════════════════════════════════════════════════════════
    // SYMBOL MATCH CLEANUP
    // ══════════════════════════════════════════════════════════════
    {
        let sc = SymMatchCleanup;
        let mut syms: Vec<Symbol> = rule_internals.to_vec();
        syms.extend_from_slice(&[Semi, Dot]);
        scan_right(&mut r, sc, &syms);
        r.add(sc, Hash, SmcS1, Hash, Dir::Right);
    }
    {
        let s1 = SmcS1;
        let mut syms: Vec<Symbol> = bits_and_marked.to_vec();
        syms.push(Semi);
        scan_right(&mut r, s1, &syms);
        r.add(s1, Hash, SmcS2, Hash, Dir::Right);
    }
    {
        let s2 = SmcS2;
        scan_right(&mut r, s2, bits_and_marked);
        r.add(s2, Hash, SmcS3, Hash, Dir::Right);
    }
    {
        let s3 = SmcS3;
        scan_right(&mut r, s3, bits);
        r.add(s3, Hash, SmcFh, Hash, Dir::Right);
    }
    {
        let fh = SmcFh;
        let mut syms: Vec<Symbol> = bits_and_marked.to_vec();
        syms.push(Comma);
        scan_right(&mut r, fh, &syms);
        r.add(fh, Caret, SmcRestHead, Caret, Dir::Right);
    }
    {
        let rh = SmcRestHead;
        r.add(rh, X, rh, Zero, Dir::Right);
        r.add(rh, Y, rh, One, Dir::Right);
        scan_right(&mut r, rh, bits);
        r.add(rh, Comma, SmcRestDone, Comma, Dir::Left);
        r.add(rh, Blank, SmcRestDone, Blank, Dir::Left);
    }
    {
        seek_star(&mut r, SmcRestDone, SmcSkipSt);
    }
    {
        let ss = SmcSkipSt;
        scan_right(&mut r, ss, bits);
        r.add(ss, Pipe, SmcRestSym, Pipe, Dir::Right);
    }
    {
        let rs = SmcRestSym;
        r.add(rs, X, rs, Zero, Dir::Right);
        r.add(rs, Y, rs, One, Dir::Right);
        scan_right(&mut r, rs, bits);
        r.add(rs, Pipe, ApplyReadNst, Pipe, Dir::Right);
    }

    // ══════════════════════════════════════════════════════════════
    // PHASE 4: APPLY RULE - COPY NEW STATE
    // ══════════════════════════════════════════════════════════════
    r.add(ApplyReadNst, Zero, CpNstC0, X, Dir::Right);
    r.add(ApplyReadNst, One, CpNstC1, Y, Dir::Right);
    r.add(ApplyReadNst, Pipe, CpNstDone, Pipe, Dir::Left);

    for c in [0u8, 1u8] {
        let (carry, s1, w, mark) = if c == 0 {
            (CpNstC0, CpNstC0S1, CpNstC0W, X)
        } else {
            (CpNstC1, CpNstC1S1, CpNstC1W, Y)
        };

        scan_right(&mut r, carry, rule_all);
        r.add(carry, Hash, s1, Hash, Dir::Right);

        {
            let mut syms: Vec<Symbol> = bits_and_marked.to_vec();
            syms.push(Semi);
            scan_right(&mut r, s1, &syms);
            r.add(s1, Hash, w, Hash, Dir::Right);
        }

        scan_right(&mut r, w, marked_bits);
        r.add(w, Zero, CpNstRet, mark, Dir::Left);
        r.add(w, One, CpNstRet, mark, Dir::Left);
    }
    {
        seek_star(&mut r, CpNstRet, CpNstNext);
    }
    {
        let n = CpNstNext;
        scan_right(&mut r, n, bits);
        r.add(n, Pipe, CpNstNext2, Pipe, Dir::Right);
    }
    {
        let n2 = CpNstNext2;
        scan_right(&mut r, n2, bits);
        r.add(n2, Pipe, CpNstNext3, Pipe, Dir::Right);
    }
    {
        let n3 = CpNstNext3;
        scan_right(&mut r, n3, marked_bits);
        r.add(n3, Zero, CpNstC0, X, Dir::Right);
        r.add(n3, One, CpNstC1, Y, Dir::Right);
        r.add(n3, Pipe, CpNstDone, Pipe, Dir::Left);
    }

    // cp_nst_done: restore marks
    {
        let d = CpNstDone;
        r.add(d, X, d, Zero, Dir::Left);
        r.add(d, Y, d, One, Dir::Left);
        r.add(d, Pipe, CpNstRestNav, Pipe, Dir::Right);
    }
    {
        let nav = CpNstRestNav;
        let mut syms: Vec<Symbol> = rule_internals.to_vec();
        syms.extend_from_slice(&[Semi, Dot]);
        scan_right(&mut r, nav, &syms);
        r.add(nav, Hash, CpNstRestS1, Hash, Dir::Right);
    }
    {
        let s1 = CpNstRestS1;
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.push(Semi);
        scan_right(&mut r, s1, &syms);
        r.add(s1, Hash, CpNstRestDo, Hash, Dir::Right);
    }
    {
        let rd = CpNstRestDo;
        r.add(rd, X, rd, Zero, Dir::Right);
        r.add(rd, Y, rd, One, Dir::Right);
        scan_right(&mut r, rd, bits);
        r.add(rd, Hash, CpNsymSeek, Hash, Dir::Left);
    }
    {
        seek_star(&mut r, CpNsymSeek, CpNsymNav);
    }

    // ══════════════════════════════════════════════════════════════
    // PHASE 5: COPY NEW SYMBOL
    // ══════════════════════════════════════════════════════════════
    {
        let n = CpNsymNav;
        scan_right(&mut r, n, bits);
        r.add(n, Pipe, CpNsymNav2, Pipe, Dir::Right);
    }
    {
        let n2 = CpNsymNav2;
        scan_right(&mut r, n2, bits);
        r.add(n2, Pipe, CpNsymNav3, Pipe, Dir::Right);
    }
    {
        let n3 = CpNsymNav3;
        scan_right(&mut r, n3, bits);
        r.add(n3, Pipe, CpNsymRead, Pipe, Dir::Right);
    }

    r.add(CpNsymRead, Zero, CpNsymC0, X, Dir::Right);
    r.add(CpNsymRead, One, CpNsymC1, Y, Dir::Right);
    r.add(CpNsymRead, Pipe, CpNsymDone, Pipe, Dir::Left);

    // Carry to head cell: skip rules, ACC, STATE, BLANK, find ^
    for c in [0u8, 1u8] {
        let (carry, s1, s2, s3, fh, fb, mark) = if c == 0 {
            (
                CpNsymC0, CpNsymC0S1, CpNsymC0S2, CpNsymC0S3, CpNsymC0Fh, CpNsymC0Fb, X,
            )
        } else {
            (
                CpNsymC1, CpNsymC1S1, CpNsymC1S2, CpNsymC1S3, CpNsymC1Fh, CpNsymC1Fb, Y,
            )
        };

        scan_right(&mut r, carry, rule_all);
        r.add(carry, Hash, s1, Hash, Dir::Right);

        {
            let mut syms: Vec<Symbol> = bits_and_marked.to_vec();
            syms.push(Semi);
            scan_right(&mut r, s1, &syms);
            r.add(s1, Hash, s2, Hash, Dir::Right);
        }
        scan_right(&mut r, s2, bits_and_marked);
        r.add(s2, Hash, s3, Hash, Dir::Right);

        scan_right(&mut r, s3, bits);
        r.add(s3, Hash, fh, Hash, Dir::Right);

        {
            let mut syms: Vec<Symbol> = bits_and_marked.to_vec();
            syms.push(Comma);
            scan_right(&mut r, fh, &syms);
            r.add(fh, Caret, fb, Caret, Dir::Right);
        }

        scan_right(&mut r, fb, marked_bits);
        r.add(fb, Zero, CpNsymRet, mark, Dir::Left);
        r.add(fb, One, CpNsymRet, mark, Dir::Left);
    }
    {
        seek_star(&mut r, CpNsymRet, CpNsymFnext);
    }
    {
        let fn_ = CpNsymFnext;
        scan_right(&mut r, fn_, bits);
        r.add(fn_, Pipe, CpNsymFn2, Pipe, Dir::Right);
    }
    {
        let fn2 = CpNsymFn2;
        scan_right(&mut r, fn2, bits);
        r.add(fn2, Pipe, CpNsymFn3, Pipe, Dir::Right);
    }
    {
        let fn3 = CpNsymFn3;
        scan_right(&mut r, fn3, bits);
        r.add(fn3, Pipe, CpNsymFn4, Pipe, Dir::Right);
    }
    {
        let fn4 = CpNsymFn4;
        scan_right(&mut r, fn4, marked_bits);
        r.add(fn4, Zero, CpNsymC0, X, Dir::Right);
        r.add(fn4, One, CpNsymC1, Y, Dir::Right);
        r.add(fn4, Pipe, CpNsymDone, Pipe, Dir::Left);
    }

    // cp_nsym_done: restore newsym field and head cell
    {
        let d = CpNsymDone;
        r.add(d, X, d, Zero, Dir::Left);
        r.add(d, Y, d, One, Dir::Left);
        r.add(d, Pipe, CpNsymRestNav, Pipe, Dir::Right);
    }
    {
        let nav = CpNsymRestNav;
        let mut syms: Vec<Symbol> = rule_internals.to_vec();
        syms.extend_from_slice(&[Semi, Dot]);
        scan_right(&mut r, nav, &syms);
        r.add(nav, Hash, CpNsymRnS1, Hash, Dir::Right);
    }
    {
        let s1 = CpNsymRnS1;
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.push(Semi);
        scan_right(&mut r, s1, &syms);
        r.add(s1, Hash, CpNsymRnS2, Hash, Dir::Right);
    }
    {
        let s2 = CpNsymRnS2;
        scan_right(&mut r, s2, bits);
        r.add(s2, Hash, CpNsymRnS3, Hash, Dir::Right);
    }
    {
        let s3 = CpNsymRnS3;
        scan_right(&mut r, s3, bits);
        r.add(s3, Hash, CpNsymRnFh, Hash, Dir::Right);
    }
    {
        let fh = CpNsymRnFh;
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.push(Comma);
        scan_right(&mut r, fh, &syms);
        r.add(fh, Caret, CpNsymRnDo, Caret, Dir::Right);
    }
    {
        let rd = CpNsymRnDo;
        r.add(rd, X, rd, Zero, Dir::Right);
        r.add(rd, Y, rd, One, Dir::Right);
        scan_right(&mut r, rd, bits);
        r.add(rd, Comma, ReadDir, Comma, Dir::Left);
        r.add(rd, Blank, ReadDir, Blank, Dir::Left);
    }

    // ══════════════════════════════════════════════════════════════
    // PHASE 6: READ DIRECTION AND MOVE HEAD
    // ══════════════════════════════════════════════════════════════
    {
        seek_star(&mut r, ReadDir, RdSkipToDir);
    }
    {
        let sk = RdSkipToDir;
        scan_right(&mut r, sk, bits);
        r.add(sk, Pipe, RdSk2, Pipe, Dir::Right);
    }
    {
        let sk2 = RdSk2;
        scan_right(&mut r, sk2, bits);
        r.add(sk2, Pipe, RdSk3, Pipe, Dir::Right);
    }
    {
        let sk3 = RdSk3;
        scan_right(&mut r, sk3, bits);
        r.add(sk3, Pipe, RdSk4, Pipe, Dir::Right);
    }
    {
        let sk4 = RdSk4;
        scan_right(&mut r, sk4, bits);
        r.add(sk4, Pipe, RdRead, Pipe, Dir::Right);
    }
    {
        r.add(RdRead, L, MoveLeft, L, Dir::Left);
        r.add(RdRead, D, MoveRight, D, Dir::Left);
    }

    // ══════════════════════════════════════════════════════════════
    // MOVE RIGHT
    // ══════════════════════════════════════════════════════════════
    {
        let mr = MoveRight;
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.extend_from_slice(&[Pipe, L, D]);
        scan_left(&mut r, mr, &syms);
        r.add(mr, Star, MrNav, Dot, Dir::Right);
    }
    {
        let nav = MrNav;
        let mut syms: Vec<Symbol> = rule_internals.to_vec();
        syms.extend_from_slice(&[Semi, Dot]);
        scan_right(&mut r, nav, &syms);
        r.add(nav, Hash, MrS1, Hash, Dir::Right);
    }
    {
        let s1 = MrS1;
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.push(Semi);
        scan_right(&mut r, s1, &syms);
        r.add(s1, Hash, MrS2, Hash, Dir::Right);
    }
    {
        let s2 = MrS2;
        scan_right(&mut r, s2, bits);
        r.add(s2, Hash, MrS3, Hash, Dir::Right);
    }
    {
        let s3 = MrS3;
        scan_right(&mut r, s3, bits);
        r.add(s3, Hash, MrFindHead, Hash, Dir::Right);
    }
    {
        let fh = MrFindHead;
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.push(Comma);
        scan_right(&mut r, fh, &syms);
        r.add(fh, Caret, MrSkipCell, Gt, Dir::Right);
    }
    {
        let sc = MrSkipCell;
        scan_right(&mut r, sc, bits);
        r.add(sc, Comma, MrPlaceHead, Caret, Dir::Left);
        r.add(sc, Blank, MrExtendInit, Blank, Dir::Left);
    }
    {
        let ph = MrPlaceHead;
        scan_left(&mut r, ph, bits);
        r.add(ph, Gt, DoneSeekHome, Comma, Dir::Left);
    }

    // EXTEND TAPE (move right past end)
    {
        let ei = MrExtendInit;
        scan_left(&mut r, ei, bits);
        r.add(ei, Gt, MrExtToBlank, Comma, Dir::Right);
    }
    {
        let tb = MrExtToBlank;
        scan_right(&mut r, tb, bits);
        r.add(tb, Blank, MrExtWriteHead, Caret, Dir::Left);
    }
    {
        seek_home(&mut r, MrExtWriteHead, MrExtHome);
    }
    {
        let eh = MrExtHome;
        r.add(eh, Hash, MrExtH1, Hash, Dir::Right);
    }
    {
        let h1 = MrExtH1;
        let mut syms: Vec<Symbol> = rule_internals.to_vec();
        syms.extend_from_slice(&[Semi, Dot]);
        scan_right(&mut r, h1, &syms);
        r.add(h1, Hash, MrExtH2, Hash, Dir::Right);
    }
    {
        let h2 = MrExtH2;
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.push(Semi);
        scan_right(&mut r, h2, &syms);
        r.add(h2, Hash, MrExtH3, Hash, Dir::Right);
    }
    {
        let h3 = MrExtH3;
        scan_right(&mut r, h3, bits);
        r.add(h3, Hash, MrExtReadBlank, Hash, Dir::Right);
    }
    {
        let rb = MrExtReadBlank;
        r.add(rb, Zero, MrExtBc0, X, Dir::Right);
        r.add(rb, One, MrExtBc1, Y, Dir::Right);
        r.add(rb, Hash, MrExtRestBlank, Hash, Dir::Left);
    }
    for c in [0u8, 1u8] {
        let (carry, c_sym) = if c == 0 {
            (MrExtBc0, Zero)
        } else {
            (MrExtBc1, One)
        };
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.extend_from_slice(&[Hash, Comma, Caret]);
        scan_right(&mut r, carry, &syms);
        r.add(carry, Blank, MrExtBcRet, c_sym, Dir::Left);
    }
    {
        let ret = MrExtBcRet;
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.extend_from_slice(&[Hash, Comma, Caret]);
        scan_left(&mut r, ret, &syms);
        r.add(ret, X, MrExtBcNext, X, Dir::Right);
        r.add(ret, Y, MrExtBcNext, Y, Dir::Right);
    }
    {
        let next = MrExtBcNext;
        scan_right(&mut r, next, marked_bits);
        r.add(next, Zero, MrExtBc0, X, Dir::Right);
        r.add(next, One, MrExtBc1, Y, Dir::Right);
        r.add(next, Hash, MrExtRestBlank, Hash, Dir::Left);
    }
    {
        let rb = MrExtRestBlank;
        r.add(rb, X, rb, Zero, Dir::Left);
        r.add(rb, Y, rb, One, Dir::Left);
        scan_left(&mut r, rb, bits);
        r.add(rb, Hash, DoneSeekHome, Hash, Dir::Left);
    }

    // ══════════════════════════════════════════════════════════════
    // MOVE LEFT
    // ══════════════════════════════════════════════════════════════
    {
        let ml = MoveLeft;
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.extend_from_slice(&[Pipe, L, D]);
        scan_left(&mut r, ml, &syms);
        r.add(ml, Star, MlNav, Dot, Dir::Right);
    }
    {
        let nav = MlNav;
        let mut syms: Vec<Symbol> = rule_internals.to_vec();
        syms.extend_from_slice(&[Semi, Dot]);
        scan_right(&mut r, nav, &syms);
        r.add(nav, Hash, MlS1, Hash, Dir::Right);
    }
    {
        let s1 = MlS1;
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.push(Semi);
        scan_right(&mut r, s1, &syms);
        r.add(s1, Hash, MlS2, Hash, Dir::Right);
    }
    {
        let s2 = MlS2;
        scan_right(&mut r, s2, bits);
        r.add(s2, Hash, MlS3, Hash, Dir::Right);
    }
    {
        let s3 = MlS3;
        scan_right(&mut r, s3, bits);
        r.add(s3, Hash, MlFindHead, Hash, Dir::Right);
    }
    {
        let fh = MlFindHead;
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.push(Comma);
        scan_right(&mut r, fh, &syms);
        r.add(fh, Caret, MlMark, Gt, Dir::Left);
    }
    {
        let mk = MlMark;
        scan_left(&mut r, mk, bits);
        r.add(mk, Comma, MlRestore, Caret, Dir::Right);
    }
    {
        let rs = MlRestore;
        scan_right(&mut r, rs, bits);
        r.add(rs, Gt, DoneSeekHome, Comma, Dir::Left);
    }

    // ══════════════════════════════════════════════════════════════
    // PHASE 7: SEEK HOME AND RESTART
    // ══════════════════════════════════════════════════════════════
    seek_home(&mut r, DoneSeekHome, Init);

    // ══════════════════════════════════════════════════════════════
    // PHASE 8: CHECK ACCEPT STATES
    // ══════════════════════════════════════════════════════════════
    {
        let ci = ChkAccInit;
        r.add(ci, Hash, RejFinalHome, Hash, Dir::Left);
        r.add(ci, Zero, ChkAccC0, X, Dir::Right);
        r.add(ci, One, ChkAccC1, Y, Dir::Right);
    }

    for (carry, find) in [(ChkAccC0, ChkAccC0Find), (ChkAccC1, ChkAccC1Find)] {
        let mut syms: Vec<Symbol> = bits_and_marked.to_vec();
        syms.push(Semi);
        scan_right(&mut r, carry, &syms);
        r.add(carry, Hash, find, Hash, Dir::Right);

        scan_right(&mut r, find, marked_bits);
        if carry == ChkAccC0 {
            r.add(find, Zero, ChkAccOk, X, Dir::Left);
            r.add(find, One, ChkAccFailBit, Y, Dir::Left);
        } else {
            r.add(find, One, ChkAccOk, Y, Dir::Left);
            r.add(find, Zero, ChkAccFailBit, X, Dir::Left);
        }
    }

    // Bit matched -> go back for next bit
    {
        let ok = ChkAccOk;
        scan_left(&mut r, ok, bits_and_marked);
        r.add(ok, Hash, ChkAccOkAcc, Hash, Dir::Left);
    }
    {
        let oa = ChkAccOkAcc;
        let mut syms: Vec<Symbol> = bits_and_marked.to_vec();
        syms.push(Semi);
        scan_left(&mut r, oa, &syms);
        r.add(oa, Hash, ChkAccOkFind, Hash, Dir::Right);
    }
    {
        let of_ = ChkAccOkFind;
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.push(Semi);
        scan_right(&mut r, of_, &syms);
        r.add(of_, X, ChkAccOkSkip, X, Dir::Right);
        r.add(of_, Y, ChkAccOkSkip, Y, Dir::Right);
        r.add(of_, Hash, AcceptSeekHome, Hash, Dir::Left);
    }
    {
        let skip = ChkAccOkSkip;
        scan_right(&mut r, skip, marked_bits);
        r.add(skip, Zero, ChkAccC0, X, Dir::Right);
        r.add(skip, One, ChkAccC1, Y, Dir::Right);
        r.add(skip, Semi, AcceptSeekHome, Semi, Dir::Left);
        r.add(skip, Hash, AcceptSeekHome, Hash, Dir::Left);
    }

    // Bit mismatch -> restore STATE marks, restore acc entry marks, try next entry
    {
        let fb = ChkAccFailBit;
        scan_left(&mut r, fb, bits_and_marked);
        r.add(fb, Hash, ChkAccRestState, Hash, Dir::Right);
    }
    {
        let rs = ChkAccRestState;
        r.add(rs, X, rs, Zero, Dir::Right);
        r.add(rs, Y, rs, One, Dir::Right);
        scan_right(&mut r, rs, bits);
        r.add(rs, Hash, ChkAccBack2acc, Hash, Dir::Left);
    }
    {
        let ba = ChkAccBack2acc;
        scan_left(&mut r, ba, bits);
        r.add(ba, Hash, ChkAccIntoAcc, Hash, Dir::Left);
    }
    {
        let ia = ChkAccIntoAcc;
        let mut syms: Vec<Symbol> = bits_and_marked.to_vec();
        syms.push(Semi);
        scan_left(&mut r, ia, &syms);
        r.add(ia, Hash, ChkAccDoRest, Hash, Dir::Right);
    }
    {
        let dr = ChkAccDoRest;
        scan_right(&mut r, dr, bits);
        r.add(dr, X, ChkAccDoRest2, Zero, Dir::Right);
        r.add(dr, Y, ChkAccDoRest2, One, Dir::Right);
        r.add(dr, Semi, ChkAccNextEntry, Semi, Dir::Right);
        r.add(dr, Hash, RejectSeekHome, Hash, Dir::Left);
    }
    {
        let dr2 = ChkAccDoRest2;
        r.add(dr2, X, dr2, Zero, Dir::Right);
        r.add(dr2, Y, dr2, One, Dir::Right);
        scan_right(&mut r, dr2, bits);
        r.add(dr2, Semi, ChkAccNextEntry, Semi, Dir::Right);
        r.add(dr2, Hash, RejectSeekHome, Hash, Dir::Left);
    }
    {
        let ne = ChkAccNextEntry;
        r.add(ne, Zero, ChkAccC0, X, Dir::Right);
        r.add(ne, One, ChkAccC1, Y, Dir::Right);
        r.add(ne, Hash, RejectSeekHome, Hash, Dir::Left);
    }

    // Accept: restore ACCEPTSTATES and STATE, seek home
    {
        let ash = AcceptSeekHome;
        let mut syms: Vec<Symbol> = bits_and_marked.to_vec();
        syms.push(Semi);
        scan_left(&mut r, ash, &syms);
        r.add(ash, Hash, AccRestAcc, Hash, Dir::Right);
    }
    {
        let ra = AccRestAcc;
        r.add(ra, X, ra, Zero, Dir::Right);
        r.add(ra, Y, ra, One, Dir::Right);
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.push(Semi);
        scan_right(&mut r, ra, &syms);
        r.add(ra, Hash, AccRestState, Hash, Dir::Right);
    }
    {
        let rs = AccRestState;
        r.add(rs, X, rs, Zero, Dir::Right);
        r.add(rs, Y, rs, One, Dir::Right);
        scan_right(&mut r, rs, bits);
        r.add(rs, Hash, AccFinalHome, Hash, Dir::Left);
    }
    seek_home(&mut r, AccFinalHome, Accept);

    // Reject: restore marks
    {
        let rsh = RejectSeekHome;
        let mut syms: Vec<Symbol> = bits_and_marked.to_vec();
        syms.push(Semi);
        scan_left(&mut r, rsh, &syms);
        r.add(rsh, Hash, RejRestAcc, Hash, Dir::Right);
    }
    {
        let ra = RejRestAcc;
        r.add(ra, X, ra, Zero, Dir::Right);
        r.add(ra, Y, ra, One, Dir::Right);
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.push(Semi);
        scan_right(&mut r, ra, &syms);
        r.add(ra, Hash, RejRestState, Hash, Dir::Right);
    }
    {
        let rs = RejRestState;
        r.add(rs, X, rs, Zero, Dir::Right);
        r.add(rs, Y, rs, One, Dir::Right);
        scan_right(&mut r, rs, bits);
        r.add(rs, Hash, RejFinalHome, Hash, Dir::Left);
    }
    seek_home(&mut r, RejFinalHome, Reject);

    r
}

// ════════════════════════════════════════════════════════════════════
// build_utm_spec: Assemble the full TuringMachineSpec for the UTM
// ════════════════════════════════════════════════════════════════════

pub static UTM_SPEC: LazyLock<SimpleTuringMachineSpec<State, Symbol>> = LazyLock::new(|| {
    SimpleTuringMachineSpec {
        initial: State::Init,
        accepting: HashSet::from([State::Accept]),
        blank: Symbol::Blank,
        transitions: build_utm_rules().0,
        all_states: ALL_STATES.to_vec(),
        all_symbols: ALL_SYMBOLS.to_vec(),
    }

    // SmallTuringMachineSpec {
    //     n_states: N_UTM_STATES,
    //     n_symbols: N_SYMBOLS,
    //     initial: SmallTmState(State::Init.0),
    //     accept: SmallTmState(State::Accept.0),
    //     blank: SmallTmSymbol(Symbol::Blank.0),
    //     transitions: {
    //         let mut transitions = [None; 65536];
    //         for (st, sym, nst, nsym, dir) in r.ordered {
    //             transitions[((st.0 as usize) << 8) | (sym.0 as usize)] =
    //                 Some((SmallTmState(nst.0), SmallTmSymbol(nsym.0), dir));
    //         }
    //         transitions
    //     },
    //     state_names: STATE_NAMES.to_vec(),
    //     symbol_names: SYMBOL_NAMES.to_vec(),
    // }
});

// ════════════════════════════════════════════════════════════════════
// Tape formatting for debugging
// ════════════════════════════════════════════════════════════════════

pub fn format_tape(tape: &[Symbol]) -> String {
    tape.iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join("")
}

pub fn encode_tape<GuestSymbol: PartialEq + Eq + Hash>(
    guest_sym_to_idx: &HashMap<GuestSymbol, usize>,
    tape: &[GuestSymbol],
) -> Vec<Symbol> {
    let n_bits = num_bits(guest_sym_to_idx.len());

    let mut res = Vec::new();
    for sym in tape {
        res.push(Symbol::Comma);
        res.extend_from_slice(&to_binary(guest_sym_to_idx[sym], n_bits));
    }

    res
}
