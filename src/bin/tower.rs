use std::cmp::max;
use std::fmt::Write;
use std::io::Write as IoWrite;
use utmmmmm::compiled::{CState, CSymbol, CompiledTapeExtender, CompiledTuringMachineSpec};
use utmmmmm::infinity::{header_len, InfiniteTapeExtender};
use utmmmmm::tm::{
    Dir, RunningTuringMachine, SimpleTuringMachineSpec, TapeExtender, TuringMachineSpec,
};
use utmmmmm::utm::{MyUtmEncodingScheme, State, Symbol, UtmEncodingScheme, UTM_SPEC};

type UtmTm<'a> = RunningTuringMachine<'a, SimpleTuringMachineSpec<State, Symbol>>;

struct TowerLevel<'a> {
    machine: UtmTm<'a>,
    max_head_pos: usize,
    prev_state: Option<State>,
}

impl<'a> TowerLevel<'a> {
    fn new(machine: UtmTm<'a>) -> Self {
        let max_head_pos = machine.pos;
        Self {
            machine,
            max_head_pos,
            prev_state: None,
        }
    }

    fn update_machine(&mut self, machine: UtmTm<'a>) {
        self.machine = machine;
        if self.machine.pos > self.max_head_pos {
            self.max_head_pos = self.machine.pos;
        }
    }

    fn snapshot_state(&mut self) {
        self.prev_state = Some(self.machine.state);
    }

    fn entered_init(&self) -> bool {
        match self.prev_state {
            Some(prev) => self.machine.state == State::Init && prev != State::Init,
            None => false,
        }
    }
}

/// Format a tape slice with the head cell highlighted in light red.
/// Shows tape[0..end].
fn tape_view_range(tm: &UtmTm, end: usize) -> String {
    let mut out = String::from("    ");
    let blank = tm.spec.blank();

    for i in 0..end {
        let sym = if i < tm.tape.len() {
            tm.tape[i]
        } else {
            blank
        };
        let green = matches!(sym, Symbol::Star | Symbol::X | Symbol::Y | Symbol::Caret | Symbol::Gt);
        if i == tm.pos {
            write!(out, "\x1b[101m{}\x1b[0m", sym).unwrap();
        } else if green {
            write!(out, "\x1b[102m{}\x1b[0m", sym).unwrap();
        } else {
            write!(out, "{}", sym).unwrap();
        }
    }

    if end < tm.tape.len() {
        out.push_str(" ...");
    }
    write!(out, " (state={:?}, pos={})", tm.state, tm.pos).unwrap();
    out
}


/// Decode the next level from a parent machine, extending the tape as needed.
/// Returns None if decoding fails (tape too short, etc.)
fn decode_next_level<'a>(
    utm: &'a SimpleTuringMachineSpec<State, Symbol>,
    parent: &mut UtmTm<'a>,
    extender: &mut InfiniteTapeExtender,
) -> Option<UtmTm<'a>> {
    let min_len = max(header_len(), parent.pos + 100);
    extender.extend(&mut parent.tape, min_len);
    MyUtmEncodingScheme::decode(utm, &parent.tape).ok()
}

/// Build the tower by decoding each level from the previous.
/// tower[0] = decompiled L0, tower[1] = decode(tower[0]), etc.
/// Re-decodes level i+1 when level i entered Init.
/// Grows the tower by at most one new level per call.
fn update_tower<'a>(
    utm: &'a SimpleTuringMachineSpec<State, Symbol>,
    tower: &mut Vec<TowerLevel<'a>>,
    extender: &mut InfiniteTapeExtender,
) {
    let mut level = 0;
    loop {
        if level > 0 && !tower[level].entered_init() {
            break;
        }

        if let Some(next) = decode_next_level(utm, &mut tower[level].machine, extender) {
            if level + 1 < tower.len() {
                tower[level + 1].update_machine(next);
            } else {
                tower.push(TowerLevel::new(next));
                break;
            }
            level += 1;
        } else {
            break;
        }
    }

    for tl in tower.iter_mut() {
        tl.snapshot_state();
    }
}

