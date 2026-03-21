// ════════════════════════════════════════════════════════════════════
// Toy machines for testing the UTM
// ════════════════════════════════════════════════════════════════════

use crate::utm::*;

/// Helper to build a TuringMachineSpec from a simple description.
/// States and symbols are given as string slices; transitions as tuples.
fn build_spec(
    state_names: &[&'static str],
    symbol_names: &[&'static str],
    initial: &str,
    blank: &str,
    accepting: &[&str],
    // Each transition: (state, sym, new_state, new_sym, dir)
    transitions: &[(&str, &str, &str, &str, Dir)],
) -> TuringMachineSpec {
    let n_states = state_names.len();
    let n_symbols = symbol_names.len();

    let find_state = |name: &str| -> u8 {
        state_names
            .iter()
            .position(|&n| n == name)
            .unwrap_or_else(|| panic!("unknown state: {}", name)) as u8
    };
    let find_sym = |name: &str| -> u8 {
        symbol_names
            .iter()
            .position(|&n| n == name)
            .unwrap_or_else(|| panic!("unknown symbol: {}", name)) as u8
    };

    let initial_idx = find_state(initial);
    let blank_idx = find_sym(blank);

    let mut acc = vec![false; n_states];
    for &a in accepting {
        acc[find_state(a) as usize] = true;
    }

    let mut trans = vec![None; 65536];
    let mut ordered = Vec::new();

    // We need to preserve insertion order matching the TypeScript Map iteration order.
    // Group transitions by state in the order states first appear in the transitions list.
    let mut state_order: Vec<u8> = Vec::new();
    let mut state_seen = vec![false; 256];
    for &(st, sy, ns, nsym, dir) in transitions {
        let st_idx = find_state(st);
        let sy_idx = find_sym(sy);
        let ns_idx = find_state(ns);
        let nsym_idx = find_sym(nsym);

        if !state_seen[st_idx as usize] {
            state_seen[st_idx as usize] = true;
            state_order.push(st_idx);
        }

        let key = ((st_idx as usize) << 8) | (sy_idx as usize);
        trans[key] = Some((ns_idx, nsym_idx, dir));
    }

    // Build ordered_rules following state_order and per-state insertion order
    for &(st, sy, ns, nsym, dir) in transitions {
        let st_idx = find_state(st);
        let sy_idx = find_sym(sy);
        let ns_idx = find_state(ns);
        let nsym_idx = find_sym(nsym);
        ordered.push((st_idx, sy_idx, ns_idx, nsym_idx, dir));
    }

    // Find accept state index for hot loop (first accepting state, or 255 if none)
    let accept_idx = accepting.first().map(|&a| find_state(a)).unwrap_or(255);

    TuringMachineSpec {
        n_states,
        n_symbols,
        initial: initial_idx,
        accept: accept_idx,
        blank: blank_idx,
        accepting: acc,
        transitions: trans,
        state_names: state_names.to_vec(),
        symbol_names: symbol_names.to_vec(),
        ordered_rules: ordered,
    }
}

pub fn write1s_forever_spec() -> TuringMachineSpec {
    build_spec(
        &["init"],
        &["_", "1"],
        "init",
        "_",
        &["init"],
        &[
            ("init", "_", "init", "1", Dir::Right),
            ("init", "1", "init", "1", Dir::Right),
        ],
    )
}

pub fn accept_immediately_spec() -> TuringMachineSpec {
    build_spec(
        &["init"],
        &["_"],
        "init",
        "_",
        &["init"],
        &[],
    )
}

pub fn reject_immediately_spec() -> TuringMachineSpec {
    build_spec(
        &["init"],
        &["_"],
        "init",
        "_",
        &[],
        &[],
    )
}

pub fn flip_bits_spec() -> TuringMachineSpec {
    build_spec(
        &["init"],
        &["_", "0", "1"],
        "init",
        "_",
        &["init"],
        &[
            ("init", "0", "init", "1", Dir::Right),
            ("init", "1", "init", "0", Dir::Right),
        ],
    )
}

pub fn check_palindrome_spec() -> TuringMachineSpec {
    let letters = ["a", "b", "c"];

    let mut state_names: Vec<&'static str> = vec!["start", "accept", "seekL"];
    for l in &letters {
        state_names.push(match *l {
            "a" => "seekR_a",
            "b" => "seekR_b",
            "c" => "seekR_c",
            _ => unreachable!(),
        });
    }
    for l in &letters {
        state_names.push(match *l {
            "a" => "check_a",
            "b" => "check_b",
            "c" => "check_c",
            _ => unreachable!(),
        });
    }

    let symbol_names: Vec<&'static str> = vec!["_", "a", "b", "c"];

    let mut transitions: Vec<(&str, &str, &str, &str, Dir)> = Vec::new();

    // start rules
    transitions.push(("start", "_", "accept", "_", Dir::Right));
    for &l in &letters {
        let seek = match l {
            "a" => "seekR_a",
            "b" => "seekR_b",
            "c" => "seekR_c",
            _ => unreachable!(),
        };
        transitions.push(("start", l, seek, "_", Dir::Right));
    }

    // seekR_x rules
    for &l in &letters {
        let seek = match l {
            "a" => "seekR_a",
            "b" => "seekR_b",
            "c" => "seekR_c",
            _ => unreachable!(),
        };
        let check = match l {
            "a" => "check_a",
            "b" => "check_b",
            "c" => "check_c",
            _ => unreachable!(),
        };
        transitions.push((seek, "_", check, "_", Dir::Left));
        for &l2 in &letters {
            transitions.push((seek, l2, seek, l2, Dir::Right));
        }
    }

    // check_x rules
    for &l in &letters {
        let check = match l {
            "a" => "check_a",
            "b" => "check_b",
            "c" => "check_c",
            _ => unreachable!(),
        };
        transitions.push((check, "_", "accept", "_", Dir::Right));
        transitions.push((check, l, "seekL", "_", Dir::Left));
    }

    // seekL rules
    transitions.push(("seekL", "_", "start", "_", Dir::Right));
    for &l in &letters {
        transitions.push(("seekL", l, "seekL", l, Dir::Left));
    }

    // Leak the vecs to get &'static str slices
    let state_names_leaked: &'static [&'static str] = Box::leak(state_names.into_boxed_slice());
    let symbol_names_leaked: &'static [&'static str] = Box::leak(symbol_names.into_boxed_slice());

    build_spec(
        state_names_leaked,
        symbol_names_leaked,
        "start",
        "_",
        &["accept"],
        &transitions,
    )
}

