use crate::compiled::CompiledTuringMachineSpec;
use crate::gen_utm::{Encoder, UtmSpec as _};
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
    pub encoder: &'a MyUtmSpecOptimizationHints<'a, MyUtmSpec>,
    pub base: CompiledTowerLevel<'a>,
    pub decoded: Vec<UtmTowerLevel<'a>>,
}

impl<'a> Tower<'a> {
    pub fn new(
        encoder: &'a MyUtmSpecOptimizationHints<'a, MyUtmSpec>,
        tm: RunningTuringMachine<'a, CompiledUtmSpec<'a>>,
    ) -> Self {
        Self {
            encoder,
            base: TowerLevel {
                tm,
                total_steps: 0,
                max_head_pos: 0,
            },
            decoded: Vec::new(),
        }
    }

    pub fn step(&mut self) -> RunningTMStatus {
        let prev_state = self.base.tm.state;
        let res = step(&mut self.base.tm);
        self.base.total_steps += 1;
        self.base.max_head_pos = max(self.base.max_head_pos, self.base.tm.pos);
        // eprintln!("step: {:?}", res);

        if !self
            .base
            .tm
            .spec
            .is_tick_boundary(prev_state, self.base.tm.state)
        {
            // We didn't just transition into the clean state, so decoding isn't well-defined.
            return res;
        }

        let base_decompiled = self.base.tm.spec.decompile(&self.base.tm);

        let mut cur = &base_decompiled;
        let mut decoding = self.decoded.as_mut_slice();
        while let Some((next, rest)) = decoding.split_first_mut() {
            if !decode_into_level(self.encoder, &cur, next) {
                // next level didn't enter Init, so we're done
                return res;
            }
            (cur, decoding) = (&next.tm, rest);
        }

        // we ran into the end of self.decoded, so we need to add a new level
        let new_tm = self
            .encoder
            .decode(&cur.tape)
            .expect("it should always be okay to decode a utm that just hit a tick boundary");
        let new_level: TowerLevel<UtmTm<'a>> = TowerLevel {
            total_steps: 1,
            max_head_pos: new_tm.pos,
            tm: new_tm,
        };
        self.decoded.push(new_level);

        return res;
    }
}

fn decode_into_level<'a>(
    encoder: &'a MyUtmSpecOptimizationHints<'a, MyUtmSpec>,
    tm: &'_ UtmTm<'_>,
    dst: &'_ mut UtmTowerLevel<'a>,
) -> bool {
    let decoded = encoder
        .decode(&tm.tape)
        .expect("it should always be okay to decode a utm that just hit a tick boundary");
    let old_state = dst.tm.state;
    let new_state = decoded.state;
    dst.total_steps += 1;
    dst.max_head_pos = max(dst.max_head_pos, decoded.pos);
    dst.tm = decoded;

    return encoder.guest.is_tick_boundary(old_state, new_state);
}