fn format_tower<'a>(
    tower: &mut [TowerLevel<'a>],
    total_steps: u64,
    utm: &'a SimpleTuringMachineSpec<State, Symbol>,
    extender: &mut InfiniteTapeExtender,
) -> String {
    let mut buf = String::new();
    writeln!(
        buf,
        "═══ {} steps ═══════════════════════════════════════",
        total_steps
    )
    .unwrap();

    for (i, tl) in tower.iter().enumerate() {
        writeln!(buf, "Level {} ({} symbols):", i, tl.machine.tape.len()).unwrap();
        writeln!(buf, "{}", tape_view_range(&tl.machine, tl.max_head_pos + 10)).unwrap();
    }

    // Decode and print one more level beyond the tower.
    let last = tower.last_mut().unwrap();
    if let Some(extra) = decode_next_level(utm, &mut last.machine, extender) {
        let i = tower.len();
        writeln!(buf, "Level {} ({} symbols):", i, extra.tape.len()).unwrap();
        writeln!(buf, "{}", tape_view_range(&extra, extra.pos + 10)).unwrap();
        assert!(
            extra.pos == 0,
            "Extra tower level should have head at pos 0, but has pos {}",
            extra.pos
        );
    }

    buf
}

// ════════════════════════════════════════════════════════════════════
// Savepoint: binary format for compiled TM state
// ════════════════════════════════════════════════════════════════════
// u64 total_steps | u64 guest_steps | u8 state | u64 pos | u64 tape_len | [u8] tape

fn save_savepoint(
    path: &str,
    total_steps: u64,
    guest_steps: u64,
    tm: &RunningTuringMachine<CompiledTuringMachineSpec<SimpleTuringMachineSpec<State, Symbol>>>,
) {
    let tmp = format!("{}.tmp", path);
    let mut f = std::io::BufWriter::new(std::fs::File::create(&tmp).expect("create savepoint"));
    f.write_all(&total_steps.to_le_bytes()).unwrap();
    f.write_all(&guest_steps.to_le_bytes()).unwrap();
    f.write_all(&[tm.state.0]).unwrap();
    f.write_all(&(tm.pos as u64).to_le_bytes()).unwrap();
    f.write_all(&(tm.tape.len() as u64).to_le_bytes()).unwrap();
    let tape_bytes: Vec<u8> = tm.tape.iter().map(|s| s.0).collect();
    f.write_all(&tape_bytes).unwrap();
    drop(f);
    std::fs::rename(&tmp, path).expect("rename savepoint");
    eprintln!("Saved savepoint at step {} to {}", total_steps, path);
}

fn load_savepoint(path: &str) -> Option<(u64, u64, CState, usize, Vec<CSymbol>)> {
    let data = std::fs::read(path).ok()?;
    if data.len() < 25 {
        return None;
    }
    let total_steps = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let guest_steps = u64::from_le_bytes(data[8..16].try_into().unwrap());
    let state = CState(data[16]);
    let pos = u64::from_le_bytes(data[17..25].try_into().unwrap()) as usize;
    let tape_len = u64::from_le_bytes(data[25..33].try_into().unwrap()) as usize;
    if data.len() < 33 + tape_len {
        return None;
    }
    let tape: Vec<CSymbol> = data[33..33 + tape_len]
        .iter()
        .map(|&b| CSymbol(b))
        .collect();
    Some((total_steps, guest_steps, state, pos, tape))
}

// ════════════════════════════════════════════════════════════════════

