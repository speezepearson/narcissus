// ════════════════════════════════════════════════════════════════════
// UTM core: types, constants, rule builder, encoding, infinite tape
// ════════════════════════════════════════════════════════════════════

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::{
    gen_utm::UtmSpec,
    tm::{Dir, RunningTuringMachine, SimpleTuringMachineSpec, TuringMachineSpec},
};

// ── Newtype wrappers for type safety ──
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum State {
    Accept,
    AcceptSeekHome,
    AccFinalHome,
    AccRestAcc,
    AccRestSkip1,
    AccRestSkip2,
    AccRestState,
    ApplyReadNst,
    ChkAccBack2acc,
    ChkAccC0,
    ChkAccC0Find,
    ChkAccC0Sk1,
    ChkAccC0Sk2,
    ChkAccC1,
    ChkAccC1Find,
    ChkAccC1Sk1,
    ChkAccC1Sk2,
    ChkAccDoRest,
    ChkAccDoRest2,
    ChkAccFailBit,
    ChkAccInit,
    ChkAccIntoAcc,
    ChkAccNextEntry,
    ChkAccOk,
    ChkAccOkAcc,
    ChkAccOkSk1,
    ChkAccOkSk2,
    ChkAccOkFind,
    ChkAccOkSkip,
    ChkAccRestState,
    CmpStC0,
    CmpStC0Find,
    CmpStC1,
    CmpStC1Find,
    CmpStFail,
    CmpStNextbit,
    CmpStOk,
    CmpStRead,
    CmpSymC0,
    CmpSymC0Fb,
    CmpSymC0S1,
    CmpSymC1,
    CmpSymC1Fb,
    CmpSymC1S1,
    CmpSymFail,
    CmpSymNb2,
    CmpSymNextbit,
    CmpSymOk,
    CmpSymRead,
    CpNstC0,
    CpNstC0W,
    CpNstC1,
    CpNstC1W,
    CpNstDone,
    CpNstNext,
    CpNstNext2,
    CpNstNext3,
    CpNstRestDo,
    CpNstRestNav,
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
    LdC0,
    LdC0Bl,
    LdC0Sc,
    LdC0Tp,
    LdC0Wr,
    LdC1,
    LdC1Bl,
    LdC1Sc,
    LdC1Tp,
    LdC1Wr,
    LdDone,
    LdNavSc1,
    LdNavSc2,
    LdNavSc3,
    LdRead,
    LdRestSc,
    LdRestTp,
    LdRetBl,
    LdRetFh,
    LdRetSc,
    LdSeekHead,
    InitSeekEnd,
    InitSkip,
    MarkRule,
    MarkRuleNoMatch,
    MlFindHead,
    MlMark,
    MlNav,
    MlNavHead,
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
    MrExtNavToHead,
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
    NpMatchPre,
    NpNextbit,
    NpReadDir,
    NpSmcHandler,
    NpSymfRestore,
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
    RejRestSkip1,
    RejRestSkip2,
    RejRestState,
    SmcRestDone,
    SmcRestHead,
    SmcRestSym,
    SmcS1,
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
const ALL_STATES: [State; 189] = [
    State::Accept,
    State::AcceptSeekHome,
    State::AccFinalHome,
    State::AccRestAcc,
    State::AccRestSkip1,
    State::AccRestSkip2,
    State::AccRestState,
    State::ApplyReadNst,
    State::ChkAccBack2acc,
    State::ChkAccC0,
    State::ChkAccC0Find,
    State::ChkAccC0Sk1,
    State::ChkAccC0Sk2,
    State::ChkAccC1,
    State::ChkAccC1Find,
    State::ChkAccC1Sk1,
    State::ChkAccC1Sk2,
    State::ChkAccDoRest,
    State::ChkAccDoRest2,
    State::ChkAccFailBit,
    State::ChkAccInit,
    State::ChkAccIntoAcc,
    State::ChkAccNextEntry,
    State::ChkAccOk,
    State::ChkAccOkAcc,
    State::ChkAccOkSk1,
    State::ChkAccOkSk2,
    State::ChkAccOkFind,
    State::ChkAccOkSkip,
    State::ChkAccRestState,
    State::CmpStC0,
    State::CmpStC0Find,
    State::CmpStC1,
    State::CmpStC1Find,
    State::CmpStFail,
    State::CmpStNextbit,
    State::CmpStOk,
    State::CmpStRead,
    State::CmpSymC0,
    State::CmpSymC0Fb,
    State::CmpSymC0S1,
    State::CmpSymC1,
    State::CmpSymC1Fb,
    State::CmpSymC1S1,
    State::CmpSymFail,
    State::CmpSymNb2,
    State::CmpSymNextbit,
    State::CmpSymOk,
    State::CmpSymRead,
    State::CpNstC0,
    State::CpNstC0W,
    State::CpNstC1,
    State::CpNstC1W,
    State::CpNstDone,
    State::CpNstNext,
    State::CpNstNext2,
    State::CpNstNext3,
    State::CpNstRestDo,
    State::CpNstRestNav,
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
    State::LdC0,
    State::LdC0Bl,
    State::LdC0Sc,
    State::LdC0Tp,
    State::LdC0Wr,
    State::LdC1,
    State::LdC1Bl,
    State::LdC1Sc,
    State::LdC1Tp,
    State::LdC1Wr,
    State::LdDone,
    State::LdNavSc1,
    State::LdNavSc2,
    State::LdNavSc3,
    State::LdRead,
    State::LdRestSc,
    State::LdRestTp,
    State::LdRetBl,
    State::LdRetFh,
    State::LdRetSc,
    State::LdSeekHead,
    State::InitSeekEnd,
    State::InitSkip,
    State::MarkRule,
    State::MarkRuleNoMatch,
    State::MlFindHead,
    State::MlMark,
    State::MlNav,
    State::MlNavHead,
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
    State::MrExtNavToHead,
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
    State::NpMatchPre,
    State::NpNextbit,
    State::NpReadDir,
    State::NpSmcHandler,
    State::NpSymfRestore,
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
    State::RejRestSkip1,
    State::RejRestSkip2,
    State::RejRestState,
    State::SmcRestDone,
    State::SmcRestHead,
    State::SmcRestSym,
    State::SmcS1,
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
    R,
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
                Symbol::R => "R",
                Symbol::Dot => ".",
                Symbol::Star => "*",
                Symbol::Gt => ">",
                Symbol::Dollar => "$",
            }
        )
    }
}

