use std::collections::HashMap;

use utmmmmm::tm::TuringMachineSpec;
use utmmmmm::transition_tallies::TRANSITION_TALLIES;
use utmmmmm::utm::{make_utm_spec, num_bits, MyUtmSpecOptimizationHints, State, Symbol, TmTransitionStats};

struct SimResult {
    avg_rules: f64,
    avg_state_bits: f64,
    avg_sym_bits: f64,
}

fn simulate(
    hints: &MyUtmSpecOptimizationHints<utmmmmm::utm::MyUtmSpec>,
    all_rules: &HashMap<(State, Symbol), ()>,
    iterations: u64,
) -> SimResult {
    let n_state_bits = num_bits(hints.state_encodings.len());
    let n_sym_bits = num_bits(hints.symbol_encodings.len());

    let rule_encodings: Vec<(usize, usize)> = hints
        .rule_order
        .iter()
        .map(|&(st, sym)| (hints.state_encodings[&st], hints.symbol_encodings[&sym]))
        .collect();
    let n_rules = rule_encodings.len();

    // Build weighted sampling table
    let mut samples: Vec<(usize, usize)> = Vec::new();
    let mut weights: Vec<usize> = Vec::new();
    for &((st, sym), count) in TRANSITION_TALLIES {
        if count > 0 && all_rules.contains_key(&(st, sym)) {
            samples.push((hints.state_encodings[&st], hints.symbol_encodings[&sym]));
            weights.push(count);
        }
    }
    let total_weight: usize = weights.iter().sum();
    let mut cumulative: Vec<usize> = Vec::with_capacity(weights.len());
    let mut acc = 0usize;
    for &w in &weights {
        acc += w;
        cumulative.push(acc);
    }

    let mut total_rule_checks = 0u64;
    let mut total_state_bits = 0u64;
    let mut total_sym_bits = 0u64;
    let mut rng_state: u64 = 12345678901;

    for _ in 0..iterations {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let r = (rng_state >> 33) as usize % total_weight;
        let idx = cumulative.partition_point(|&c| c <= r);
        let (target_st, target_sym) = samples[idx];

        for i in (0..n_rules).rev() {
            let (rule_st, rule_sym) = rule_encodings[i];
            total_rule_checks += 1;

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
                break;
            }
        }
    }

    SimResult {
        avg_rules: total_rule_checks as f64 / iterations as f64,
        avg_state_bits: total_state_bits as f64 / iterations as f64,
        avg_sym_bits: total_sym_bits as f64 / iterations as f64,
    }
}

fn main() {
    let spec = make_utm_spec();
    let stats = TmTransitionStats(TRANSITION_TALLIES.iter().copied().collect::<HashMap<_, _>>());

    let all_rules: HashMap<_, _> = spec
        .iter_rules()
        .map(|(st, sym, _, _, _)| ((st, sym), ()))
        .collect();

    let iterations = 1_000_000u64;

    // Optimized encoding (greedy bisection)
    let optimized_hints = stats.make_optimization_hints(&spec);
    let opt = simulate(&optimized_hints, &all_rules, iterations);

    // Default encoding (sequential indices, same rule order)
    let default_hints = MyUtmSpecOptimizationHints {
        rule_order: optimized_hints.rule_order.clone(),
        state_encodings: spec.iter_states().enumerate().map(|(i, s)| (s, i)).collect(),
        symbol_encodings: spec.iter_symbols().enumerate().map(|(i, s)| (s, i)).collect(),
    };
    let def = simulate(&default_hints, &all_rules, iterations);

    // No optimization at all (default encoding + default rule order)
    let no_opt_hints = MyUtmSpecOptimizationHints::guess(&spec);
    let no_opt = simulate(&no_opt_hints, &all_rules, iterations);

    println!("{:<30} {:>10} {:>10} {:>10}", "", "no-opt", "default-enc", "optimized");
    println!("{:<30} {:>10.2} {:>10.2} {:>10.2}", "avg rules checked", no_opt.avg_rules, def.avg_rules, opt.avg_rules);
    println!("{:<30} {:>10.2} {:>10.2} {:>10.2}", "avg state-bit comparisons", no_opt.avg_state_bits, def.avg_state_bits, opt.avg_state_bits);
    println!("{:<30} {:>10.2} {:>10.2} {:>10.2}", "avg sym-bit comparisons", no_opt.avg_sym_bits, def.avg_sym_bits, opt.avg_sym_bits);
}
