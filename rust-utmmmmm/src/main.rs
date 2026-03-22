mod compiled;
mod infinity;
mod optimization_hints;
mod tm;
#[allow(dead_code)]
mod toy_machines;
mod utm;

use std::cmp::max;
use std::fmt::Write;
use tm::{
    run_until_enters_state, RunUntilResult, RunningTuringMachine, TapeExtender, TuringMachineSpec,
};
use utm::*;

use crate::compiled::{CState, CompiledTapeExtender, CompiledTuringMachineSpec};
use crate::infinity::{header_len, InfiniteTapeExtender};
use crate::tm::SimpleTuringMachineSpec;

type UtmTm<'a> = RunningTuringMachine<'a, SimpleTuringMachineSpec<State, Symbol>>;

const RADIUS: usize = 30;

/// Format a tape view: 30 symbols on each side of the head, with ^ below.
fn tape_view(tm: &UtmTm) -> String {
    let mut top = String::new();
    let mut bot = String::new();

    let prefix = if tm.pos > RADIUS { "... " } else { "    " };
    top.push_str(prefix);
    bot.push_str("    ");

    for i in 0..(2 * RADIUS + 1) {
        let tape_idx = tm.pos as isize + i as isize - RADIUS as isize;
        let sym = if tape_idx < 0 {
            " ".to_string()
        } else {
            let idx = tape_idx as usize;
            if idx < tm.tape.len() {
                tm.tape[idx].to_string()
            } else {
                tm.spec.blank().to_string()
            }
        };
        top.push_str(&sym);

        if i == RADIUS {
            bot.push('^');
        } else {
            bot.push(' ');
        }
    }

    top.push_str(" ...");
    bot.push_str(&format!(" (state={:?}, pos={})", tm.state, tm.pos));

    format!("{}\n{}", top, bot)
}

/// Decode tower[level+1] from tower[level], extending the tape as needed.
/// Returns None if decoding fails (tape too short, etc.)
fn decode_next_level<'a>(
    utm: &'a SimpleTuringMachineSpec<State, Symbol>,
    parent: &mut UtmTm<'a>,
    extender: &mut InfiniteTapeExtender,
) -> Option<UtmTm<'a>> {
    // Ensure parent tape is long enough for decoding
    let min_len = max(header_len(), parent.pos + 100);
    extender.extend(&mut parent.tape, min_len);
    MyUtmEncodingScheme::decode(utm, &parent.tape).ok()
}

/// Build the tower by decoding each level from the previous.
/// tower[0] = decompiled L0, tower[1] = decode(tower[0]), etc.
/// Re-decodes level i+1 when level i entered Init (compared to prev_states).
/// Grows the tower by at most one new level per call.
fn update_tower<'a>(
    utm: &'a SimpleTuringMachineSpec<State, Symbol>,
    tower: &mut Vec<UtmTm<'a>>,
    prev_states: &mut Vec<State>,
    extender: &mut InfiniteTapeExtender,
) {
    // Walk the tower: re-decode level+1 from level when level entered Init.
    // Level 0 is always freshly set by the caller, so always decode level 1.
    let mut level = 0;
    loop {
        // Check whether this level just entered Init
        let entered_init = if level < prev_states.len() {
            tower[level].state == State::Init && prev_states[level] != State::Init
        } else {
            // New level we haven't seen before — don't cascade further
            false
        };

        // Level 0 always triggers decoding of level 1 (caller just set it).
        // Deeper levels only trigger if they entered Init.
        if level > 0 && !entered_init {
            break;
        }

        // Decode the next level from this one
        if let Some(next) = decode_next_level(utm, &mut tower[level], extender) {
            if level + 1 < tower.len() {
                tower[level + 1] = next;
            } else {
                // Grow the tower by one
                tower.push(next);
                break; // Don't cascade into the brand-new level
            }
            level += 1;
        } else {
            break;
        }
    }

    // Snapshot current states for next comparison
    prev_states.clear();
    for tm in tower.iter() {
        prev_states.push(tm.state);
    }
}

fn format_tower(tower: &[UtmTm], total_steps: u64) -> String {
    let mut buf = String::new();
    writeln!(
        buf,
        "═══ {} steps ═══════════════════════════════════════",
        total_steps
    )
    .unwrap();

    for (i, tm) in tower.iter().enumerate() {
        writeln!(buf, "Level {} ({} symbols):", i, tm.tape.len()).unwrap();
        writeln!(buf, "{}", tape_view(tm)).unwrap();
    }
    buf
}

fn main() {
    let utm = &*UTM_SPEC;
    let compiled = CompiledTuringMachineSpec::compile(utm).expect("UTM should compile");

    // Find the CState corresponding to State::Init
    let init_cstate = compiled
        .original_states
        .iter()
        .position(|&s| s == State::Init)
        .map(|i| CState(i as u8))
        .expect("Init state should exist");

    let mut tm = RunningTuringMachine::new(&compiled);
    let mut extender = CompiledTapeExtender::new(&compiled, Box::new(InfiniteTapeExtender));
    extender.extend(&mut tm.tape, 1);

    let mut total_steps: u64 = 0;
    let mut guest_steps: u64 = 0;
    let mut inf_extender = InfiniteTapeExtender;

    // Initialize tower from the starting state
    let mut tower: Vec<UtmTm> = vec![compiled.decompile(&tm)];
    let mut prev_states: Vec<State> = Vec::new();
    update_tower(utm, &mut tower, &mut prev_states, &mut inf_extender);
    eprint!("{}", format_tower(&tower, total_steps));

    loop {
        match run_until_enters_state(&mut tm, init_cstate, usize::MAX, Some(&mut extender)) {
            Ok(steps) => {
                total_steps += steps as u64;
                guest_steps += 1;

                tower[0] = compiled.decompile(&tm);
                update_tower(utm, &mut tower, &mut prev_states, &mut inf_extender);

                eprintln!(
                    "Guest step {} after {} UTM steps (total: {})",
                    guest_steps, steps, total_steps
                );
                eprint!("{}", format_tower(&tower, total_steps));
            }
            Err(
                RunUntilResult::Accepted { num_steps } | RunUntilResult::Rejected { num_steps },
            ) => {
                total_steps += num_steps as u64;
                tower[0] = compiled.decompile(&tm);
                update_tower(utm, &mut tower, &mut prev_states, &mut inf_extender);
                eprint!("{}", format_tower(&tower, total_steps));
                let status = if compiled.is_accepting(tm.state) {
                    "accept"
                } else {
                    "reject"
                };
                println!(
                    "halted ({}) in state {:?} after {} UTM steps ({} guest steps)",
                    status, tower[0].state, total_steps, guest_steps
                );
                break;
            }
            Err(RunUntilResult::StepLimit) => {
                eprintln!("step limit reached after {} UTM steps", total_steps);
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests;
