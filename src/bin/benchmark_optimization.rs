use std::collections::HashMap;
use std::time::Instant;

use utmmmmm::compiled::CompiledTuringMachineSpec;
use utmmmmm::infinity::InfiniteTape;
use utmmmmm::tm::{step, RunningTMStatus, RunningTuringMachine};
use utmmmmm::utm::{
    make_utm_spec, MyUtmSpec, MyUtmSpecOptimizationHints, State, Symbol, TmTransitionStats,
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

    for loop_idx in 0..max_loops {
        eprintln!(
            "=== Loop {} (no prior stats: {}) ===",
            loop_idx,
            loop_idx == 0,
        );

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
    }
}
