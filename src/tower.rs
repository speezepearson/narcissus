use std::cmp::max;
use std::fmt::Write;
use std::sync::Arc;
use std::sync::LazyLock;

use serde::Serialize;

use crate::compiled::CompiledTapeExtender;
use crate::compiled::{CState, CompiledTuringMachineSpec};
use crate::tm::{
    step, RunningTMStatus, RunningTuringMachine, SimpleTuringMachineSpec, TapeExtender,
    TuringMachineSpec,
};
use crate::utm::UTM_SPEC;
use crate::utm::{MyUtmEncodingScheme, State, Symbol, UtmEncodingScheme};

pub type UtmTm<'a> = RunningTuringMachine<'a, SimpleTuringMachineSpec<State, Symbol>>;
pub type CompiledUtmSpec<'a> =
    CompiledTuringMachineSpec<'a, SimpleTuringMachineSpec<State, Symbol>>;

#[derive(Clone)]
pub struct TowerLevel<TM> {
    pub tm: TM,
    pub total_steps: u64,
}
pub type CompiledTowerLevel<'a> = TowerLevel<RunningTuringMachine<'a, CompiledUtmSpec<'a>>>;
pub type UtmTowerLevel<'a> =
    TowerLevel<RunningTuringMachine<'a, SimpleTuringMachineSpec<State, Symbol>>>;

pub struct Tower<'a> {
    pub base: CompiledTowerLevel<'a>,
    pub decoded: Vec<UtmTowerLevel<'a>>,
    pub clean_compiled_state: CState,
}

impl<'a> Tower<'a> {
    pub fn new(tm: RunningTuringMachine<'a, CompiledUtmSpec<'a>>) -> Self {
        let clean_compiled_state = tm.spec.compile_state(State::Init);
        Self {
            base: TowerLevel { tm, total_steps: 0 },
            decoded: Vec::new(),
            clean_compiled_state,
        }
    }

    pub fn step(
        &mut self,
        extender: &mut CompiledTapeExtender<SimpleTuringMachineSpec<State, Symbol>>,
    ) -> RunningTMStatus {
        if self.base.tm.pos >= self.base.tm.tape.len() {
            extender.extend(&mut self.base.tm.tape, self.base.tm.pos + 1);
        }
        let prev_state = self.base.tm.state;
        let res = step(&mut self.base.tm);
        self.base.total_steps += 1;
        // eprintln!("step: {:?}", res);

        if self.base.tm.state == prev_state || self.base.tm.state != self.clean_compiled_state {
            // We didn't just transition into the clean state, so decoding isn't well-defined.
            return res;
        }

        let base_decompiled = self.base.tm.spec.decompile(&self.base.tm);

        let mut cur = &base_decompiled;
        let mut decoding = self.decoded.as_mut_slice();
        while let Some((next, rest)) = decoding.split_first_mut() {
            if !decode_into_level(cur, next) {
                // next level didn't enter Init, so we're done
                return res;
            }
            (cur, decoding) = (&next.tm, rest);
        }

        // we ran into the end of self.decoded, so we need to add a new level
        let new_level = TowerLevel {
            total_steps: 0,
            tm: MyUtmEncodingScheme::decode(&*UTM_SPEC, &cur.tape)
                .expect("it should always be okay to decode a utm that just entered Init"),
        };
        self.decoded.push(new_level);

        return res;
    }
}

fn decode_into_level<'a>(tm: &UtmTm<'a>, dst: &mut UtmTowerLevel<'a>) -> bool {
    let decoded = MyUtmEncodingScheme::decode(&*UTM_SPEC, &tm.tape)
        .expect("it should always be okay to decode a utm that just entered Init");
    let old_state = dst.tm.state;
    let new_state = decoded.state;
    dst.tm = decoded;

    if new_state != old_state && new_state == State::Init {
        dst.total_steps += 1;
        return true;
    }

    return false;
}

#[derive(Serialize)]
pub struct TowerLevelJson {
    pub tape: String,
    pub head_pos: usize,
    pub state: String,
    pub tape_len: usize,
}

#[derive(Serialize)]
pub struct TowerJson {
    pub steps: u64,
    pub steps_per_sec: f64,
    pub tower: Vec<TowerLevelJson>,
}

fn tape_string(tm: &UtmTm, end: usize) -> String {
    let mut out = String::new();
    let blank = tm.spec.blank();
    for i in 0..end {
        let sym = if i < tm.tape.len() { tm.tape[i] } else { blank };
        write!(out, "{}", sym).unwrap();
    }
    if end < tm.tape.len() {
        out.push_str(" ...");
    }
    out
}

fn level_to_json(tm: &UtmTm, end: usize) -> TowerLevelJson {
    TowerLevelJson {
        tape: tape_string(tm, end),
        head_pos: tm.pos,
        state: format!("{:?}", tm.state),
        tape_len: tm.tape.len(),
    }
}
