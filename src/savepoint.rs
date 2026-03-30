use std::collections::HashMap;
use std::io::Write as IoWrite;

use serde::{Deserialize, Serialize};

use crate::delta::current_overwrites;
use crate::infinity::InfiniteTape;
use crate::tm::RunningTuringMachine;
use crate::tower::{CompiledUtmSpec, Tower, TowerLevel};
use crate::utm::{MyUtmSpec, MyUtmSpecOptimizationHints, State, Symbol};

#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    pub levels: Vec<TowerLevelJson>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TowerLevelJson {
    pub steps: u64,
    pub max_head_pos: usize,
    pub head_pos: usize,
    pub state: State,
    pub overwrites: HashMap<usize, Symbol>,
}

pub fn build_snapshot<'a>(tower: &'_ Tower<'a>, background: &InfiniteTape) -> Snapshot {
    let it = std::iter::once(TowerLevel {
        total_steps: tower.base.total_steps,
        max_head_pos: tower.base.max_head_pos,
        tm: tower.base.tm.spec.decompile(&tower.base.tm),
    })
    .chain(tower.decoded.iter().map(|l| l.clone()));
    Snapshot {
        levels: it
            .map(|l| TowerLevelJson {
                steps: l.total_steps,
                max_head_pos: l.max_head_pos,
                head_pos: l.tm.pos,
                state: l.tm.state,
                overwrites: current_overwrites(&l.tm.tape, background)
                    .iter()
                    .map(|(&i, &s)| (i, s))
                    .collect::<HashMap<_, _>>(),
            })
            .collect(),
    }
}

pub fn save_savepoint(path: &str, tower: &Tower<'_>, background: &InfiniteTape) {
    let data = build_snapshot(tower, background);
    let tmp = format!("{}.tmp", path);
    let json = serde_json::to_string(&data).expect("serialize savepoint");
    let mut f = std::io::BufWriter::new(std::fs::File::create(&tmp).expect("create savepoint"));
    f.write_all(json.as_bytes()).unwrap();
    drop(f);
    std::fs::rename(&tmp, path).expect("rename savepoint");
    eprintln!(
        "[{:?}] Saved savepoint at step {} to {}",
        std::time::Instant::now(),
        tower.base.total_steps,
        path
    );
}

pub fn load_savepoint<'a>(
    encoder: &'a MyUtmSpecOptimizationHints<'a, MyUtmSpec>,
    path: &str,
    spec: &'a CompiledUtmSpec<'a>,
    background: &InfiniteTape,
) -> Option<Tower<'a>> {
    let data = std::fs::read(path).ok()?;
    let snapshot: Snapshot = serde_json::from_slice(&data).ok()?;
    let (snap_base, snap_decoded) = snapshot
        .levels
        .split_first()
        .expect("savepoint should have at least one level");

    let mut tower = Tower::new(encoder, RunningTuringMachine::new(spec));

    tower.base = TowerLevel {
        total_steps: snap_base.steps,
        max_head_pos: snap_base.max_head_pos,
        tm: RunningTuringMachine {
            state: spec.compile_state(snap_base.state),
            tape: {
                let mut tape = Vec::new();
                for (pos, sym) in snap_base.overwrites.iter() {
                    background.extend(&mut tape, pos + 1);
                    tape[*pos] = *sym;
                }
                tape.iter().map(|&s| spec.compile_symbol(s)).collect()
            },
            pos: snap_base.head_pos,
            spec: spec,
        },
    };
    tower.decoded = snap_decoded
        .iter()
        .map(|level| TowerLevel {
            total_steps: level.steps,
            max_head_pos: level.max_head_pos,
            tm: RunningTuringMachine {
                spec: spec.guest,
                state: level.state,
                tape: {
                    let mut tape = Vec::new();
                    for (pos, sym) in level.overwrites.iter() {
                        background.extend(&mut tape, pos + 1);
                        tape[*pos] = *sym;
                    }
                    tape
                },
                pos: level.head_pos,
            },
        })
        .collect();

    Some(tower)
}
