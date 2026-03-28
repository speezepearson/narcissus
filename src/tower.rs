use crate::compiled::{CState, CompiledTuringMachineSpec};
use crate::infinity::InfiniteTape;
use crate::tm::{step, RunningTMStatus, RunningTuringMachine, SimpleTuringMachineSpec};
use crate::utm::{MyUtmSpec, MyUtmSpecOptimizationHints};
use crate::utm::{State, Symbol};
use std::cmp::max;

pub type UtmTm<'a> = RunningTuringMachine<'a, MyUtmSpec>;
pub type CompiledUtmSpec<'a> = CompiledTuringMachineSpec<'a, MyUtmSpec>;

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
    pub utm_spec: &'a MyUtmSpec,
    pub hints: &'a MyUtmSpecOptimizationHints<MyUtmSpec>,
    pub base: CompiledTowerLevel<'a>,
    pub decoded: Vec<UtmTowerLevel<'a>>,
    pub clean_compiled_state: CState,
}

impl<'a> Tower<'a> {
    pub fn new(
        utm_spec: &'a MyUtmSpec,
        hints: &'a MyUtmSpecOptimizationHints<MyUtmSpec>,
        tm: RunningTuringMachine<'a, CompiledUtmSpec<'a>>,
    ) -> Self {
        let clean_compiled_state = tm.spec.compile_state(State::Init);
        Self {
            utm_spec,
            hints,
            base: TowerLevel {
                tm,
                total_steps: 0,
                max_head_pos: 0,
            },
            decoded: Vec::new(),
            clean_compiled_state,
        }
    }

    /// Step the tower, extending the tape from the background as needed.
    pub fn step_extending(&mut self, background: &InfiniteTape) -> RunningTMStatus {
        let compiled = self.base.tm.spec;
        background.extend_compiled(&mut self.base.tm.tape, self.base.tm.pos + 1, compiled);
        self.step()
    }

    pub fn step(&mut self) -> RunningTMStatus {
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
            if !decode_into_level(self.utm_spec, self.hints, cur, next) {
                // next level didn't enter Init, so we're done
                return res;
            }
            (cur, decoding) = (&next.tm, rest);
        }

        // we ran into the end of self.decoded, so we need to add a new level
        let new_level: TowerLevel<UtmTm<'a>> = TowerLevel {
            total_steps: 0,
            max_head_pos: 0,
            tm: self
                .utm_spec
                .decode_optimized(self.utm_spec, &cur.tape, self.hints)
                .expect("it should always be okay to decode a utm that just entered Init"),
        };
        self.decoded.push(new_level);

        return res;
    }
}

fn decode_into_level<'a>(
    utm_spec: &'a MyUtmSpec,
    hints: &'a MyUtmSpecOptimizationHints<MyUtmSpec>,
    tm: &'_ UtmTm<'_>,
    dst: &'_ mut UtmTowerLevel<'a>,
) -> bool {
    let decoded = utm_spec
        .decode_optimized(utm_spec, &tm.tape, hints)
        .expect("it should always be okay to decode a utm that just entered Init");
    let old_state = dst.tm.state;
    let new_state = decoded.state;
    dst.total_steps += 1;
    dst.max_head_pos = max(dst.max_head_pos, decoded.pos);
    dst.tm = decoded;

    return new_state != old_state && new_state == State::Init;
}
