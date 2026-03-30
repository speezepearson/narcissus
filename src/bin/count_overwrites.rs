use utmmmmm::compiled::CompiledTuringMachineSpec;
use utmmmmm::infinity::InfiniteTape;
use utmmmmm::optimization_hints::make_my_utm_self_optimization_hints;
use utmmmmm::tm::{Dir, RunningTuringMachine, TuringMachineSpec};
use utmmmmm::utm::make_utm_spec;

fn main() {
    let utm_spec = make_utm_spec();
    let encoder = make_my_utm_self_optimization_hints(&utm_spec);
    let compiled = CompiledTuringMachineSpec::compile(&utm_spec).expect("UTM should compile");

    let mut tm = RunningTuringMachine::new(&compiled);
    let background = InfiniteTape::new(&encoder);

    // Snapshot the original tape for dirty-cell counting
    let mut original_tape = tm.tape.clone();

    let mut total_steps: u64 = 0;
    let mut overwrites: u64 = 0;

    loop {
        if tm.pos >= tm.tape.len() {
            background.extend_compiled(&mut tm.tape, tm.pos + 1, &compiled);
        }

        let sym = tm.tape[tm.pos];
        if let Some((ns, nsym, dir)) = compiled.get_transition(tm.state, sym) {
            if sym != nsym {
                overwrites += 1;
            }
            tm.state = ns;
            tm.tape[tm.pos] = nsym;
            tm.pos = match dir {
                Dir::Left => tm.pos.saturating_sub(1),
                Dir::Right => tm.pos + 1,
            };
            total_steps += 1;

            if total_steps % 1_000_000_000 == 0 {
                background.extend_compiled(&mut original_tape, tm.tape.len(), &compiled);
                let dirty: u64 = tm
                    .tape
                    .iter()
                    .zip(original_tape.iter())
                    .map(|(a, b)| if a != b { 1u64 } else { 0 })
                    .sum();
                println!(
                    "{:.0}B steps, {} overwrites ({:.2}%), {} dirty cells out of {} ({:.2}%)",
                    total_steps as f64 / 1e9,
                    overwrites,
                    overwrites as f64 / total_steps as f64 * 100.0,
                    dirty,
                    tm.tape.len(),
                    dirty as f64 / tm.tape.len() as f64 * 100.0,
                );
            }
        } else {
            let dirty: u64 = tm
                .tape
                .iter()
                .zip(original_tape.iter())
                .map(|(a, b)| if a != b { 1u64 } else { 0 })
                .sum();
            println!(
                "Halted after {} steps, {} overwrites ({:.2}%), {} dirty cells out of {} ({:.2}%)",
                total_steps,
                overwrites,
                overwrites as f64 / total_steps as f64 * 100.0,
                dirty,
                tm.tape.len(),
                dirty as f64 / tm.tape.len() as f64 * 100.0,
            );
            return;
        }
    }
}