fn get_flag(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .map(|i| args[i + 1].clone())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let savepoint_path = get_flag(&args, "--savepoint");

    let utm = &*UTM_SPEC;
    let compiled = CompiledTuringMachineSpec::compile(utm).expect("UTM should compile");

    // Find the CState corresponding to State::Init
    let init_cstate = compiled
        .original_states
        .iter()
        .position(|&s| s == State::Init)
        .map(|i| CState(i as u8))
        .expect("Init state should exist");

    let mut tm = RunningTuringMachine::new(&compiled);
    let mut extender = CompiledTapeExtender::new(&compiled, Box::new(InfiniteTapeExtender));
    extender.extend(&mut tm.tape, 1);

    let mut total_steps: u64 = 0;
    let mut guest_steps: u64 = 0;

    // Load savepoint if it exists
    if let Some(ref sp_path) = savepoint_path {
        if let Some((sp_steps, sp_guest, sp_state, sp_pos, sp_tape)) = load_savepoint(sp_path) {
            total_steps = sp_steps;
            guest_steps = sp_guest;
            tm.state = sp_state;
            tm.pos = sp_pos;
            tm.tape = sp_tape;
            // Sync the extender's shadow tape to the loaded tape length
            let tape_len = tm.tape.len();
            extender.extend(&mut tm.tape, tape_len);
            eprintln!(
                "Loaded savepoint from {}: step {}, {} guest steps, tape len {}",
                sp_path,
                total_steps,
                guest_steps,
                tm.tape.len()
            );
        }
    }

    let mut inf_extender = InfiniteTapeExtender;

    // Initialize tower
    let mut tower: Vec<TowerLevel> = vec![TowerLevel::new(compiled.decompile(&tm))];
    if tm.state == init_cstate {
        update_tower(utm, &mut tower, &mut inf_extender);
    }
    eprint!("{}", format_tower(&mut tower, total_steps, utm, &mut inf_extender));

    let print_interval = std::time::Duration::from_millis(100);
    let mut last_print = std::time::Instant::now();
    let start_time = std::time::Instant::now();
    let mut prev_cstate = tm.state;
    let mut last_savepoint_step = total_steps;

    loop {
        // Extend tape if needed
        if tm.pos >= tm.tape.len() {
            extender.extend(&mut tm.tape, tm.pos + 1);
        }

        // Step
        let sym = tm.tape[tm.pos];
        if let Some((ns, nsym, dir)) = compiled.get_transition(tm.state, sym) {
            tm.state = ns;
            tm.tape[tm.pos] = nsym;
            tm.pos = match dir {
                Dir::Left => tm.pos.saturating_sub(1),
                Dir::Right => tm.pos + 1,
            };
            total_steps += 1;
        } else {
            // Halted
            tower[0].update_machine(compiled.decompile(&tm));
            update_tower(utm, &mut tower, &mut inf_extender);
            eprint!("{}", format_tower(&mut tower, total_steps, utm, &mut inf_extender));
            let status = if compiled.is_accepting(tm.state) {
                "accept"
            } else {
                "reject"
            };
            println!(
                "halted ({}) in state {:?} after {} UTM steps ({} guest steps)",
                status, tower[0].machine.state, total_steps, guest_steps
            );
            if let Some(ref sp_path) = savepoint_path {
                save_savepoint(sp_path, total_steps, guest_steps, &tm);
            }
            return;
        }

        // Detect Init entry
        if tm.state != prev_cstate {
            if tm.state == init_cstate {
                guest_steps += 1;
                tower[0].update_machine(compiled.decompile(&tm));
                update_tower(utm, &mut tower, &mut inf_extender);
            }
            prev_cstate = tm.state;
        }

        // Periodic checks (every 100K steps to avoid syscall overhead)
        if total_steps % 100_000 == 0 {
            // Savepoint every 1B steps
            if let Some(ref sp_path) = savepoint_path {
                if total_steps - last_savepoint_step >= 1_000_000_000 {
                    save_savepoint(sp_path, total_steps, guest_steps, &tm);
                    last_savepoint_step = total_steps;
                }
            }

            // Print every 0.1s
            if last_print.elapsed() >= print_interval {
                tower[0].update_machine(compiled.decompile(&tm));
                let wall_secs = start_time.elapsed().as_secs_f64().max(0.001);
                eprint!(
                    "{}  ({} guest steps, {:.1}M steps/s)\n",
                    format_tower(&mut tower, total_steps, utm, &mut inf_extender),
                    guest_steps,
                    total_steps as f64 / wall_secs / 1_000_000.0
                );
                last_print = std::time::Instant::now();
            }
        }
    }
}
