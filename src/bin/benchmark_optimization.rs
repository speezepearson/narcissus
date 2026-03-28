use std::collections::HashMap;
use std::time::Instant;

use utmmmmm::compiled::CompiledTuringMachineSpec;
use utmmmmm::infinity::InfiniteTape;
use utmmmmm::tm::{step, RunningTMStatus, RunningTuringMachine, TuringMachineSpec};
use utmmmmm::utm::{
    make_utm_spec, num_bits, MyUtmSpec, MyUtmSpecOptimizationHints, State, Symbol,
    TmTransitionStats,
};

fn run_loop(
    spec: &MyUtmSpec,
    hints: &MyUtmSpecOptimizationHints<MyUtmSpec>,
    max_steps: u64,
) -> (u64, u64, TmTransitionStats<MyUtmSpec>) {
    let compiled = CompiledTuringMachineSpec::compile(spec).expect("UTM should compile");
    let background = InfiniteTape::new(spec, hints);

    let compiled_init = compiled.compile_state(State::Init);

    let mut tm = RunningTuringMachine::new(&compiled);
    let mut prev_state = tm.state;
    let mut inner_steps: u64 = 0;
    let mut stats: HashMap<(State, Symbol), usize> = HashMap::new();

    for outer_step in 0..max_steps {
        background.extend_compiled(&mut tm.tape, tm.pos + 1, &compiled);

        // Track transition: decompile current (state, symbol) before stepping
        let orig_state = compiled.decompile_state(tm.state);
        let orig_symbol = compiled.decompile_symbol(tm.tape[tm.pos]);
        *stats.entry((orig_state, orig_symbol)).or_insert(0) += 1;

        if let RunningTMStatus::Accepted | RunningTMStatus::Rejected = step(&mut tm) {
            panic!("infinite machine should never halt");
        }

        // Detect when the simulated UTM enters Init (= one inner step completed)
        if tm.state == compiled_init && prev_state != compiled_init {
            inner_steps += 1;
        }
        prev_state = tm.state;

        if (outer_step + 1) % 100_000_000 == 0 {
            eprintln!(
                "  ... {:.0}M outer steps, {} inner steps so far",
                (outer_step + 1) as f64 / 1_000_000.0,
                inner_steps,
            );
        }
    }

    (inner_steps, max_steps, TmTransitionStats(stats))
}

fn common_prefix_len(a: usize, b: usize, width: usize) -> usize {
    if width == 0 {
        return 0;
    }
    let xor = a ^ b;
    if xor == 0 {
        return width;
    }
    let first_diff = (width - 1) - (xor.ilog2() as usize);
    first_diff
}

/// Compute prefix confusion cost for state or symbol encodings given pair weights.
/// Returns sum over all pairs (i,j) of pair_weight[i][j] * common_prefix_len(enc[i], enc[j], width).
fn prefix_confusion_cost(
    pair_weight: &[Vec<u64>],
    encodings: &[usize],
    width: usize,
) -> u64 {
    let n = encodings.len();
    let mut total = 0u64;
    for i in 0..n {
        for j in i + 1..n {
            let w = pair_weight[i][j]; // already symmetrized
            if w == 0 {
                continue;
            }
            total += w * common_prefix_len(encodings[i], encodings[j], width) as u64;
        }
    }
    total
}

/// Compute state pair weights from transition stats and rule order.
fn compute_state_pair_weights(
    stats: &TmTransitionStats<MyUtmSpec>,
    spec: &MyUtmSpec,
) -> (Vec<State>, Vec<Vec<u64>>) {
    let states: Vec<State> = spec.iter_states().collect();
    let n = states.len();
    let state_idx: HashMap<State, usize> =
        states.iter().enumerate().map(|(i, &s)| (s, i)).collect();
    let rule_order = stats.get_optimal_rule_order(spec);

    let mut pair_weight = vec![vec![0u64; n]; n];
    for (pos, &(s, sym)) in rule_order.iter().enumerate() {
        let count = *stats.0.get(&(s, sym)).unwrap_or(&0) as u64;
        if count == 0 {
            continue;
        }
        let si = state_idx[&s];
        for &(s2, _) in &rule_order[pos + 1..] {
            if s2 != s {
                let s2i = state_idx[&s2];
                pair_weight[si][s2i] += count;
            }
        }
    }
    for i in 0..n {
        for j in i + 1..n {
            let total = pair_weight[i][j] + pair_weight[j][i];
            pair_weight[i][j] = total;
            pair_weight[j][i] = total;
        }
    }
    (states, pair_weight)
}