impl serde::Serialize for Symbol {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
impl<'de> serde::Deserialize<'de> for Symbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "_" => Symbol::Blank,
            "0" => Symbol::Zero,
            "1" => Symbol::One,
            "X" => Symbol::X,
            "Y" => Symbol::Y,
            "#" => Symbol::Hash,
            "|" => Symbol::Pipe,
            ";" => Symbol::Semi,
            "," => Symbol::Comma,
            "^" => Symbol::Caret,
            "L" => Symbol::L,
            "R" => Symbol::R,
            "." => Symbol::Dot,
            "*" => Symbol::Star,
            ">" => Symbol::Gt,
            "$" => Symbol::Dollar,
            _ => {
                return Err(serde::de::Error::custom(format!(
                    "invalid utm symbol: {}",
                    s
                )))
            }
        })
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
    Symbol::R,
    Symbol::Dot,
    Symbol::Star,
    Symbol::Gt,
    Symbol::Dollar,
];

// ── Helpers ──
pub fn num_bits(count: usize) -> usize {
    1.max((count.max(2) as f64).log2().ceil() as usize)
}

pub fn to_binary(index: usize, width: usize) -> Vec<Symbol> {
    if index >= 1 << width {
        panic!("index {} is too large for width {}", index, width);
    }
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
            Symbol::R,
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
            Symbol::R,
            Symbol::Gt,
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
    let rule_internals: &[Symbol] = &[Zero, One, X, Y, Pipe, L, R, Comma];
    let rule_all: &[Symbol] = &[Zero, One, X, Y, Pipe, L, R, Semi, Dot, Star, Comma, Gt];
    let bits: &[Symbol] = &[Zero, One];
    let marked_bits: &[Symbol] = &[X, Y];
    let bits_and_marked: &[Symbol] = &[Zero, One, X, Y];

    // ══════════════════════════════════════════════════════════════
    // PHASE 0: INIT
    // ══════════════════════════════════════════════════════════════
    // Layout: $ ACC #[0] BLANK #[1] RULES #[2] STATE ...
    // Init starts at $ (or $+1 after seek_home), skip ACC and BLANK to reach RULES
    r.add(Init, Dollar, Init, Dollar, Dir::Right);
    // Scan right through ACCEPT content (bits, semicolons)
    scan_right(&mut r, Init, &[Zero, One, Semi]);
    // Hit #[0] → skip BLANK
    r.add(Init, Hash, InitSkip, Hash, Dir::Right);
    // InitSkip: scan right through BLANK (bits)
    scan_right(&mut r, InitSkip, bits);
    // Hit #[1] → enter RULES section
    r.add(InitSkip, Hash, InitSeekEnd, Hash, Dir::Right);
    {
        let s = InitSeekEnd;
        let mut syms: Vec<Symbol> = rule_internals.to_vec();
        syms.extend_from_slice(&[Semi, Dot, Gt]);
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
        // No more rules: seek home to check accept states
        r.add(mr, Hash, MarkRuleNoMatch, Hash, Dir::Left);
    }
    {
        // MarkRuleNoMatch: seek home (scan left to $) then enter ChkAccInit
        seek_home(&mut r, MarkRuleNoMatch, ChkAccInit);
    }

    // ══════════════════════════════════════════════════════════════
    // PHASE 2: COMPARE STATE BITS
    // ══════════════════════════════════════════════════════════════
    r.add(CmpStRead, Zero, CmpStC0, X, Dir::Right);
    r.add(CmpStRead, One, CmpStC1, Y, Dir::Right);
    r.add(CmpStRead, Pipe, StMatchCleanup, Pipe, Dir::Right);
    r.add(CmpStRead, Comma, StMatchCleanup, Comma, Dir::Right);

    for (c_sym, carry, find) in [
        (Zero, CmpStC0, CmpStC0Find),
        (One, CmpStC1, CmpStC1Find),
    ] {
        scan_right(&mut r, carry, rule_all);
        r.add(carry, Hash, find, Hash, Dir::Right);

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
        r.add(nb, Comma, StMatchCleanup, Comma, Dir::Right);
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
        r.add(gl, Comma, StmRestoreRule, Comma, Dir::Left);
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
        syms.extend_from_slice(&[Semi, Dot, Gt]);
        scan_right(&mut r, gs, &syms);
        r.add(gs, Hash, StmRestoreState, Hash, Dir::Right);
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
        // Noop rule: mark first comma as Caret to track current alternative
        r.add(ss, Comma, CmpSymRead, Caret, Dir::Right);
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
        syms.extend_from_slice(&[Semi, Hash, Pipe, Dot, L, R, Comma, Gt]);
        scan_left(&mut r, StfFindStar, &syms);
        r.add(StfFindStar, Star, StfRestoreRule, Dot, Dir::Right);
    }
    {
        let rr = StfRestoreRule;
        r.add(rr, X, rr, Zero, Dir::Right);
        r.add(rr, Y, rr, One, Dir::Right);
        scan_right(&mut r, rr, bits);
        r.add(rr, Pipe, StfGoPrev, Pipe, Dir::Left);
        r.add(rr, Comma, StfGoPrev, Comma, Dir::Left);
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
        let (carry, s1, fb) = if c == 0 {
            (CmpSymC0, CmpSymC0S1, CmpSymC0Fb)
        } else {
            (CmpSymC1, CmpSymC1S1, CmpSymC1Fb)
        };

        // Skip rules+accept to #[1]
        scan_right(&mut r, carry, rule_all);
        r.add(carry, Hash, s1, Hash, Dir::Right);

        // Skip STATE to #[2], enter SYMCACHE
        scan_right(&mut r, s1, bits_and_marked);
        r.add(s1, Hash, fb, Hash, Dir::Right);

        // Find next unmarked bit in SYMCACHE
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
        // For noop rules, scan past commas between alternatives to reach Caret
        scan_right(&mut r, nb, &[Zero, One, Comma]);
        r.add(nb, Pipe, CmpSymNb2, Pipe, Dir::Right);
        // Noop: caret marks current alternative
        r.add(nb, Caret, NpNextbit, Caret, Dir::Right);
    }
    {
        let nb2 = CmpSymNb2;
        scan_right(&mut r, nb2, marked_bits);
        r.add(nb2, Zero, CmpSymC0, X, Dir::Right);
        r.add(nb2, One, CmpSymC1, Y, Dir::Right);
        r.add(nb2, Pipe, SymMatchCleanup, Pipe, Dir::Right);
    }
    // ── Noop: NpNextbit - skip marked bits, read next bit or end-of-symbol
    {
        let np = NpNextbit;
        scan_right(&mut r, np, marked_bits);
        r.add(np, Zero, CmpSymC0, X, Dir::Right);
        r.add(np, One, CmpSymC1, Y, Dir::Right);
        // End of current noop symbol: all bits matched!
        r.add(np, Comma, NpMatchPre, Comma, Dir::Left);
        r.add(np, Pipe, NpMatchPre, Pipe, Dir::Left);
    }
    // ── Noop match: scan left to Caret, restore to Comma, enter SymMatchCleanup
    {
        let mp = NpMatchPre;
        scan_left(&mut r, mp, marked_bits);
        r.add(mp, Caret, SymMatchCleanup, Comma, Dir::Right);
    }

    // ══════════════════════════════════════════════════════════════
    // SYMBOL MISMATCH
    // ══════════════════════════════════════════════════════════════
    {
        let sf = CmpSymFail;
        scan_left(&mut r, sf, bits_and_marked);
        r.add(sf, Hash, SymfRestHead, Hash, Dir::Right);
    }
    {
        let rh = SymfRestHead;
        r.add(rh, X, rh, Zero, Dir::Right);
        r.add(rh, Y, rh, One, Dir::Right);
        scan_right(&mut r, rh, bits);
        r.add(rh, Hash, SymfSeekStar, Hash, Dir::Left);
    }
    {
        seek_star(&mut r, SymfSeekStar, SymfSkipSt);
    }
    {
        let ss = SymfSkipSt;
        // For noop rules, scan past commas to reach Caret (current alternative marker)
        scan_right(&mut r, ss, &[Zero, One, Comma]);
        r.add(ss, Pipe, SymfRestSym, Pipe, Dir::Right);
        // Noop: found current alternative marker
        r.add(ss, Caret, NpSymfRestore, Comma, Dir::Right);
    }
    // ── Noop mismatch: restore current alt marks, try next or deactivate
    {
        let nr = NpSymfRestore;
        r.add(nr, X, nr, Zero, Dir::Right);
        r.add(nr, Y, nr, One, Dir::Right);
        scan_right(&mut r, nr, bits);
        // Next alternative: mark comma as caret, re-enter symbol comparison
        r.add(nr, Comma, CmpSymRead, Caret, Dir::Right);
        // No more alternatives: deactivate rule
        r.add(nr, Pipe, SymfDeactivate, Pipe, Dir::Left);
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
        syms.push(Comma);
        scan_left(&mut r, da, &syms);
        r.add(da, Star, MarkRule, Dot, Dir::Left);
    }

    // ══════════════════════════════════════════════════════════════
    // SYMBOL MATCH CLEANUP
    // ══════════════════════════════════════════════════════════════
    {
        let sc = SymMatchCleanup;
        let mut syms: Vec<Symbol> = rule_internals.to_vec();
        syms.extend_from_slice(&[Semi, Dot, Gt]);
        scan_right(&mut r, sc, &syms);
        r.add(sc, Hash, SmcS1, Hash, Dir::Right);
    }
    {
        // SmcS1: skip STATE to #[2], enter SYMCACHE
        let s1 = SmcS1;
        scan_right(&mut r, s1, bits_and_marked);
        r.add(s1, Hash, SmcRestHead, Hash, Dir::Right);
    }
    {
        // SmcRestHead: restore SYMCACHE marks
        let rh = SmcRestHead;
        r.add(rh, X, rh, Zero, Dir::Right);
        r.add(rh, Y, rh, One, Dir::Right);
        scan_right(&mut r, rh, bits);
        r.add(rh, Hash, SmcRestDone, Hash, Dir::Left);
    }
    {
        seek_star(&mut r, SmcRestDone, SmcSkipSt);
    }
    {
        let ss = SmcSkipSt;
        scan_right(&mut r, ss, bits);
        r.add(ss, Pipe, SmcRestSym, Pipe, Dir::Right);
        // Noop: after cleanup, skip all symbol alternatives to direction
        r.add(ss, Comma, NpSmcHandler, Comma, Dir::Right);
    }
    // ── Noop post-match: restore marks, skip to | and read direction
    {
        let h = NpSmcHandler;
        r.add(h, X, h, Zero, Dir::Right);
        r.add(h, Y, h, One, Dir::Right);
        scan_right(&mut r, h, &[Zero, One, Comma]);
        r.add(h, Pipe, NpReadDir, Pipe, Dir::Right);
    }
    {
        r.add(NpReadDir, L, MoveLeft, L, Dir::Left);
        r.add(NpReadDir, R, MoveRight, R, Dir::Left);
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
        let (carry, w, mark) = if c == 0 {
            (CpNstC0, CpNstC0W, X)
        } else {
            (CpNstC1, CpNstC1W, Y)
        };

        // Skip rules+accept to #[1], then directly in STATE
        scan_right(&mut r, carry, rule_all);
        r.add(carry, Hash, w, Hash, Dir::Right);

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
        syms.extend_from_slice(&[Semi, Dot, Gt]);
        scan_right(&mut r, nav, &syms);
        r.add(nav, Hash, CpNstRestDo, Hash, Dir::Right);
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

    // Carry to head cell: skip rules, STATE, SYMCACHE, find ^
    for c in [0u8, 1u8] {
        let (carry, s1, s2, fh, fb, mark) = if c == 0 {
            (
                CpNsymC0, CpNsymC0S1, CpNsymC0S2, CpNsymC0Fh, CpNsymC0Fb, X,
            )
        } else {
            (
                CpNsymC1, CpNsymC1S1, CpNsymC1S2, CpNsymC1Fh, CpNsymC1Fb, Y,
            )
        };

        scan_right(&mut r, carry, rule_all);
        r.add(carry, Hash, s1, Hash, Dir::Right);

        // s1: skip STATE
        scan_right(&mut r, s1, bits_and_marked);
        r.add(s1, Hash, s2, Hash, Dir::Right);

        // s2: skip SYMCACHE → directly into TAPE
        scan_right(&mut r, s2, bits);
        r.add(s2, Hash, fh, Hash, Dir::Right);

        // fh: find ^ in TAPE
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
        syms.extend_from_slice(&[Semi, Dot, Gt]);
        scan_right(&mut r, nav, &syms);
        r.add(nav, Hash, CpNsymRnS1, Hash, Dir::Right);
    }
    {
        // CpNsymRnS1: skip STATE
        let s1 = CpNsymRnS1;
        scan_right(&mut r, s1, bits);
        r.add(s1, Hash, CpNsymRnS2, Hash, Dir::Right);
    }
    {
        // CpNsymRnS2: skip SYMCACHE → directly into TAPE
        let s2 = CpNsymRnS2;
        scan_right(&mut r, s2, bits);
        r.add(s2, Hash, CpNsymRnFh, Hash, Dir::Right);
    }
    {
        // CpNsymRnFh: find ^ in TAPE
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
        r.add(RdRead, R, MoveRight, R, Dir::Left);
    }

    // ══════════════════════════════════════════════════════════════
    // MOVE RIGHT
    // ══════════════════════════════════════════════════════════════
    {
        let mr = MoveRight;
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.extend_from_slice(&[Pipe, L, R, Comma]);
        scan_left(&mut r, mr, &syms);
        r.add(mr, Star, MrNav, Dot, Dir::Right);
    }
    {
        let nav = MrNav;
        let mut syms: Vec<Symbol> = rule_internals.to_vec();
        syms.extend_from_slice(&[Semi, Dot, Gt]);
        scan_right(&mut r, nav, &syms);
        r.add(nav, Hash, MrS1, Hash, Dir::Right);
    }
    {
        // MrS1: skip STATE
        let s1 = MrS1;
        scan_right(&mut r, s1, bits);
        r.add(s1, Hash, MrS2, Hash, Dir::Right);
    }
    {
        // MrS2: skip SYMCACHE → directly into TAPE
        let s2 = MrS2;
        scan_right(&mut r, s2, bits);
        r.add(s2, Hash, MrFindHead, Hash, Dir::Right);
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
        r.add(ph, Gt, LdSeekHead, Comma, Dir::Right);
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
        // MrExtHome: at $+1, skip ACCEPT to reach BLANK
        let eh = MrExtHome;
        scan_right(&mut r, eh, &[Zero, One, Semi]);
        r.add(eh, Hash, MrExtReadBlank, Hash, Dir::Right);
    }
    {
        let rb = MrExtReadBlank;
        r.add(rb, Zero, MrExtBc0, X, Dir::Right);
        r.add(rb, One, MrExtBc1, Y, Dir::Right);
        r.add(rb, Hash, MrExtRestBlank, Hash, Dir::Left);
    }
    // Carry blank bits from BLANK to end of TAPE.
    // Path: BLANK → #[1] → RULES → #[2] → STATE → #[3] → SYMCACHE → #[4] → TAPE → Blank
    // All sections between BLANK and end-of-tape are clean (no X/Y marks).
    {
        let all_between: Vec<Symbol> = vec![
            Zero, One, Hash, Comma, Caret, Pipe, Semi, Dot, L, R, Star, Gt,
        ];
        for c in [0u8, 1u8] {
            let (carry, c_sym) = if c == 0 {
                (MrExtBc0, Zero)
            } else {
                (MrExtBc1, One)
            };
            scan_right(&mut r, carry, &all_between);
            r.add(carry, Blank, MrExtBcRet, c_sym, Dir::Left);
        }
        {
            let ret = MrExtBcRet;
            scan_left(&mut r, ret, &all_between);
            r.add(ret, X, MrExtBcNext, X, Dir::Right);
            r.add(ret, Y, MrExtBcNext, Y, Dir::Right);
        }
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
        // Navigate right from BLANK boundary to find ^ in TAPE
        r.add(rb, Hash, MrExtNavToHead, Hash, Dir::Right);
    }
    {
        // MrExtNavToHead: scan right past all sections to find ^ in TAPE
        let nth = MrExtNavToHead;
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.extend_from_slice(&[Hash, Comma, Pipe, Semi, Dot, L, R, Star, Gt]);
        scan_right(&mut r, nth, &syms);
        r.add(nth, Caret, LdRead, Caret, Dir::Right);
    }

    // ══════════════════════════════════════════════════════════════
    // MOVE LEFT
    // ══════════════════════════════════════════════════════════════
    {
        let ml = MoveLeft;
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.extend_from_slice(&[Pipe, L, R, Comma]);
        scan_left(&mut r, ml, &syms);
        r.add(ml, Star, MlNav, Dot, Dir::Right);
    }
    {
        let nav = MlNav;
        let mut syms: Vec<Symbol> = rule_internals.to_vec();
        syms.extend_from_slice(&[Semi, Dot, Gt]);
        scan_right(&mut r, nav, &syms);
        r.add(nav, Hash, MlS1, Hash, Dir::Right);
    }
    {
        // MlS1: skip STATE
        let s1 = MlS1;
        scan_right(&mut r, s1, bits);
        r.add(s1, Hash, MlS2, Hash, Dir::Right);
    }
    {
        // MlS2: skip SYMCACHE → directly into TAPE
        let s2 = MlS2;
        scan_right(&mut r, s2, bits);
        r.add(s2, Hash, MlFindHead, Hash, Dir::Right);
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
        r.add(rs, Gt, MlNavHead, Comma, Dir::Left);
    }

    // ══════════════════════════════════════════════════════════════
    // PHASE 7: SEEK HOME AND RESTART
    // ══════════════════════════════════════════════════════════════
    seek_home(&mut r, DoneSeekHome, Init);

    // ══════════════════════════════════════════════════════════════
    // PHASE 8: CHECK ACCEPT STATES
    // ══════════════════════════════════════════════════════════════
    // Layout: $ ACC #[0] BLANK #[1] RULES #[2] STATE #[3] SYMCACHE #[4] TAPE
    // ChkAccInit enters at $+1 (start of ACCEPT section)
    {
        let ci = ChkAccInit;
        // If ACCEPT is empty, #[0] is right after $
        r.add(ci, Hash, RejFinalHome, Hash, Dir::Left);
        r.add(ci, Zero, ChkAccC0, X, Dir::Right);
        r.add(ci, One, ChkAccC1, Y, Dir::Right);
    }

    // Carry accept bit to STATE: skip rest of ACC → #[0] → BLANK → #[1] → RULES → #[2] → STATE
    for (carry, sk1, sk2, find) in [
        (ChkAccC0, ChkAccC0Sk1, ChkAccC0Sk2, ChkAccC0Find),
        (ChkAccC1, ChkAccC1Sk1, ChkAccC1Sk2, ChkAccC1Find),
    ] {
        // Scan right through remaining ACCEPT content
        let mut syms: Vec<Symbol> = bits_and_marked.to_vec();
        syms.push(Semi);
        scan_right(&mut r, carry, &syms);
        r.add(carry, Hash, sk1, Hash, Dir::Right);

        // sk1: skip BLANK → #[1]
        scan_right(&mut r, sk1, bits);
        r.add(sk1, Hash, sk2, Hash, Dir::Right);

        // sk2: skip RULES → #[2]
        scan_right(&mut r, sk2, rule_all);
        r.add(sk2, Hash, find, Hash, Dir::Right);

        // find: in STATE, find next unmarked bit
        scan_right(&mut r, find, marked_bits);
        if carry == ChkAccC0 {
            r.add(find, Zero, ChkAccOk, X, Dir::Left);
            r.add(find, One, ChkAccFailBit, Y, Dir::Left);
        } else {
            r.add(find, One, ChkAccOk, Y, Dir::Left);
            r.add(find, Zero, ChkAccFailBit, X, Dir::Left);
        }
    }

    // Bit matched -> go back to ACCEPT for next bit
    // From STATE, go left: #[2] → RULES → #[1] → BLANK → #[0] → ACCEPT → $
    {
        let ok = ChkAccOk;
        scan_left(&mut r, ok, bits_and_marked);
        r.add(ok, Hash, ChkAccOkSk1, Hash, Dir::Left);
    }
    {
        // ChkAccOkSk1: skip RULES leftward → #[1]
        let sk1 = ChkAccOkSk1;
        scan_left(&mut r, sk1, rule_all);
        r.add(sk1, Hash, ChkAccOkSk2, Hash, Dir::Left);
    }
    {
        // ChkAccOkSk2: skip BLANK leftward → #[0]
        let sk2 = ChkAccOkSk2;
        scan_left(&mut r, sk2, bits);
        r.add(sk2, Hash, ChkAccOkAcc, Hash, Dir::Left);
    }
    {
        // ChkAccOkAcc: in ACCEPT, scan left to $
        let oa = ChkAccOkAcc;
        let mut syms: Vec<Symbol> = bits_and_marked.to_vec();
        syms.push(Semi);
        scan_left(&mut r, oa, &syms);
        r.add(oa, Dollar, ChkAccOkFind, Dollar, Dir::Right);
    }
    {
        // ChkAccOkFind: scan right to find next marked bit (resume position)
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

    // Bit mismatch -> restore STATE marks, return to ACCEPT, restore entry, try next
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
        // ChkAccBack2acc: go left from right boundary of STATE, cross RULES and BLANK to ACCEPT
        let ba = ChkAccBack2acc;
        scan_left(&mut r, ba, bits);
        r.add(ba, Hash, ChkAccIntoAcc, Hash, Dir::Left);
    }
    {
        // ChkAccIntoAcc: scan left through RULES, BLANK, into ACCEPT, to $
        let ia = ChkAccIntoAcc;
        let mut syms: Vec<Symbol> = bits_and_marked.to_vec();
        syms.extend_from_slice(&[Semi, Hash, Pipe, Dot, L, R, Comma, Star, Gt]);
        scan_left(&mut r, ia, &syms);
        r.add(ia, Dollar, ChkAccDoRest, Dollar, Dir::Right);
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
        r.add(ash, Dollar, AccRestAcc, Dollar, Dir::Right);
    }
    {
        let ra = AccRestAcc;
        r.add(ra, X, ra, Zero, Dir::Right);
        r.add(ra, Y, ra, One, Dir::Right);
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.push(Semi);
        scan_right(&mut r, ra, &syms);
        // Hit #[0] → skip BLANK → #[1] → skip RULES → #[2] → STATE
        r.add(ra, Hash, AccRestSkip1, Hash, Dir::Right);
    }
    {
        // AccRestSkip1: skip BLANK
        let sk1 = AccRestSkip1;
        scan_right(&mut r, sk1, bits);
        r.add(sk1, Hash, AccRestSkip2, Hash, Dir::Right);
    }
    {
        // AccRestSkip2: skip RULES
        let sk2 = AccRestSkip2;
        scan_right(&mut r, sk2, rule_all);
        r.add(sk2, Hash, AccRestState, Hash, Dir::Right);
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
        r.add(rsh, Dollar, RejRestAcc, Dollar, Dir::Right);
    }
    {
        let ra = RejRestAcc;
        r.add(ra, X, ra, Zero, Dir::Right);
        r.add(ra, Y, ra, One, Dir::Right);
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.push(Semi);
        scan_right(&mut r, ra, &syms);
        // Hit #[0] → skip BLANK → #[1] → skip RULES → #[2] → STATE
        r.add(ra, Hash, RejRestSkip1, Hash, Dir::Right);
    }
    {
        // RejRestSkip1: skip BLANK
        let sk1 = RejRestSkip1;
        scan_right(&mut r, sk1, bits);
        r.add(sk1, Hash, RejRestSkip2, Hash, Dir::Right);
    }
    {
        // RejRestSkip2: skip RULES
        let sk2 = RejRestSkip2;
        scan_right(&mut r, sk2, rule_all);
        r.add(sk2, Hash, RejRestState, Hash, Dir::Right);
    }
    {
        let rs = RejRestState;
        r.add(rs, X, rs, Zero, Dir::Right);
        r.add(rs, Y, rs, One, Dir::Right);
        scan_right(&mut r, rs, bits);
        r.add(rs, Hash, RejFinalHome, Hash, Dir::Left);
    }
    seek_home(&mut r, RejFinalHome, Reject);

    // ══════════════════════════════════════════════════════════════
    // LOAD PHASE: copy new head cell from TAPE to SYMCACHE
    // ══════════════════════════════════════════════════════════════

    // Entry points after move:
    // MoveRight: MrPlaceHead → LdSeekHead → scan right past old cell to ^
    {
        scan_right(&mut r, LdSeekHead, bits);
        r.add(LdSeekHead, Caret, LdRead, Caret, Dir::Right);
    }
    // MoveLeft: MlRestore → MlNavHead → scan left to ^
    {
        scan_left(&mut r, MlNavHead, bits);
        r.add(MlNavHead, Caret, LdRead, Caret, Dir::Right);
    }

    // LdRead: at head cell in TAPE, find next unmarked bit
    {
        scan_right(&mut r, LdRead, marked_bits);
        r.add(LdRead, Zero, LdC0, X, Dir::Left);
        r.add(LdRead, One, LdC1, Y, Dir::Left);
        r.add(LdRead, Comma, LdDone, Comma, Dir::Left);
        r.add(LdRead, Blank, LdDone, Blank, Dir::Left);
    }

    // LdC0/LdC1: carry bit left from head cell through TAPE → SYMCACHE
    // (No BLANK between SYMCACHE and TAPE in new layout)
    for c in [0u8, 1u8] {
        let (carry, tp, sc, wr, mark) = if c == 0 {
            (LdC0, LdC0Tp, LdC0Sc, LdC0Wr, X)
        } else {
            (LdC1, LdC1Tp, LdC1Sc, LdC1Wr, Y)
        };

        // Scan left past marks in head cell to ^
        scan_left(&mut r, carry, marked_bits);
        r.add(carry, Caret, tp, Caret, Dir::Left);

        // tp: scan left through TAPE cells to #[4]
        {
            let mut syms: Vec<Symbol> = bits.to_vec();
            syms.push(Comma);
            scan_left(&mut r, tp, &syms);
            r.add(tp, Hash, sc, Hash, Dir::Left);
        }

        // sc: scan left through SYMCACHE to #[3]
        {
            let mut syms: Vec<Symbol> = bits.to_vec();
            syms.extend_from_slice(&[X, Y]);
            scan_left(&mut r, sc, &syms);
            r.add(sc, Hash, wr, Hash, Dir::Right);
        }

        // wr: in SYMCACHE, scan right past already-written X/Y, write at next position
        scan_right(&mut r, wr, marked_bits);
        r.add(wr, Zero, LdRetSc, mark, Dir::Right);
        r.add(wr, One, LdRetSc, mark, Dir::Right);
    }

    // LdRetSc: return from SYMCACHE to TAPE
    {
        // Scan right through rest of SYMCACHE to #[4]
        {
            let mut syms: Vec<Symbol> = bits.to_vec();
            syms.extend_from_slice(&[X, Y]);
            scan_right(&mut r, LdRetSc, &syms);
            r.add(LdRetSc, Hash, LdRetFh, Hash, Dir::Right);
        }
    }
    {
        // LdRetFh: scan right through TAPE to ^
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.push(Comma);
        scan_right(&mut r, LdRetFh, &syms);
        r.add(LdRetFh, Caret, LdRead, Caret, Dir::Right);
    }

    // LdDone: all bits loaded, restore marks
    {
        // Go left: restore TAPE head cell marks (fix: must restore current bit too)
        r.add(LdDone, X, LdRestTp, Zero, Dir::Left);
        r.add(LdDone, Y, LdRestTp, One, Dir::Left);
        r.add(LdDone, Caret, LdNavSc1, Caret, Dir::Left);
    }
    {
        let rt = LdRestTp;
        r.add(rt, X, rt, Zero, Dir::Left);
        r.add(rt, Y, rt, One, Dir::Left);
        scan_left(&mut r, rt, bits);
        r.add(rt, Caret, LdNavSc1, Caret, Dir::Left);
    }

    // Navigate left from ^ to SYMCACHE
    {
        // LdNavSc1: scan left through TAPE to #[4]
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.push(Comma);
        scan_left(&mut r, LdNavSc1, &syms);
        r.add(LdNavSc1, Hash, LdNavSc2, Hash, Dir::Left);
    }
    {
        // LdNavSc2: scan left through SYMCACHE (has X/Y marks) to #[3]
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.extend_from_slice(&[X, Y]);
        scan_left(&mut r, LdNavSc2, &syms);
        r.add(LdNavSc2, Hash, LdRestSc, Hash, Dir::Right);
    }
    {
        // LdRestSc: restore SYMCACHE marks, then seek home
        let rs = LdRestSc;
        r.add(rs, X, rs, Zero, Dir::Right);
        r.add(rs, Y, rs, One, Dir::Right);
        scan_right(&mut r, rs, bits);
        r.add(rs, Hash, DoneSeekHome, Hash, Dir::Left);
    }

    r
}

// ════════════════════════════════════════════════════════════════════
// build_utm_spec: Assemble the full TuringMachineSpec for the UTM
// ════════════════════════════════════════════════════════════════════

pub type MyUtmSpec = SimpleTuringMachineSpec<State, Symbol>;
impl UtmSpec for MyUtmSpec {
    fn encode<Guest: TuringMachineSpec>(
        &self,
        tm: &RunningTuringMachine<Guest>,
    ) -> Vec<Self::Symbol> {
        self.encode_optimized(tm, &MyUtmSpecOptimizationHints::guess(tm.spec))
    }
    fn decode<'a, Guest: TuringMachineSpec>(
        &self,
        guest: &'a Guest,
        tape: &[Self::Symbol],
    ) -> Result<RunningTuringMachine<'a, Guest>, String> {
        let guest_states: Vec<Guest::State> = guest.iter_states().collect();
        let guest_symbols: Vec<Guest::Symbol> = guest.iter_symbols().collect();

        let n_state_bits = num_bits(guest_states.len());
        let n_sym_bits = num_bits(guest_symbols.len());

        // Find the sections separated by #
        // Layout: $ ACC #[0] BLANK #[1] RULES #[2] STATE #[3] SYMCACHE #[4] TAPE
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

    fn is_tick_boundary(&self, prev_state: State, state: State) -> bool {
        prev_state != state && state == State::DoneSeekHome
    }
}

