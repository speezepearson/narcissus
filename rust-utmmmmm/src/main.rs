mod compiled;
mod infinity;
mod optimization_hints;
mod tm;
#[allow(dead_code)]
mod toy_machines;
mod utm;

use std::fmt::Debug;
use tm::{
    run_until_enters_state, RunUntilResult, RunningTuringMachine, TapeExtender, TuringMachineSpec,
};
use utm::*;

use crate::compiled::{CState, CompiledTapeExtender, CompiledTuringMachineSpec};
use crate::infinity::{header_len, InfiniteTapeExtender};

const RADIUS: usize = 30;

/// Format a tape view: 30 symbols on each side of the head, with ^ below.
fn tape_view<Spec: TuringMachineSpec<Symbol = Symbol>>(tm: &RunningTuringMachine<Spec>) -> String
where
    Spec::State: Debug,
{
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

/// Must be larger than the UTM header to allow decode to find all 5 `#` delimiters.
fn min_display_tape_len() -> usize {
    // Header + some extra cells so decode has tape content to work with.
    header_len() + 1_000
}

fn print_tower<Spec: TuringMachineSpec<Symbol = Symbol>>(
    tm: &RunningTuringMachine<Spec>,
    steps: u64,
) where
    Spec::State: Debug,
{
    let utm = &*UTM_SPEC;
    let mut extender = InfiniteTapeExtender;

    eprintln!(
        "═══ {} steps ═══════════════════════════════════════",
        steps
    );

    eprintln!("Level 0 (outermost UTM):");
    let tape_str = format_tape(&tm.tape);
    eprintln!("{}", &tape_str[..tape_str.len().min(1000)]);

    eprintln!("Level 0 tape view:");
    eprintln!("{}", tape_view(tm));

    // Extend a copy of the tape so decode can see the full encoding
    let mut full_tape = tm.tape.clone();
    extender.extend(&mut full_tape, min_display_tape_len());

    match MyUtmEncodingScheme::decode(utm, &full_tape) {
        Ok(mut level1) => {
            // Level 1's tape contains the guest symbols of the simulated UTM.
            // To decode level 2, we need the full UTM encoding of level 1's machine,
            // which we get by re-encoding level 1 and extending it.
            extender.extend(&mut level1.tape, min_display_tape_len());
            eprintln!("Level 1 (simulated UTM, {} symbols):", level1.tape.len());
            eprintln!("{}", tape_view(&level1));

            let mut level1_encoded = MyUtmEncodingScheme::encode(&level1);
            extender.extend(&mut level1_encoded, min_display_tape_len());

            match MyUtmEncodingScheme::decode(utm, &level1_encoded) {
                Ok(mut level2) => {
                    extender.extend(&mut level2.tape, min_display_tape_len());
                    eprintln!(
                        "Level 2 (simulated simulated UTM, {} symbols):",
                        level2.tape.len()
                    );
                    eprintln!("{}", tape_view(&level2));
                }
                Err(e) => {
                    eprintln!("Level 2: (unable to decode: {})", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Level 1: (unable to decode: {})", e);
            eprintln!("Level 2: (unable to decode)");
        }
    }

    eprintln!();
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
    // Initialize tape with at least one cell
    extender.extend(&mut tm.tape, 1);

    let mut total_steps: u64 = 0;
    let mut guest_steps: u64 = 0;
    let decompiled = compiled.decompile(&tm);
    print_tower(&decompiled, total_steps);

    loop {
        match run_until_enters_state(&mut tm, init_cstate, usize::MAX, Some(&mut extender)) {
            Ok(steps) => {
                total_steps += steps as u64;
                guest_steps += 1;
                let decompiled = compiled.decompile(&tm);
                eprintln!(
                    "Guest step {} completed after {} UTM steps (total: {})",
                    guest_steps, steps, total_steps
                );
                print_tower(&decompiled, total_steps);
            }
            Err(RunUntilResult::Accepted { num_steps }) => {
                total_steps += num_steps as u64;
                let decompiled = compiled.decompile(&tm);
                print_tower(&decompiled, total_steps);
                println!(
                    "halted (accept) in state {:?} after {} UTM steps ({} guest steps)",
                    decompiled.state, total_steps, guest_steps
                );
                break;
            }
            Err(RunUntilResult::Rejected { num_steps }) => {
                total_steps += num_steps as u64;
                let decompiled = compiled.decompile(&tm);
                print_tower(&decompiled, total_steps);
                println!(
                    "halted (reject) in state {:?} after {} UTM steps ({} guest steps)",
                    decompiled.state, total_steps, guest_steps
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
