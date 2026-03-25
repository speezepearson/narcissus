use std::cmp::max;
use std::fmt::Write;
use std::sync::Arc;
use std::sync::LazyLock;

use serde::Deserialize;
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
    pub max_head_pos: usize,
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
            base: TowerLevel { tm, total_steps: 0, max_head_pos: 0 },
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
        self.base.max_head_pos = max(self.base.max_head_pos, self.base.tm.pos);
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
            max_head_pos: 0,
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
    let new_total_steps = dst.total_steps + 1;
    let new_max_head_pos = max(dst.max_head_pos, decoded.pos);
    let new_head_pos = decoded.pos;
    dst.tm = decoded;
    dst.max_head_pos = new_max_head_pos;

    if new_state != old_state && new_state == State::Init {
        dst.total_steps = new_total_steps;
        return true;
    }

    return false;
}