/// Step a UTM until `is_tick_boundary` fires (or it halts).
/// Takes at least one step before checking.
/// Returns Ok(num_steps) on tick, Err on halt or step limit.
#[allow(dead_code)]
pub fn run_until_inner_step<Spec: UtmSpec>(
    spec: &Spec,
    tm: &mut RunningTuringMachine<Spec>,
    max_steps: usize,
) -> Result<usize, crate::tm::RunUntilResult> {
    use crate::tm::{step, RunUntilResult, RunningTMStatus};

    let mut prev_state = tm.state;

    for step_count in 1..=max_steps {
        if tm.pos >= tm.tape.len() {
            tm.tape.resize(tm.pos + 1, spec.blank());
        }
        match step(tm) {
            RunningTMStatus::Running => {
                if tm.pos >= tm.tape.len() {
                    tm.tape.resize(tm.pos + 1, spec.blank());
                }
                if spec.is_tick_boundary(prev_state, tm.state) {
                    return Ok(step_count);
                }
                prev_state = tm.state;
            }
            RunningTMStatus::Accepted => {
                return Err(RunUntilResult::Accepted {
                    num_steps: step_count,
                });
            }
            RunningTMStatus::Rejected => {
                return Err(RunUntilResult::Rejected {
                    num_steps: step_count,
                });
            }
        }
    }
    Err(RunUntilResult::StepLimit)
}

