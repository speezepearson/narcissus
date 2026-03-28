use std::collections::HashMap;

use utmmmmm::gen_utm::UtmSpec as _;
use utmmmmm::tm::{step, RunningTMStatus, RunningTuringMachine, TuringMachineSpec};
use utmmmmm::toy_machines::{DoubleXSymbol, DOUBLE_X_SPEC};
use utmmmmm::utm::{make_utm_spec, State, Symbol};

fn main() {
    let utm_spec = make_utm_spec();
    let guest_spec = &*DOUBLE_X_SPEC;

    let mut guest_tm = RunningTuringMachine::new(guest_spec);
    guest_tm.tape = vec![
        DoubleXSymbol::Dollar,
        DoubleXSymbol::X,
        DoubleXSymbol::X,
        DoubleXSymbol::X,
        DoubleXSymbol::X,
        DoubleXSymbol::X,
        DoubleXSymbol::X,
        DoubleXSymbol::X,
    ];

    let encoded = utm_spec.encode(&guest_tm);
    let mut utm_tm = RunningTuringMachine::new(&utm_spec);
    utm_tm.tape = encoded;

    // States that write "real" carets (head movement), vs comparison carets
    // MrSkipCell writes Caret (line 1165): MoveRight placing new head
    // MlMark writes Caret (line 1300): MoveLeft placing new head
    // MrExtToBlank writes Caret (line 1183): MoveRight extend placing new head
    let head_move_caret_states = [State::MrSkipCell, State::MlMark, State::MrExtToBlank];

    let mut transition_counts: HashMap<(State, Symbol), usize> = HashMap::new();
    let mut caret_number = 0;
    let mut total_intervals = 0;

    let mut interval_appearances: HashMap<(State, Symbol), usize> = HashMap::new();
    let mut interval_exactly_once: HashMap<(State, Symbol), usize> = HashMap::new();

    let max_steps = 200_000_000;

    for _ in 0..max_steps {
        if utm_tm.pos >= utm_tm.tape.len() {
            utm_tm.tape.resize(utm_tm.pos + 1, utm_spec.blank());
        }

        let state = utm_tm.state;
        let sym = utm_tm.tape[utm_tm.pos];

        if let Some((_, new_sym, _)) = utm_spec.get_transition(state, sym) {
            *transition_counts.entry((state, sym)).or_insert(0) += 1;

            if new_sym == Symbol::Caret && head_move_caret_states.contains(&state) {
                caret_number += 1;
                total_intervals += 1;

                for (&key, &count) in &transition_counts {
                    *interval_appearances.entry(key).or_insert(0) += 1;
                    if count == 1 {
                        *interval_exactly_once.entry(key).or_insert(0) += 1;
                    }
                }
                transition_counts.clear();
            }
        }

        match step(&mut utm_tm) {
            RunningTMStatus::Running => {}
            RunningTMStatus::Accepted | RunningTMStatus::Rejected => {
                break;
            }
        }
    }

    println!("Total head-move carets: {}", caret_number);
    println!();

    // Find transitions that appeared in EVERY interval AND were exactly-once in every interval
    let mut always_once: Vec<_> = interval_appearances
        .iter()
        .filter(|(&key, &appearances)| {
            appearances == total_intervals
                && interval_exactly_once.get(&key) == Some(&total_intervals)
        })
        .map(|(&(st, sy), _)| (st, sy))
        .collect();
    always_once.sort_by_key(|&(st, _)| format!("{:?}", st));

    println!("Transitions occurring exactly once in EVERY interval ({} intervals):", total_intervals);
    for (st, sy) in &always_once {
        println!("  ({:?}, {:?})", st, sy);
    }
    println!("  ({} transitions)", always_once.len());

    // Also show transitions that appeared in MOST intervals and were exactly-once
    println!();
    println!("Transitions exactly-once in >95% of intervals:");
    let threshold = (total_intervals as f64 * 0.95) as usize;
    let mut most_once: Vec<_> = interval_appearances
        .iter()
        .filter(|(&key, &appearances)| {
            appearances >= threshold
                && interval_exactly_once.get(&key).copied().unwrap_or(0) >= threshold
        })
        .map(|(&(st, sy), &apps)| {
            let once = interval_exactly_once.get(&(st, sy)).copied().unwrap_or(0);
            (st, sy, apps, once)
        })
        .collect();
    most_once.sort_by_key(|&(st, _, _, _)| format!("{:?}", st));

    for (st, sy, apps, once) in &most_once {
        println!("  ({:?}, {:?}) — in {}/{} intervals, exactly-once in {}", st, sy, apps, total_intervals, once);
    }
}