pub fn double_x_spec() -> TuringMachineSpec {
    build_spec(
        &["start", "findX", "goRight", "goBack", "cleanL", "cleanR", "done"],
        &["_", "$", "X", "Y", "Z"],
        "start",
        "_",
        &["done"],
        &[
            ("start", "$", "findX", "$", Dir::Right),
            ("findX", "X", "goRight", "Y", Dir::Right),
            ("findX", "Y", "findX", "Y", Dir::Right),
            ("findX", "Z", "cleanL", "Z", Dir::Left),
            ("findX", "_", "done", "_", Dir::Left),
            ("goRight", "X", "goRight", "X", Dir::Right),
            ("goRight", "Z", "goRight", "Z", Dir::Right),
            ("goRight", "_", "goBack", "Z", Dir::Left),
            ("goBack", "X", "goBack", "X", Dir::Left),
            ("goBack", "Y", "goBack", "Y", Dir::Left),
            ("goBack", "Z", "goBack", "Z", Dir::Left),
            ("goBack", "$", "findX", "$", Dir::Right),
            ("cleanL", "Y", "cleanL", "X", Dir::Left),
            ("cleanL", "$", "cleanR", "$", Dir::Right),
            ("cleanR", "X", "cleanR", "X", Dir::Right),
            ("cleanR", "Z", "cleanR", "X", Dir::Right),
            ("cleanR", "_", "done", "_", Dir::Left),
        ],
    )
}
