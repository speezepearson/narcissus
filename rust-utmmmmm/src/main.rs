mod utm;
mod toy_machines;

use utm::*;

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
    let initial_cells = 4096;
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
            if steps % 10_000_000 == 0 {
                eprintln!("steps: {}M, pos: {}, state: {}", steps / 1_000_000, pos, STATE_NAMES.get(state as usize).unwrap_or(&"?"));
            }
        } else {
            break;
        }
    }
    let status = if state == spec.accept { "accept" } else { "reject" };
    println!("halted in state {} ({}) after {} steps", STATE_NAMES.get(state as usize).unwrap_or(&"?"), status, steps);
}

#[cfg(test)]
mod tests;
