use utmmmmm::compiled::CompiledTuringMachineSpec;
use utmmmmm::infinity::InfiniteTape;
use utmmmmm::optimization_hints::make_my_utm_self_optimization_hints;
use utmmmmm::tm::{Dir, RunningTuringMachine, TuringMachineSpec};
use utmmmmm::utm::make_utm_spec;

fn main() {
    let max_steps: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000_000_000);

    let report_interval: u64 = 100_000_000;

    let optimization_hints = make_my_utm_self_optimization_hints();
    let utm_spec = make_utm_spec();
    let compiled = CompiledTuringMachineSpec::compile(&utm_spec).expect("UTM should compile");

    let mut tm = RunningTuringMachine::new(&compiled);
    let background = InfiniteTape::new(&utm_spec, &optimization_hints);

    let mut total_steps: u64 = 0;
    let mut inner_steps: u64 = 0;
    let mut prev_state = tm.state;

    let start = std::time::Instant::now();
    let mut last_report = start;

    println!(
        "Running UTM benchmark for {} steps, reporting every {}...",
        max_steps, report_interval
    );

    loop {
        if total_steps >= max_steps {
            break;
        }

        if tm.pos >= tm.tape.len() {
            background.extend_compiled(&mut tm.tape, tm.pos + 1, &compiled);
        }

        let sym = tm.tape[tm.pos];
        if let Some((ns, nsym, dir)) = compiled.get_transition(tm.state, sym) {
            tm.state = ns;
            tm.tape[tm.pos] = nsym;
            tm.pos = match dir {
                Dir::Left => tm.pos.saturating_sub(1),
                Dir::Right => tm.pos + 1,
            };
            total_steps += 1;

            if compiled.is_tick_boundary(prev_state, tm.state) {
                inner_steps += 1;
            }
            prev_state = tm.state;

            if total_steps % report_interval == 0 {
                let now = std::time::Instant::now();
                let elapsed = now.duration_since(start).as_secs_f64();
                let interval_elapsed = now.duration_since(last_report).as_secs_f64();
                let steps_per_sec = report_interval as f64 / interval_elapsed;
                last_report = now;

                println!(
                    "{:>6.0}M outer steps | {:>8} inner steps | ratio {:>10.1} | {:.1}M steps/s | {:.1}s elapsed",
                    total_steps as f64 / 1e6,
                    inner_steps,
                    if inner_steps > 0 { total_steps as f64 / inner_steps as f64 } else { f64::INFINITY },
                    steps_per_sec / 1e6,
                    elapsed,
                );
            }
        } else {
            let orig_state = compiled.original_states[tm.state.0 as usize];
            let orig_sym = compiled.original_symbols[sym.0 as usize];
            println!(
                "UTM halted after {} steps ({} inner steps) in state {:?} reading {:?} at pos {}",
                total_steps, inner_steps, orig_state, orig_sym, tm.pos
            );
            // Print surrounding tape context
            let start = tm.pos.saturating_sub(30);
            let end = (tm.pos + 30).min(tm.tape.len());
            for i in start..end {
                let s = compiled.original_symbols[tm.tape[i].0 as usize];
                if i == tm.pos {
                    print!("[{:?}]", s);
                } else {
                    print!("{:?} ", s);
                }
            }
            println!();
            break;
        }
    }

    let elapsed = start.elapsed().as_secs_f64();
    println!(
        "\nDone: {} outer steps, {} inner steps, ratio {:.1}, {:.1}s total ({:.1}M steps/s)",
        total_steps,
        inner_steps,
        if inner_steps > 0 {
            total_steps as f64 / inner_steps as f64
        } else {
            f64::INFINITY
        },
        elapsed,
        total_steps as f64 / elapsed / 1e6,
    );
}
