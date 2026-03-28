use std::time::Instant;

use utmmmmm::compiled::CompiledTuringMachineSpec;
use utmmmmm::infinity::InfiniteTape;
use utmmmmm::optimization_hints::make_my_utm_self_optimization_hints;
use utmmmmm::tm::{RunningTMStatus, RunningTuringMachine};
use utmmmmm::tower::Tower;
use utmmmmm::utm::make_utm_spec;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let max_steps: u64 = args
        .get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1_000_000_000);
    let report_interval: u64 = 100_000_000;

    eprintln!("Running UTM benchmark for {} steps, reporting every {} steps",
        max_steps, report_interval);

    let utm_spec = make_utm_spec();
    let optimization_hints = make_my_utm_self_optimization_hints();
    let background = InfiniteTape::new(&utm_spec, &optimization_hints);
    let compiled = CompiledTuringMachineSpec::compile(&utm_spec).expect("UTM should compile");

    let mut tower = Tower::new(&utm_spec, RunningTuringMachine::new(&compiled));

    let start = Instant::now();
    let mut last_report = Instant::now();

    loop {
        background.extend_compiled(&mut tower.base.tm.tape, tower.base.tm.pos + 1, &compiled);
        if let RunningTMStatus::Accepted | RunningTMStatus::Rejected = tower.step() {
            panic!("infinite machine should never halt");
        }

        let steps = tower.base.total_steps;
        if steps % report_interval == 0 {
            let elapsed = start.elapsed().as_secs_f64();
            let interval_elapsed = last_report.elapsed().as_secs_f64();
            let steps_per_sec = report_interval as f64 / interval_elapsed;

            let level_steps: Vec<String> = tower
                .decoded
                .iter()
                .map(|l| format!("{}", l.total_steps))
                .collect();

            println!(
                "outer={:.0e}  inner=[{}]  elapsed={:.1}s  interval_rate={:.2e} steps/s  head={}  tape_len={}",
                steps as f64,
                level_steps.join(", "),
                elapsed,
                steps_per_sec,
                tower.base.tm.pos,
                tower.base.tm.tape.len(),
            );

            last_report = Instant::now();

            if steps >= max_steps {
                break;
            }
        }
    }

    let elapsed = start.elapsed().as_secs_f64();
    println!("\nDone: {} steps in {:.1}s ({:.2e} steps/s)",
        max_steps, elapsed, max_steps as f64 / elapsed);
}
