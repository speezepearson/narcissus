mod compiled;
mod infinity;
mod tm;
mod toy_machines;
mod utm;

fn main() {}

use utm::*;

use crate::tm::RunningTuringMachine;

const RADIUS: usize = 30;

/// Format a tape view: 30 symbols on each side of the head, with ^ below.
fn tape_view(
    tape: &[usize],
    head_pos: usize,
    state_name: &str,
    symbol_names: &[&str],
    blank_idx: usize,
) -> String {
    let mut top = String::new();
    let mut bot = String::new();

    let prefix = if head_pos > RADIUS { "... " } else { "    " };
    top.push_str(prefix);
    bot.push_str("    ");

    for i in 0..(2 * RADIUS + 1) {
        let tape_idx = head_pos as isize + i as isize - RADIUS as isize;
        let sym = if tape_idx < 0 {
            " "
        } else {
            let idx = tape_idx as usize;
            if idx < tape.len() {
                symbol_names.get(tape[idx]).unwrap_or(&"?")
            } else {
                symbol_names.get(blank_idx).unwrap_or(&"_")
            }
        };
        top.push_str(sym);

        if i == RADIUS {
            bot.push('^');
        } else {
            bot.push(' ');
        }
    }

    top.push_str(" ...");
    bot.push_str(&format!(" (state={}, pos={})", state_name, head_pos));

    format!("{}\n{}", top, bot)
}

fn print_tower(utm: RunningTuringMachine<utm> tape: &[Symbol], pos: usize, state: State, steps: u64) {
    let utm = &*UTM_SPEC;

    eprintln!(
        "═══ {} steps ═══════════════════════════════════════",
        steps
    );

    eprintln!("Level 0 (outermost UTM):");
    eprintln!("{}", format_tape(tape));

    let outer_tape: Vec<usize> = tape.iter().map(|s| s.0 as usize).collect();
    if let Some(level1) = try_decode(&outer_tape, &utm) {
        let state_name = utm.state_names.get(level1.state).unwrap_or(&"?");
        eprintln!("Level 1 (simulated UTM):");
        eprintln!(
            "{}",
            tape_view(
                &level1.tape,
                level1.head_pos,
                state_name,
                &SYMBOL_NAMES,
                SYM_BLANK.0 as usize,
            )
        );

        if let Some(level2) = try_decode(&level1.tape, &utm) {
            let state_name = utm.state_names.get(level2.state).unwrap_or(&"?");
            eprintln!("Level 2 (simulated simulated UTM):");
            eprintln!(
                "{}",
                tape_view(
                    &level2.tape,
                    level2.head_pos,
                    state_name,
                    &SYMBOL_NAMES,
                    SYM_BLANK.0 as usize,
                )
            );
        } else {
            eprintln!("Level 2: (unable to decode)");
        }
    } else {
        eprintln!("Level 1: (unable to decode)");
        eprintln!("Level 2: (unable to decode)");
    }

    eprintln!();
}

fn main() {
    let header = infinite_utm_tape_header();
    let n_sym_bits = num_bits(N_SYMBOLS);
    let cell_size = 1 + n_sym_bits;

    // Need enough cells so that decoding Level 1 yields a tape long enough
    // to contain the Level 2 header (~28K symbols) + some tape cells.
    let initial_cells = 35000;
    let initial_len = header.len() + initial_cells * cell_size;
    let mut tape: Vec<Symbol> = Vec::with_capacity(initial_len);
    for i in 0..initial_len {
        tape.push(infinite_utm_tape_background(
            &header, n_sym_bits, cell_size, i,
        ));
    }

    let spec = build_utm_spec();
    let mut state = spec.initial;
    let mut pos: usize = 0;
    let mut steps: u64 = 0;

    let extend =
        |tape: &mut Vec<Symbol>, header: &[Symbol], n_sym_bits: usize, cell_size: usize| {
            let old_len = tape.len();
            let new_len = old_len + 1024 * cell_size;
            tape.reserve(new_len - old_len);
            for i in old_len..new_len {
                tape.push(infinite_utm_tape_background(
                    header, n_sym_bits, cell_size, i,
                ));
            }
        };

    print_tower(&tape, pos, state, steps);

    loop {
        if pos >= tape.len() {
            extend(&mut tape, &header, n_sym_bits, cell_size);
        }
        let sym = tape[pos];
        let key = ((state.0 as usize) << 8) | (sym.0 as usize);
        if let Some((ns, nsym, dir)) = spec.transitions[key] {
            state = ns;
            tape[pos] = nsym;
            pos = match dir {
                Dir::Left => pos.saturating_sub(1),
                Dir::Right => pos + 1,
            };
            steps += 1;
            if steps % 100_000_000 == 0 {
                print_tower(&tape, pos, state, steps);
            }
        } else {
            break;
        }
    }

    print_tower(&tape, pos, state, steps);
    let status = if state == spec.accept {
        "accept"
    } else {
        "reject"
    };
    println!(
        "halted in state {} ({}) after {} steps",
        STATE_NAMES.get(state.0 as usize).unwrap_or(&"?"),
        status,
        steps
    );
}

#[cfg(test)]
mod tests;