pub struct TmTransitionStats<Guest: TuringMachineSpec>(
    pub HashMap<(Guest::State, Guest::Symbol), usize>,
);
impl<Guest: TuringMachineSpec> Default for TmTransitionStats<Guest> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}
impl<Guest: TuringMachineSpec> TmTransitionStats<Guest> {
    pub fn make_optimization_hints(&self, guest: &Guest) -> MyUtmSpecOptimizationHints<Guest> {
        MyUtmSpecOptimizationHints {
            rule_order: self.get_optimal_rule_order(guest),
            state_encodings: self.get_optimal_state_encoding(guest),
            symbol_encodings: self.get_optimal_symbol_encoding(guest),
        }
    }

    pub fn get_optimal_rule_order(&self, guest: &Guest) -> Vec<(Guest::State, Guest::Symbol)> {
        let mut rules = guest
            .iter_rules()
            .map(|(st, sym, _, _, _)| (st, sym))
            .collect::<Vec<_>>();
        rules.sort_by_key(|&(st, sym)| self.0.get(&(st, sym)).unwrap_or(&0));
        rules
    }

    pub fn get_optimal_state_encoding(&self, guest: &Guest) -> HashMap<Guest::State, usize> {
        // todo!()
        guest
            .iter_states()
            .enumerate()
            .map(|(i, s)| (s, i))
            .collect()
    }

