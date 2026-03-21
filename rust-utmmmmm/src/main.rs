mod utm;
mod toy_machines;

use utm::*;

fn main() {
    // Run the infinite UTM tower
    let header = infinite_utm_tape_header();
    let n_sym_bits = num_bits(N_SYMBOLS);
    let cell_size = 1 + n_sym_bits;

    let mut tape: Vec<u8> = header.clone();
    // Extend tape with some initial cells from the infinite background
    let initial_cells = 4096;
    for cell_idx in 0..initial_cells {
        let base = header.len() + cell_idx * cell_size;
        while tape.len() < base + cell_size {
            tape.push(0);
        }
        // marker
        tape[base] = if cell_idx == 0 { SYM_CARET } else { SYM_COMMA };
        // bits: encode infinite_utm_tape_background(cell_idx) symbol
        let bg_sym = infinite_utm_tape_background_sym(&header, n_sym_bits, cell_size, cell_idx);
        let bits = to_binary(bg_sym as usize, n_sym_bits);
        for (j, &b) in bits.iter().enumerate() {
            tape[base + 1 + j] = b;
        }
    }

    let spec = build_utm_spec();
    let mut state = spec.initial;
    let mut pos: usize = 0;
    let mut steps: u64 = 0;

    let extend = |tape: &mut Vec<u8>, header: &[u8], n_sym_bits: usize, cell_size: usize| {
        let chunk = 1024;
        let tape_sec_start = header.len();
        let current_cells = (tape.len() - tape_sec_start) / cell_size;
        let new_len = tape_sec_start + (current_cells + chunk) * cell_size;
        let old_len = tape.len();
        tape.resize(new_len, 0);
        for i in old_len..new_len {
            let offset = i - tape_sec_start;
            let cell_idx = offset / cell_size;
            let within = offset % cell_size;
            if within == 0 {
                tape[i] = if cell_idx == 0 { SYM_CARET } else { SYM_COMMA };
            } else {
                let bg_sym = infinite_utm_tape_background_sym(header, n_sym_bits, cell_size, cell_idx);
                let bits = to_binary(bg_sym as usize, n_sym_bits);
                tape[i] = bits[within - 1];
            }
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
