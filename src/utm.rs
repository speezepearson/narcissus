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
    // Navigation states for new tape layout (ACC ↔ STATE crossing)
    ChkAccC0SkBk,
    ChkAccC0SkR,
    ChkAccC1SkBk,
    ChkAccC1SkR,
    ChkAccOkSkR,
    ChkAccOkSkBk,
    ChkAccBkSkR,
    ChkAccBkSkBk,
    AccRstSkBk,
    AccRstSkR,
    RejRstSkBk,
    RejRstSkR,
    NmSkipBk,
}
const ALL_STATES: [State; 179] = [
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
    State::ChkAccC0SkBk,
    State::ChkAccC0SkR,
    State::ChkAccC1SkBk,
    State::ChkAccC1SkR,
    State::ChkAccOkSkR,
    State::ChkAccOkSkBk,
    State::ChkAccBkSkR,
    State::ChkAccBkSkBk,
    State::AccRstSkBk,
    State::AccRstSkR,
    State::RejRstSkBk,
    State::RejRstSkR,
    State::NmSkipBk,
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
            Symbol::Dollar,
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
    let rule_internals: &[Symbol] = &[Zero, One, X, Y, Pipe, L, R];
    let rule_all: &[Symbol] = &[Zero, One, X, Y, Pipe, L, R, Semi, Dot, Star];
    let bits: &[Symbol] = &[Zero, One];
    let marked_bits: &[Symbol] = &[X, Y];
    let bits_and_marked: &[Symbol] = &[Zero, One, X, Y];

    // ══════════════════════════════════════════════════════════════
    // PHASE 0: INIT
    // New layout: # ACC # BLANK # RULES $ STATE # TAPE
    // Init scans right to $, then goes left into RULES for MarkRule
    // ══════════════════════════════════════════════════════════════
    {
        scan_right(
            &mut r,
            Init,
            &[Zero, One, X, Y, Hash, Pipe, Semi, Comma, Caret, Dot, Star, Gt, L, R],
        );
        r.add(Init, Dollar, MarkRule, Dollar, Dir::Left);
    }

    // ══════════════════════════════════════════════════════════════
    // PHASE 1: MARK RULE (right-to-left search)
    // ══════════════════════════════════════════════════════════════
    {
        let mr = MarkRule;
        scan_left(&mut r, mr, rule_internals);
        r.add(mr, Semi, mr, Semi, Dir::Left);
        r.add(mr, Dot, CmpStRead, Star, Dir::Right);
        r.add(mr, Hash, MarkRuleNoMatch, Hash, Dir::Left);
    }
    // MarkRuleNoMatch: no matching rule. Navigate LEFT to ACC section.
    // Layout: # ACC # BLANK # RULES (we're at hashes[2])
    // Go left through BLANK to hashes[1], then left through ACC to hashes[0]
    {
        let nm = MarkRuleNoMatch;
        // Skip BLANK going left
        scan_left(&mut r, nm, bits);
        r.add(nm, Hash, NmSkipBk, Hash, Dir::Left);
    }
    {
        // Skip ACC going left to hashes[0], then go right into ACC
        let sk = NmSkipBk;
        let mut syms: Vec<Symbol> = bits_and_marked.to_vec();
        syms.push(Semi);
        scan_left(&mut r, sk, &syms);
        r.add(sk, Hash, ChkAccInit, Hash, Dir::Right);
    }

    // ══════════════════════════════════════════════════════════════
    // PHASE 2: COMPARE STATE BITS
    // ══════════════════════════════════════════════════════════════
    r.add(CmpStRead, Zero, CmpStC0, X, Dir::Right);
    r.add(CmpStRead, One, CmpStC1, Y, Dir::Right);
    r.add(CmpStRead, Pipe, StMatchCleanup, Pipe, Dir::Right);

    // In new layout: RULES $ STATE — no ACC to skip
    for (c_sym, carry, find) in [
        (Zero, CmpStC0, CmpStC0Find),
        (One, CmpStC1, CmpStC1Find),
    ] {
        scan_right(&mut r, carry, rule_all);
        r.add(carry, Dollar, find, Dollar, Dir::Right);

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
        // New layout: RULES $ STATE — go directly from $ to STATE
        r.add(gs, Dollar, StmRestoreState, Dollar, Dir::Right);
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
        // New layout: left of STATE is $ (not #)
        r.add(sf, Dollar, StfRestoreState, Dollar, Dir::Right);
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
        syms.extend_from_slice(&[Semi, Hash, Pipe, Dot, L, R, Dollar]);
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

    // New layout: RULES $ STATE # TAPE — skip STATE only (no ACC, no BLANK)
    for c in [0u8, 1u8] {
        let (carry, s1, fh, fb) = if c == 0 {
            (CmpSymC0, CmpSymC0S1, CmpSymC0Fh, CmpSymC0Fb)
        } else {
            (CmpSymC1, CmpSymC1S1, CmpSymC1Fh, CmpSymC1Fb)
        };

        scan_right(&mut r, carry, rule_all);
        r.add(carry, Dollar, s1, Dollar, Dir::Right);

        // Skip STATE
        scan_right(&mut r, s1, bits_and_marked);
        r.add(s1, Hash, fh, Hash, Dir::Right);
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
        // New layout: RULES $ STATE # TAPE
        r.add(sc, Dollar, SmcS1, Dollar, Dir::Right);
    }
    {
        // Skip STATE (no ACC, no BLANK)
        let s1 = SmcS1;
        scan_right(&mut r, s1, bits_and_marked);
        r.add(s1, Hash, SmcFh, Hash, Dir::Right);
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

    // New layout: RULES $ STATE — no ACC to skip
    for c in [0u8, 1u8] {
        let (carry, w, mark) = if c == 0 {
            (CpNstC0, CpNstC0W, X)
        } else {
            (CpNstC1, CpNstC1W, Y)
        };

        scan_right(&mut r, carry, rule_all);
        r.add(carry, Dollar, w, Dollar, Dir::Right);

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
        // New layout: RULES $ STATE — go directly to STATE
        r.add(nav, Dollar, CpNstRestDo, Dollar, Dir::Right);
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

    // Carry to head cell: new layout RULES $ STATE # TAPE — skip STATE only
    for c in [0u8, 1u8] {
        let (carry, s1, fh, fb, mark) = if c == 0 {
            (CpNsymC0, CpNsymC0S1, CpNsymC0Fh, CpNsymC0Fb, X)
        } else {
            (CpNsymC1, CpNsymC1S1, CpNsymC1Fh, CpNsymC1Fb, Y)
        };

        scan_right(&mut r, carry, rule_all);
        r.add(carry, Dollar, s1, Dollar, Dir::Right);

        // Skip STATE
        scan_right(&mut r, s1, bits_and_marked);
        r.add(s1, Hash, fh, Hash, Dir::Right);

        // Find ^
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
        // New layout: RULES $ STATE # TAPE
        r.add(nav, Dollar, CpNsymRnS1, Dollar, Dir::Right);
    }
    {
        // Skip STATE only (no ACC, no BLANK)
        let s1 = CpNsymRnS1;
        scan_right(&mut r, s1, bits);
        r.add(s1, Hash, CpNsymRnFh, Hash, Dir::Right);
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
        r.add(RdRead, R, MoveRight, R, Dir::Left);
    }

    // ══════════════════════════════════════════════════════════════
    // MOVE RIGHT
    // ══════════════════════════════════════════════════════════════
    {
        let mr = MoveRight;
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.extend_from_slice(&[Pipe, L, R]);
        scan_left(&mut r, mr, &syms);
        r.add(mr, Star, MrNav, Dot, Dir::Right);
    }
    {
        let nav = MrNav;
        let mut syms: Vec<Symbol> = rule_internals.to_vec();
        syms.extend_from_slice(&[Semi, Dot]);
        scan_right(&mut r, nav, &syms);
        // New layout: RULES $ STATE # TAPE
        r.add(nav, Dollar, MrS1, Dollar, Dir::Right);
    }
    {
        // Skip STATE only (no ACC, no BLANK)
        let s1 = MrS1;
        scan_right(&mut r, s1, bits);
        r.add(s1, Hash, MrFindHead, Hash, Dir::Right);
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
    // New layout: # ACC # BLANK # RULES $ STATE # TAPE
    // BLANK is at the far left, so we navigate left from $ to reach it
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
    // MrExtWriteHead: scan left to $ then navigate left to BLANK
    {
        scan_left(
            &mut r,
            MrExtWriteHead,
            &[Zero, One, X, Y, Hash, Pipe, Semi, Comma, Caret, Dot, Star, Gt, L, R],
        );
        r.add(MrExtWriteHead, Dollar, MrExtHome, Dollar, Dir::Left);
    }
    {
        // MrExtHome: scan left through RULES to # (hashes[2])
        let eh = MrExtHome;
        scan_left(&mut r, eh, rule_all);
        r.add(eh, Hash, MrExtH1, Hash, Dir::Left);
    }
    {
        // MrExtH1: scan left through BLANK to # (hashes[1]), then go right
        let h1 = MrExtH1;
        scan_left(&mut r, h1, bits);
        r.add(h1, Hash, MrExtReadBlank, Hash, Dir::Right);
    }
    {
        let rb = MrExtReadBlank;
        r.add(rb, Zero, MrExtBc0, X, Dir::Right);
        r.add(rb, One, MrExtBc1, Y, Dir::Right);
        // All bits read: hit # (hashes[2]), restore marks
        r.add(rb, Hash, MrExtRestBlank, Hash, Dir::Left);
    }
    // Carry bits from BLANK across BLANK remainder, #, RULES, $, STATE, #, TAPE to Blank
    for c in [0u8, 1u8] {
        let (carry, c_sym) = if c == 0 {
            (MrExtBc0, Zero)
        } else {
            (MrExtBc1, One)
        };
        let mut syms: Vec<Symbol> = bits_and_marked.to_vec();
        syms.extend_from_slice(&[Hash, Pipe, Semi, Comma, Caret, Dot, Star, L, R, Dollar, Gt]);
        scan_right(&mut r, carry, &syms);
        r.add(carry, Blank, MrExtBcRet, c_sym, Dir::Left);
    }
    {
        // Return from tape end back to marked bit in BLANK
        let ret = MrExtBcRet;
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.extend_from_slice(&[Hash, Pipe, Semi, Comma, Caret, Dot, Star, L, R, Dollar, Gt]);
        scan_left(&mut r, ret, &syms);
        r.add(ret, X, MrExtBcNext, X, Dir::Right);
        r.add(ret, Y, MrExtBcNext, Y, Dir::Right);
    }
    {
        let next = MrExtBcNext;
        scan_right(&mut r, next, marked_bits);
        r.add(next, Zero, MrExtBc0, X, Dir::Right);
        r.add(next, One, MrExtBc1, Y, Dir::Right);
        // All bits copied: hit # (hashes[2])
        r.add(next, Hash, MrExtRestBlank, Hash, Dir::Left);
    }
    {
        // Restore BLANK marks, then navigate right to $ for MarkRule
        let rb = MrExtRestBlank;
        r.add(rb, X, rb, Zero, Dir::Left);
        r.add(rb, Y, rb, One, Dir::Left);
        scan_left(&mut r, rb, bits);
        // Hit # (hashes[1]). Go right through BLANK, #, RULES to $
        r.add(rb, Hash, MrExtH2, Hash, Dir::Right);
    }
    {
        // MrExtH2: skip BLANK going right
        let h2 = MrExtH2;
        scan_right(&mut r, h2, bits);
        r.add(h2, Hash, MrExtH3, Hash, Dir::Right);
    }
    {
        // MrExtH3: skip RULES going right to $
        let h3 = MrExtH3;
        scan_right(&mut r, h3, rule_all);
        r.add(h3, Dollar, MarkRule, Dollar, Dir::Left);
    }

    // ══════════════════════════════════════════════════════════════
    // MOVE LEFT
    // ══════════════════════════════════════════════════════════════
    {
        let ml = MoveLeft;
        let mut syms: Vec<Symbol> = bits.to_vec();
        syms.extend_from_slice(&[Pipe, L, R]);
        scan_left(&mut r, ml, &syms);
        r.add(ml, Star, MlNav, Dot, Dir::Right);
    }
    {
        let nav = MlNav;
        let mut syms: Vec<Symbol> = rule_internals.to_vec();
        syms.extend_from_slice(&[Semi, Dot]);
        scan_right(&mut r, nav, &syms);
        // New layout: RULES $ STATE # TAPE
        r.add(nav, Dollar, MlS1, Dollar, Dir::Right);
    }
    {
        // Skip STATE only
        let s1 = MlS1;
        scan_right(&mut r, s1, bits);
        r.add(s1, Hash, MlFindHead, Hash, Dir::Right);
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
    // New layout: scan left from TAPE through # STATE to $, then
    // go left into RULES for MarkRule
    // ══════════════════════════════════════════════════════════════
    scan_left(
        &mut r,
        DoneSeekHome,
        &[Zero, One, X, Y, Hash, Pipe, Semi, Comma, Caret, Dot, Star, Gt, L, R],
    );
    r.add(DoneSeekHome, Dollar, MarkRule, Dollar, Dir::Left);

    // ══════════════════════════════════════════════════════════════
    // PHASE 8: CHECK ACCEPT STATES
    // New layout: # ACC # BLANK # RULES $ STATE # TAPE
    // ACC and STATE are now far apart. Navigation requires crossing
    // BLANK and RULES sections. This path is cold (only on halt).
    // ══════════════════════════════════════════════════════════════
    {
        let ci = ChkAccInit;
        r.add(ci, Hash, RejFinalHome, Hash, Dir::Left);
        r.add(ci, Zero, ChkAccC0, X, Dir::Right);
        r.add(ci, One, ChkAccC1, Y, Dir::Right);
    }

    // ChkAccC0/C1: carry bit from ACC right to STATE
    // Path: ACC → # (hashes[1]) → BLANK → # (hashes[2]) → RULES → $ → STATE
    for (carry, sk_bk, sk_r, find) in [
        (ChkAccC0, ChkAccC0SkBk, ChkAccC0SkR, ChkAccC0Find),
        (ChkAccC1, ChkAccC1SkBk, ChkAccC1SkR, ChkAccC1Find),
    ] {
        let mut syms: Vec<Symbol> = bits_and_marked.to_vec();
        syms.push(Semi);
        scan_right(&mut r, carry, &syms);
        r.add(carry, Hash, sk_bk, Hash, Dir::Right);

        // Skip BLANK
        scan_right(&mut r, sk_bk, bits);
        r.add(sk_bk, Hash, sk_r, Hash, Dir::Right);

        // Skip RULES
        scan_right(&mut r, sk_r, rule_all);
        r.add(sk_r, Dollar, find, Dollar, Dir::Right);

        scan_right(&mut r, find, marked_bits);
        if carry == ChkAccC0 {
            r.add(find, Zero, ChkAccOk, X, Dir::Left);
            r.add(find, One, ChkAccFailBit, Y, Dir::Left);
        } else {
            r.add(find, One, ChkAccOk, Y, Dir::Left);
            r.add(find, Zero, ChkAccFailBit, X, Dir::Left);
        }
    }

    // Bit matched -> go back from STATE to ACC for next bit
    // Path: STATE → $ → RULES → # (hashes[2]) → BLANK → # (hashes[1]) → ACC
    {
        let ok = ChkAccOk;
        scan_left(&mut r, ok, bits_and_marked);
        r.add(ok, Dollar, ChkAccOkSkR, Dollar, Dir::Left);
    }
    {
        scan_left(&mut r, ChkAccOkSkR, rule_all);
        r.add(ChkAccOkSkR, Hash, ChkAccOkSkBk, Hash, Dir::Left);
    }
    {
        scan_left(&mut r, ChkAccOkSkBk, bits);
        r.add(ChkAccOkSkBk, Hash, ChkAccOkAcc, Hash, Dir::Left);
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

    // Bit mismatch -> restore STATE marks, navigate back to ACC
    {
        let fb = ChkAccFailBit;
        scan_left(&mut r, fb, bits_and_marked);
        // Left of STATE is $ (not #)
        r.add(fb, Dollar, ChkAccRestState, Dollar, Dir::Right);
    }
    {
        let rs = ChkAccRestState;
        r.add(rs, X, rs, Zero, Dir::Right);
        r.add(rs, Y, rs, One, Dir::Right);
        scan_right(&mut r, rs, bits);
        r.add(rs, Hash, ChkAccBack2acc, Hash, Dir::Left);
    }
    // ChkAccBack2acc: go from STATE left to ACC
    // Path: STATE → $ → RULES → # → BLANK → # → ACC
    {
        let ba = ChkAccBack2acc;
        scan_left(&mut r, ba, bits);
        r.add(ba, Dollar, ChkAccBkSkR, Dollar, Dir::Left);
    }
    {
        scan_left(&mut r, ChkAccBkSkR, rule_all);
        r.add(ChkAccBkSkR, Hash, ChkAccBkSkBk, Hash, Dir::Left);
    }
    {
        scan_left(&mut r, ChkAccBkSkBk, bits);
        r.add(ChkAccBkSkBk, Hash, ChkAccIntoAcc, Hash, Dir::Left);
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

    // Accept: restore ACCEPTSTATES and STATE
    // AcceptSeekHome starts in ACC, goes left to hashes[0]
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
        // Hit hashes[1] (between ACC and BLANK). Navigate to STATE.
        r.add(ra, Hash, AccRstSkBk, Hash, Dir::Right);
    }
    {
        // Skip BLANK right
        scan_right(&mut r, AccRstSkBk, bits);
        r.add(AccRstSkBk, Hash, AccRstSkR, Hash, Dir::Right);
    }
    {
        // Skip RULES right to $
        scan_right(&mut r, AccRstSkR, rule_all);
        r.add(AccRstSkR, Dollar, AccRestState, Dollar, Dir::Right);
    }
    {
        let rs = AccRestState;
        r.add(rs, X, rs, Zero, Dir::Right);
        r.add(rs, Y, rs, One, Dir::Right);
        scan_right(&mut r, rs, bits);
        r.add(rs, Hash, AccFinalHome, Hash, Dir::Left);
    }
    // AccFinalHome: scan left to $ (always reachable since ACC restore
    // ends at hashes[3], and $ is between RULES and STATE)
    seek_home(&mut r, AccFinalHome, Accept);

    // Reject: restore marks (same navigation pattern)
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
        r.add(ra, Hash, RejRstSkBk, Hash, Dir::Right);
    }
    {
        scan_right(&mut r, RejRstSkBk, bits);
        r.add(RejRstSkBk, Hash, RejRstSkR, Hash, Dir::Right);
    }
    {
        scan_right(&mut r, RejRstSkR, rule_all);
        r.add(RejRstSkR, Dollar, RejRestState, Dollar, Dir::Right);
    }
    {
        let rs = RejRestState;
        r.add(rs, X, rs, Zero, Dir::Right);
        r.add(rs, Y, rs, One, Dir::Right);
        scan_right(&mut r, rs, bits);
        r.add(rs, Hash, RejFinalHome, Hash, Dir::Left);
    }
    // RejFinalHome: scan to $ and halt. Can be entered from:
    // 1. RejRestState at hashes[3] going left → $ is leftward (seek_home works)
    // 2. ChkAccInit at hashes[0] → $ is rightward (seek_home fails)
    // Handle both by scanning left to $ OR scanning past tape start.
    // Since $ may be to the right, scan right instead.
    {
        scan_right(
            &mut r,
            RejFinalHome,
            &[Zero, One, X, Y, Hash, Pipe, Semi, Comma, Caret, Dot, Star, Gt, L, R],
        );
        r.add(RejFinalHome, Dollar, Reject, Dollar, Dir::Right);
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

        // New layout: #[0] ACC #[1] BLANK #[2] RULES $ STATE #[3] TAPE
        // Find $ to locate STATE, find # after $ for TAPE
        let dollar_pos = tape
            .iter()
            .position(|&s| s == Symbol::Dollar)
            .ok_or("no $ delimiter found")?;

        let state_start = dollar_pos + 1;
        let state = guest_states[from_binary_at(tape, state_start, n_state_bits)];

        // Find first # after $
        let tape_hash = tape[dollar_pos..]
            .iter()
            .position(|&s| s == Symbol::Hash)
            .ok_or("no # after $ for TAPE section")?
            + dollar_pos;
        let tape_start = tape_hash + 1;
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

    fn at_tick(&self, state: State, _symbol: Symbol) -> bool {
        // A tick occurs at the start of each rule-matching cycle:
        // - Init: freshly created machine (before any steps)
        // - MarkRule: start of each new inner step
        state == State::Init || state == State::MarkRule
    }
}

/// Step a UTM until `at_tick` returns true (or it halts).
/// Takes at least one step before checking.
/// Returns Ok(num_steps) on tick, Err on halt or step limit.
#[allow(dead_code)]
pub fn run_until_at_tick<Spec: UtmSpec>(
    spec: &Spec,
    tm: &mut RunningTuringMachine<Spec>,
    max_steps: usize,
) -> Result<usize, crate::tm::RunUntilResult> {
    use crate::tm::{step, RunUntilResult, RunningTMStatus};

    for step_count in 1..=max_steps {
        if tm.pos >= tm.tape.len() {
            tm.tape.resize(tm.pos + 1, spec.blank());
        }
        match step(tm) {
            RunningTMStatus::Running => {
                // Check at_tick with the state we just entered and
                // the symbol the head is now looking at
                if tm.pos >= tm.tape.len() {
                    tm.tape.resize(tm.pos + 1, spec.blank());
                }
                if spec.at_tick(tm.state, tm.tape[tm.pos]) {
                    return Ok(step_count);
                }
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

        // New layout: # ACC # BLANK # RULES $ STATE # TAPE
        let mut tape: Vec<Symbol> = Vec::new();

        // ACC section
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
            tape.extend_from_slice(&to_binary(hints.state_encodings[&state], n_state_bits));
        }

        // BLANK section
        tape.push(Symbol::Hash);
        tape.extend_from_slice(&to_binary(
            hints.symbol_encodings[&guest.spec.blank()],
            n_sym_bits,
        ));

        // RULES section
        tape.push(Symbol::Hash);
        let mut first_rule = true;
        for &(st, sym, nst, nsym, dir) in &ordered_rules {
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

        // $ separator
        tape.push(Symbol::Dollar);

        // STATE section
        tape.extend_from_slice(&to_binary(
            hints.state_encodings[&guest.state],
            n_state_bits,
        ));

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