    pub fn get_optimal_symbol_encoding(&self, guest: &Guest) -> HashMap<Guest::Symbol, usize> {
        // todo!()
        guest
            .iter_symbols()
            .enumerate()
            .map(|(i, s)| (s, i))
            .collect()
    }
}

pub struct MyUtmSpecOptimizationHints<Guest: TuringMachineSpec> {
    pub rule_order: Vec<(Guest::State, Guest::Symbol)>,
    pub state_encodings: HashMap<Guest::State, usize>,
    pub symbol_encodings: HashMap<Guest::Symbol, usize>,
}
impl<Guest: TuringMachineSpec> MyUtmSpecOptimizationHints<Guest> {
    pub fn guess(guest: &Guest) -> Self {
        let stats = TmTransitionStats::default();
        stats.make_optimization_hints(guest)
    }
}

impl MyUtmSpec {
    /// Encode a guest TM, optionally reordering rules so that `last_rules`
    /// appear at the end of the rules section (in the given order).
    /// Rules not in `last_rules` appear first (in `iter_rules` order).
    ///
    /// The UTM scans rules right-to-left, so placing frequently-used rules
    /// last reduces search time.
    pub fn encode_optimized<Guest: TuringMachineSpec>(
        &self,
        guest: &RunningTuringMachine<Guest>,
        hints: &MyUtmSpecOptimizationHints<Guest>,
    ) -> Vec<Symbol> {
        if hints.state_encodings.len() != guest.spec.iter_states().count() {
            panic!("state encodings length mismatch");
        }
        if hints.symbol_encodings.len() != guest.spec.iter_symbols().count() {
            panic!("symbol encodings length mismatch");
        }
        if hints.rule_order.len() != guest.spec.iter_rules().count() {
            panic!("rule order length mismatch");
        }
        let n_state_bits = num_bits(hints.state_encodings.len());
        let n_sym_bits = num_bits(hints.symbol_encodings.len());

        // Collect all rules
        type Rule<S, Y> = (S, Y, S, Y, Dir);
        let all_rules: Vec<Rule<Guest::State, Guest::Symbol>> = guest.spec.iter_rules().collect();

        // Reorder rules if last_rules is provided
        let ordered_rules: Vec<&Rule<Guest::State, Guest::Symbol>> = {
            let last_set: HashSet<(Guest::State, Guest::Symbol)> =
                hints.rule_order.iter().copied().collect();
            let mut front: Vec<&Rule<Guest::State, Guest::Symbol>> = all_rules
                .iter()
                .filter(|(st, sym, _, _, _)| !last_set.contains(&(*st, *sym)))
                .collect();
            // Append last_rules in the specified order
            for &(lst, lsym) in &hints.rule_order {
                if let Some(rule) = all_rules
                    .iter()
                    .find(|(st, sym, _, _, _)| *st == lst && *sym == lsym)
                {
                    front.push(rule);
                }
            }
            front
        };

        let mut tape: Vec<Symbol> = Vec::new();
        tape.push(Symbol::Dollar);

        // Layout: $ ACCEPT # BLANK # RULES # STATE # SYMCACHE # TAPE

        // ACCEPT section (right after $)
        for (i, state) in guest
            .spec
            .iter_states()
            .filter(|s| guest.spec.is_accepting(*s))
            .enumerate()
        {
            if i > 0 {
                tape.push(Symbol::Semi);
            }
            tape.extend_from_slice(&to_binary(hints.state_encodings[&state], n_state_bits));
        }

        // BLANK section
        tape.push(Symbol::Hash);
        tape.extend_from_slice(&to_binary(
            hints.symbol_encodings[&guest.spec.blank()],
            n_sym_bits,
        ));

        // Identify noop rules: (state, sym) -> (state, sym, dir) where nst==st && nsym==sym
        // Group noop rules by (state_encoding, dir) for compact encoding
        let mut noop_groups: HashMap<(usize, Dir), Vec<usize>> = HashMap::new();
        let mut noop_set: HashSet<(Guest::State, Guest::Symbol)> = HashSet::new();
        // Track last position of each noop group for placement
        let mut noop_last_pos: HashMap<(usize, Dir), usize> = HashMap::new();
        for (i, &&(st, sym, nst, nsym, dir)) in ordered_rules.iter().enumerate() {
            if nst == st && nsym == sym {
                let st_idx = hints.state_encodings[&st];
                let sym_idx = hints.symbol_encodings[&sym];
                noop_groups.entry((st_idx, dir)).or_default().push(sym_idx);
                noop_set.insert((st, sym));
                noop_last_pos.insert((st_idx, dir), i);
            }
        }

        // RULES section: # .rule1 ; .rule2 ; .rule3 ...
        tape.push(Symbol::Hash);
        let mut first_rule = true;
        for (i, &&(st, sym, nst, nsym, dir)) in ordered_rules.iter().enumerate() {
            let is_noop = noop_set.contains(&(st, sym));
            if is_noop {
                // Emit compact noop rule at the position of the last member
                let st_idx = hints.state_encodings[&st];
                if noop_last_pos.get(&(st_idx, dir)) != Some(&i) {
                    continue; // Skip; will be emitted at last position
                }
                let syms = &noop_groups[&(st_idx, dir)];
                if !first_rule {
                    tape.push(Symbol::Semi);
                }
                first_rule = false;
                // Format: . STATE , SYM1 , SYM2 , ... | DIR
                tape.push(Symbol::Dot);
                tape.extend_from_slice(&to_binary(st_idx, n_state_bits));
                for &sym_idx in syms {
                    tape.push(Symbol::Comma);
                    tape.extend_from_slice(&to_binary(sym_idx, n_sym_bits));
                }
                tape.push(Symbol::Pipe);
                tape.push(match dir {
                    Dir::Left => Symbol::L,
                    Dir::Right => Symbol::R,
                });
            } else {
                if !first_rule {
                    tape.push(Symbol::Semi);
                }
                first_rule = false;
                tape.push(Symbol::Dot);
                tape.extend_from_slice(&to_binary(hints.state_encodings[&st], n_state_bits));
                tape.push(Symbol::Pipe);
                tape.extend_from_slice(&to_binary(hints.symbol_encodings[&sym], n_sym_bits));
                tape.push(Symbol::Pipe);
                tape.extend_from_slice(&to_binary(hints.state_encodings[&nst], n_state_bits));
                tape.push(Symbol::Pipe);
                tape.extend_from_slice(&to_binary(hints.symbol_encodings[&nsym], n_sym_bits));
                tape.push(Symbol::Pipe);
                tape.push(match dir {
                    Dir::Left => Symbol::L,
                    Dir::Right => Symbol::R,
                });
            }
        }

        // STATE section
        tape.push(Symbol::Hash);
        tape.extend_from_slice(&to_binary(
            hints.state_encodings[&guest.state],
            n_state_bits,
        ));

        // SYMCACHE section: copy of head cell symbol
        tape.push(Symbol::Hash);
        {
            let default_tape = vec![guest.spec.blank()];
            let nonempty_guest_tape = if guest.tape.is_empty() {
                &default_tape
            } else {
                &guest.tape
            };
            let head_sym = nonempty_guest_tape[guest.pos];
            tape.extend_from_slice(&to_binary(hints.symbol_encodings[&head_sym], n_sym_bits));
        }

        // TAPE section
        tape.push(Symbol::Hash);

        let caret_pos = tape.len();

        let default_tape = vec![guest.spec.blank()];
        let nonempty_guest_tape = if guest.tape.is_empty() {
            &default_tape
        } else {
            &guest.tape
        };
        for sym in nonempty_guest_tape {
            tape.push(Symbol::Comma);
            tape.extend_from_slice(&to_binary(hints.symbol_encodings[&sym], n_sym_bits));
        }
        tape[caret_pos] = Symbol::Caret;

        tape
    }
}

pub fn make_utm_spec() -> MyUtmSpec {
    SimpleTuringMachineSpec {
        initial: State::Init,
        accepting: HashSet::from([State::Accept]),
        blank: Symbol::Blank,
        transitions: build_utm_rules().0,
        all_states: ALL_STATES.to_vec(),
        all_symbols: ALL_SYMBOLS.to_vec(),
    }
}
