mod utm;
mod toy_machines;

use utm::*;

const RADIUS: usize = 30;

/// Format a tape view: 30 symbols on each side of the head, with ^ below.
/// `symbols` maps indices to display strings; `blank` is shown for missing cells.
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
    bot.push_str("    "); // always 4 chars for pointer alignment

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

/// Format the outermost (raw UTM) tape view using u8 symbols directly.
fn raw_tape_view(tape: &[u8], pos: usize, state: u8) -> String {
    let tape_as_usize: Vec<usize> = tape.iter().map(|&s| s as usize).collect();
    let state_name = STATE_NAMES.get(state as usize).unwrap_or(&"?");
    // Convert &[&str] to Vec<&str> for the symbol names
    tape_view(
        &tape_as_usize,
        pos,
        state_name,
        &SYMBOL_NAMES,
        SYM_BLANK as usize,
    )
}

/// Try to decode one level of UTM simulation.
/// Returns None if the tape doesn't have enough structure to decode.
fn try_decode(tape: &[usize], guest: &TuringMachineSpec) -> Option<DecodedGuestState> {
    // Convert usize tape cells to u8 for decode_tape
    let tape_u8: Vec<u8> = tape.iter().map(|&s| s as u8).collect();

    // Need at least 5 hashes for a valid UTM tape
    let hash_count = tape_u8.iter().filter(|&&s| s == SYM_HASH).count();
    if hash_count < 5 {
        return None;
    }

    // Try decoding, catch panics from malformed tapes
    std::panic::catch_unwind(|| decode_tape(&tape_u8, guest)).ok()
}

fn print_tower(tape: &[u8], pos: usize, state: u8, steps: u64) {
    let utm = build_utm_spec();

    eprintln!("═══ {} steps ═══════════════════════════════════════", steps);

    // Level 0: outermost UTM
    eprintln!("Level 0 (outermost UTM):");
    eprintln!("{}", raw_tape_view(tape, pos, state));

    // Level 1: decode the outermost tape to get the simulated machine
    let outer_tape: Vec<usize> = tape.iter().map(|&s| s as usize).collect();
    if let Some(level1) = try_decode(&outer_tape, &utm) {
        let state_name = utm.state_names.get(level1.state).unwrap_or(&"?");
        eprintln!("Level 1 (simulated UTM):");
        eprintln!("{}", tape_view(
            &level1.tape,
            level1.head_pos,
            state_name,
            &SYMBOL_NAMES,
            SYM_BLANK as usize,
        ));

        // Level 2: decode the level-1 tape to get the doubly-simulated machine
        if let Some(level2) = try_decode(&level1.tape, &utm) {
            let state_name = utm.state_names.get(level2.state).unwrap_or(&"?");
            eprintln!("Level 2 (simulated simulated UTM):");
            eprintln!("{}", tape_view(
                &level2.tape,
                level2.head_pos,
                state_name,
                &SYMBOL_NAMES,
                SYM_BLANK as usize,
            ));
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
    // ══════════════════════════════════════════════════════════════
    // Run the infinite UTM tower.
    //
    // The tape is self-referential: it encodes a UTM whose input tape
    // is this very tape. That is, the tape says:
    //   "Run the UTM on a machine whose tape is [this tape]."
    //
    // We materialize the tape lazily: start with an initial chunk,
    // and extend on demand using the recursive background function.
    // ══════════════════════════════════════════════════════════════
    let header = infinite_utm_tape_header();
    let n_sym_bits = num_bits(N_SYMBOLS);
    let cell_size = 1 + n_sym_bits; // each cell = 1 marker + n_sym_bits data bits

    // Materialize the header + initial tape cells
    // Need enough cells so that decoding Level 1 yields a tape long enough
    // to contain the Level 2 header (~28K symbols) + some tape cells.
    let initial_cells = 35000;
    let initial_len = header.len() + initial_cells * cell_size;
    let mut tape: Vec<u8> = Vec::with_capacity(initial_len);
    for i in 0..initial_len {
        tape.push(infinite_utm_tape_background(&header, n_sym_bits, cell_size, i));
    }

    let spec = build_utm_spec();
    let mut state = spec.initial;
    let mut pos: usize = 0;
    let mut steps: u64 = 0;

    // Extend tape rightward by computing more background symbols
    let extend = |tape: &mut Vec<u8>, header: &[u8], n_sym_bits: usize, cell_size: usize| {
        let old_len = tape.len();
        let new_len = old_len + 1024 * cell_size;
        tape.reserve(new_len - old_len);
        for i in old_len..new_len {
            tape.push(infinite_utm_tape_background(header, n_sym_bits, cell_size, i));
        }
    };

    // Print initial state
    print_tower(&tape, pos, state, steps);

    loop {
        if pos >= tape.len() {
            extend(&mut tape, &header, n_sym_bits, cell_size);
        }
        let sym = tape[pos];
        let key = ((state as usize) << 8) | (sym as usize);
        if let Some((ns, nsym, dir)) = spec.transitions[key] {
            state = ns;
            tape[pos] = nsym;
            pos = match dir {
                Dir::Left => pos.saturating_sub(1),
                Dir::Right => pos + 1,
            };
            steps += 1;
            if steps % 1_000_000 == 0 {
                print_tower(&tape, pos, state, steps);
            }
        } else {
            break;
        }
    }

    print_tower(&tape, pos, state, steps);
    let status = if state == spec.accept { "accept" } else { "reject" };
    println!("halted in state {} ({}) after {} steps", STATE_NAMES.get(state as usize).unwrap_or(&"?"), status, steps);
}

#[cfg(test)]
mod tests;
