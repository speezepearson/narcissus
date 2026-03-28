use std::collections::HashMap;

use utmmmmm::tm::TuringMachineSpec;
use utmmmmm::transition_tallies::TRANSITION_TALLIES;
use utmmmmm::utm::{make_utm_spec, num_bits, TmTransitionStats};

fn main() {
    let spec = make_utm_spec();
    let stats = TmTransitionStats(TRANSITION_TALLIES.iter().copied().collect::<HashMap<_, _>>());
    let hints = stats.make_optimization_hints(&spec);

    let n_state_bits = num_bits(hints.state_encodings.len());
    let n_sym_bits = num_bits(hints.symbol_encodings.len());

    // Build encoded rule list in rule_order, storing (state_enc, sym_enc) for each rule
    let all_rules: HashMap<_, _> = spec
        .iter_rules()
        .map(|(st, sym, _, _, _)| ((st, sym), ()))
        .collect();
    let rule_encodings: Vec<(usize, usize)> = hints
        .rule_order
        .iter()
        .map(|&(st, sym)| (hints.state_encodings[&st], hints.symbol_encodings[&sym]))
        .collect();
    let n_rules = rule_encodings.len();

    // Build weighted sampling table from transition tallies
    let mut samples: Vec<(usize, usize)> = Vec::new(); // (state_enc, sym_enc)
    let mut weights: Vec<usize> = Vec::new();
    for &((st, sym), count) in TRANSITION_TALLIES {
        if count > 0 && all_rules.contains_key(&(st, sym)) {
            samples.push((hints.state_encodings[&st], hints.symbol_encodings[&sym]));
            weights.push(count);
        }
    }
    let total_weight: usize = weights.iter().sum();
    // Build cumulative distribution
    let mut cumulative: Vec<usize> = Vec::with_capacity(weights.len());
    let mut acc = 0usize;
    for &w in &weights {
        acc += w;
        cumulative.push(acc);
    }

    let iterations = 1_000_000u64;
    let mut total_rule_checks = 0u64;
    let mut total_state_bits = 0u64;
    let mut total_sym_bits = 0u64;

    // Simple LCG for fast deterministic pseudo-random
    let mut rng_state: u64 = 12345678901;

    for _ in 0..iterations {
        // Sample a (state, symbol) pair weighted by frequency
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let r = (rng_state >> 33) as usize % total_weight;
        let idx = cumulative.partition_point(|&c| c <= r);
        let (target_st, target_sym) = samples[idx];

        // Scan rules last-to-first (UTM scan order)
        for i in (0..n_rules).rev() {
            let (rule_st, rule_sym) = rule_encodings[i];
            total_rule_checks += 1;

            // Compare state bits MSB-first
            let mut state_bits_checked = 0u64;
            let mut state_matched = true;
            for bit in (0..n_state_bits).rev() {
                state_bits_checked += 1;
                if (target_st >> bit) & 1 != (rule_st >> bit) & 1 {
                    state_matched = false;
                    break;
                }
            }
            total_state_bits += state_bits_checked;

            if !state_matched {
                continue;
            }

            // State matched — compare symbol bits MSB-first
            let mut sym_bits_checked = 0u64;
            let mut sym_matched = true;
            for bit in (0..n_sym_bits).rev() {
                sym_bits_checked += 1;
                if (target_sym >> bit) & 1 != (rule_sym >> bit) & 1 {
                    sym_matched = false;
                    break;
                }
            }
            total_sym_bits += sym_bits_checked;

            if sym_matched {
                // Found the matching rule
                break;
            }
        }
    }

    println!(
        "avg rules checked:     {:.2}",
        total_rule_checks as f64 / iterations as f64
    );
    println!(
        "avg state-bit comparisons: {:.2}",
        total_state_bits as f64 / iterations as f64
    );
    println!(
        "avg sym-bit comparisons:   {:.2}",
        total_sym_bits as f64 / iterations as f64
    );
}