/// Compute symbol pair weights from transition stats and rule order.
fn compute_symbol_pair_weights(
    stats: &TmTransitionStats<MyUtmSpec>,
    spec: &MyUtmSpec,
) -> (Vec<Symbol>, Vec<Vec<u64>>) {
    let symbols: Vec<Symbol> = spec.iter_symbols().collect();
    let n = symbols.len();
    let sym_idx: HashMap<Symbol, usize> =
        symbols.iter().enumerate().map(|(i, &s)| (s, i)).collect();
    let rule_order = stats.get_optimal_rule_order(spec);

    let mut pair_weight = vec![vec![0u64; n]; n];
    for (pos, &(s, sym)) in rule_order.iter().enumerate() {
        let count = *stats.0.get(&(s, sym)).unwrap_or(&0) as u64;
        if count == 0 {
            continue;
        }
        let si = sym_idx[&sym];
        for &(s2, sym2) in &rule_order[pos + 1..] {
            if s2 == s && sym2 != sym {
                let s2i = sym_idx[&sym2];
                pair_weight[si][s2i] += count;
            }
        }
    }
    for i in 0..n {
        for j in i + 1..n {
            let total = pair_weight[i][j] + pair_weight[j][i];
            pair_weight[i][j] = total;
            pair_weight[j][i] = total;
        }
    }
    (symbols, pair_weight)
}

fn print_encoding_diagnostics(
    stats: &TmTransitionStats<MyUtmSpec>,
    hints: &MyUtmSpecOptimizationHints<MyUtmSpec>,
    spec: &MyUtmSpec,
) {
    let (states, st_pw) = compute_state_pair_weights(stats, spec);
    let n_st = states.len();
    let st_width = num_bits(n_st);

    // Optimized state encoding (what we're about to use)
    let opt_st_enc: Vec<usize> = states.iter().map(|s| hints.state_encodings[s]).collect();
    // Default sequential encoding for comparison
    let default_st_enc: Vec<usize> = (0..n_st).collect();

    let opt_st_cost = prefix_confusion_cost(&st_pw, &opt_st_enc, st_width);
    let default_st_cost = prefix_confusion_cost(&st_pw, &default_st_enc, st_width);

    let (symbols, sym_pw) = compute_symbol_pair_weights(stats, spec);
    let n_sym = symbols.len();
    let sym_width = num_bits(n_sym);

    let opt_sym_enc: Vec<usize> = symbols.iter().map(|s| hints.symbol_encodings[s]).collect();
    let default_sym_enc: Vec<usize> = (0..n_sym).collect();

    let opt_sym_cost = prefix_confusion_cost(&sym_pw, &opt_sym_enc, sym_width);
    let default_sym_cost = prefix_confusion_cost(&sym_pw, &default_sym_enc, sym_width);

    eprintln!(
        "  state prefix cost: optimized={} default={} (ratio {:.3})",
        opt_st_cost,
        default_st_cost,
        opt_st_cost as f64 / default_st_cost.max(1) as f64,
    );
    eprintln!(
        "  symbol prefix cost: optimized={} default={} (ratio {:.3})",
        opt_sym_cost,
        default_sym_cost,
        opt_sym_cost as f64 / default_sym_cost.max(1) as f64,
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let max_steps: u64 = args
        .get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000_000_000);
    let max_loops: usize = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);

    let spec = make_utm_spec();

    let mut hints = MyUtmSpecOptimizationHints::guess(&spec);
    let mut prev_stats: Option<TmTransitionStats<MyUtmSpec>> = None;

    for loop_idx in 0..max_loops {
        eprintln!(
            "=== Loop {} (no prior stats: {}) ===",
            loop_idx,
            loop_idx == 0,
        );

        if let Some(ref stats) = prev_stats {
            print_encoding_diagnostics(stats, &hints, &spec);
        }

        let start = Instant::now();
        let (inner_steps, outer_steps, stats) = run_loop(&spec, &hints, max_steps);
        let elapsed = start.elapsed();

        let ratio = if inner_steps > 0 {
            outer_steps as f64 / inner_steps as f64
        } else {
            f64::INFINITY
        };

        println!(
            "loop={} inner_steps={} outer_steps={} ratio={:.1} elapsed={:.1}s",
            loop_idx,
            inner_steps,
            outer_steps,
            ratio,
            elapsed.as_secs_f64(),
        );

        // Build new hints from the transition stats we just collected
        hints = stats.make_optimization_hints(&spec);
        prev_stats = Some(stats);
    }
}
