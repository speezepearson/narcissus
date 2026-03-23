
use std::cmp::max;
use std::fmt::Write;
use std::io::Write as IoWrite;
use utmmmmm::tm::{RunningTuringMachine, TapeExtender, TuringMachineSpec, SimpleTuringMachineSpec, Dir};
use utmmmmm::compiled::{CState, CSymbol, CompiledTapeExtender, CompiledTuringMachineSpec};
use utmmmmm::infinity::{header_len, InfiniteTapeExtender};
use utmmmmm::utm::{State, Symbol, MyUtmEncodingScheme, UtmEncodingScheme, UTM_SPEC};

type UtmTm<'a> = RunningTuringMachine<'a, SimpleTuringMachineSpec<State, Symbol>>;

const RADIUS: usize = 50;

/// Format a tape view: 30 symbols on each side of the head, with ^ below.
fn tape_view(tm: &UtmTm) -> String {
    let mut top = String::new();
    let mut bot = String::new();

    let prefix = if tm.pos > RADIUS { "... " } else { "    " };
    top.push_str(prefix);
    bot.push_str("    ");

    for i in 0..(2 * RADIUS + 1) {
        let tape_idx = tm.pos as isize + i as isize - RADIUS as isize;
        let sym = if tape_idx < 0 {
            " ".to_string()
        } else {
            let idx = tape_idx as usize;
            if idx < tm.tape.len() {
                tm.tape[idx].to_string()
            } else {
                tm.spec.blank().to_string()
            }
        };
        top.push_str(&sym);

        if i == RADIUS {
            bot.push('^');
        } else {
            bot.push(' ');
        }
    }

    top.push_str(" ...");
    bot.push_str(&format!(" (state={:?}, pos={})", tm.state, tm.pos));

    format!("{}\n{}", top, bot)
}

/// Decode tower[level+1] from tower[level], extending the tape as needed.
/// Returns None if decoding fails (tape too short, etc.)
fn decode_next_level<'a>(
    utm: &'a SimpleTuringMachineSpec<State, Symbol>,
    parent: &mut UtmTm<'a>,
    extender: &mut InfiniteTapeExtender,
) -> Option<UtmTm<'a>> {
    // Ensure parent tape is long enough for decoding
    let min_len = max(header_len(), parent.pos + 100);
    extender.extend(&mut parent.tape, min_len);
    MyUtmEncodingScheme::decode(utm, &parent.tape).ok()
}

/// Build the tower by decoding each level from the previous.
/// tower[0] = decompiled L0, tower[1] = decode(tower[0]), etc.
/// Re-decodes level i+1 when level i entered Init (compared to prev_states).
/// Grows the tower by at most one new level per call.
fn update_tower<'a>(
    utm: &'a SimpleTuringMachineSpec<State, Symbol>,
    tower: &mut Vec<UtmTm<'a>>,
    prev_states: &mut Vec<State>,
    extender: &mut InfiniteTapeExtender,
) {
    // Walk the tower: re-decode level+1 from level when level entered Init.
    // Level 0 is always freshly set by the caller, so always decode level 1.
    let mut level = 0;
    loop {
        // Check whether this level just entered Init
        let entered_init = if level < prev_states.len() {
            tower[level].state == State::Init && prev_states[level] != State::Init
        } else {
            // New level we haven't seen before — don't cascade further
            false
        };

        // Level 0 always triggers decoding of level 1 (caller just set it).
        // Deeper levels only trigger if they entered Init.
        if level > 0 && !entered_init {
            break;
        }

        // Decode the next level from this one
        if let Some(next) = decode_next_level(utm, &mut tower[level], extender) {
            if level + 1 < tower.len() {
                tower[level + 1] = next;
            } else {
                // Grow the tower by one
                tower.push(next);
                break; // Don't cascade into the brand-new level
            }
            level += 1;
        } else {
            break;
        }
    }

    // Snapshot current states for next comparison
    prev_states.clear();
    for tm in tower.iter() {
        prev_states.push(tm.state);
    }
}

fn format_tower(tower: &[UtmTm], total_steps: u64) -> String {
    let mut buf = String::new();
    writeln!(
        buf,
        "═══ {} steps ═══════════════════════════════════════",
        total_steps
    )
    .unwrap();

    for (i, tm) in tower.iter().enumerate() {
        writeln!(buf, "Level {} ({} symbols):", i, tm.tape.len()).unwrap();
        writeln!(buf, "{}", tape_view(tm)).unwrap();
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
    let mut tower: Vec<UtmTm> = vec![compiled.decompile(&tm)];
    let mut prev_states: Vec<State> = Vec::new();
    if tm.state == init_cstate {
        update_tower(utm, &mut tower, &mut prev_states, &mut inf_extender);
    }
    eprint!("{}", format_tower(&tower, total_steps));

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
            tower[0] = compiled.decompile(&tm);
            update_tower(utm, &mut tower, &mut prev_states, &mut inf_extender);
            eprint!("{}", format_tower(&tower, total_steps));
            let status = if compiled.is_accepting(tm.state) {
                "accept"
            } else {
                "reject"
            };
            println!(
                "halted ({}) in state {:?} after {} UTM steps ({} guest steps)",
                status, tower[0].state, total_steps, guest_steps
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
                tower[0] = compiled.decompile(&tm);
                update_tower(utm, &mut tower, &mut prev_states, &mut inf_extender);
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
                tower[0] = compiled.decompile(&tm);
                let wall_secs = start_time.elapsed().as_secs_f64().max(0.001);
                eprint!(
                    "{}  ({} guest steps, {:.1}M steps/s)\n",
                    format_tower(&tower, total_steps),
                    guest_steps,
                    total_steps as f64 / wall_secs / 1_000_000.0
                );
                last_print = std::time::Instant::now();
            }
        }
    }
}
