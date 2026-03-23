#[allow(dead_code)]
mod compiled {
    include!("../compiled.rs");
}
#[allow(dead_code)]
mod infinity {
    include!("../infinity.rs");
}
mod optimization_hints {
    include!("../optimization_hints.rs");
}
#[allow(dead_code)]
mod tm {
    include!("../tm.rs");
}
#[allow(dead_code)]
mod toy_machines {
    include!("../toy_machines.rs");
}
#[allow(dead_code)]
mod utm {
    include!("../utm.rs");
}

use compiled::{CompiledTapeExtender, CompiledTuringMachineSpec};
use infinity::InfiniteTapeExtender;
use tm::{RunningTuringMachine, TuringMachineSpec};
use utm::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let max_steps: u64 = match args.iter().position(|a| a == "--steps") {
        Some(i) => args[i + 1].parse().expect("--steps requires a number"),
        None => {
            eprintln!("Usage: tally --steps N");
            std::process::exit(1);
        }
    };

    let utm = &*UTM_SPEC;
    let compiled = CompiledTuringMachineSpec::compile(utm).expect("UTM should compile");

    let mut tm = RunningTuringMachine::new(&compiled);
    let mut extender = CompiledTapeExtender::new(&compiled, Box::new(InfiniteTapeExtender));
    extender.extend(&mut tm.tape, 1);

    // Tally: count (state, symbol) pairs at each step
    // Use the compiled indices directly for speed, then map back at the end.
    let mut counts = vec![0u64; 1 << 16]; // [state.0 << 8 | symbol.0]

    for step in 0..max_steps {
        if tm.pos >= tm.tape.len() {
            extender.extend(&mut tm.tape, tm.pos + 1);
        }
        let sym = tm.tape[tm.pos];
        counts[((tm.state.0 as usize) << 8) | (sym.0 as usize)] += 1;

        if let Some((ns, nsym, dir)) = compiled.get_transition(tm.state, sym) {
            tm.state = ns;
            tm.tape[tm.pos] = nsym;
            tm.pos = match dir {
                tm::Dir::Left => tm.pos.saturating_sub(1),
                tm::Dir::Right => tm.pos + 1,
            };
        } else {
            eprintln!("Halted after {} steps", step);
            break;
        }

        if step > 0 && step % 10_000_000 == 0 {
            eprintln!("  {} / {} steps...", step, max_steps);
        }
    }

    // Collect non-zero entries, map back to original State/Symbol
    let mut tallies: Vec<(State, Symbol, u64)> = Vec::new();
    for (idx, &count) in counts.iter().enumerate() {
        if count == 0 {
            continue;
        }
        let state_idx = idx >> 8;
        let sym_idx = idx & 0xFF;
        let state = compiled.original_states[state_idx];
        let sym = compiled.original_symbols[sym_idx];
        tallies.push((state, sym, count));
    }

    // Sort by count descending
    tallies.sort_by(|a, b| b.2.cmp(&a.2));

    println!("# State/Symbol tally after {} steps", max_steps);
    println!(
        "# {:>50} {:>6} {:>12} {:>8}",
        "state", "symbol", "count", "pct"
    );
    let total: u64 = tallies.iter().map(|t| t.2).sum();
    for (state, sym, count) in &tallies {
        let pct = *count as f64 / total as f64 * 100.0;
        println!("  {:>50?} {:>6} {:>12} {:>7.3}%", state, sym, count, pct);
    }
    println!("# Total: {}", total);
}
