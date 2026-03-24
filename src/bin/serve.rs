use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::path::Path;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use tiny_http::{Header, Response, Server};
use utmmmmm::compiled::{CState, CSymbol, CompiledTapeExtender, CompiledTuringMachineSpec};
use utmmmmm::infinity::InfiniteTapeExtender;
use utmmmmm::tm::{
    Dir, RunningTuringMachine, SimpleTuringMachineSpec, TapeExtender, TuringMachineSpec,
};
use utmmmmm::tower::{decode_next_level, update_tower, TowerLevel};
use utmmmmm::utm::{State, Symbol, UTM_SPEC};

type SseClients = Arc<Mutex<Vec<mpsc::Sender<String>>>>;

#[derive(Serialize)]
struct TowerDeltaLevelJson {
    head_pos: usize,
    state: String,
    tape_len: usize,
    overwritten: Vec<(usize, String)>,
}

#[derive(Serialize)]
struct TowerDeltaJson {
    steps: u64,
    guest_steps: u64,
    steps_per_sec: f64,
    tower: Vec<TowerDeltaLevelJson>,
}

fn level_overwrites(tape: &[Symbol], reference: &[Symbol]) -> Vec<(usize, String)> {
    tape.iter()
        .zip(reference.iter())
        .enumerate()
        .filter(|(_, (c, o))| c != o)
        .map(|(i, (c, _))| {
            let mut s = String::new();
            write!(s, "{}", c).unwrap();
            (i, s)
        })
        .collect()
}

fn build_delta<'a>(
    tower: &mut [TowerLevel<'a>],
    total_steps: u64,
    guest_steps: u64,
    steps_per_sec: f64,
    utm: &'a SimpleTuringMachineSpec<State, Symbol>,
    inf_extender: &mut InfiniteTapeExtender,
    reference: &mut Vec<Symbol>,
) -> TowerDeltaJson {
    let mut levels = Vec::new();

    for tl in tower.iter() {
        let tape = &tl.machine.tape;
        inf_extender.extend(reference, tape.len());
        levels.push(TowerDeltaLevelJson {
            head_pos: tl.machine.pos,
            state: format!("{:?}", tl.machine.state),
            tape_len: tape.len(),
            overwritten: level_overwrites(tape, reference),
        });
    }

    // Extra level beyond the tower
    let last = tower.last_mut().unwrap();
    if let Some(extra) = decode_next_level(utm, &mut last.machine, inf_extender) {
        inf_extender.extend(reference, extra.tape.len());
        levels.push(TowerDeltaLevelJson {
            head_pos: extra.pos,
            state: format!("{:?}", extra.state),
            tape_len: extra.tape.len(),
            overwritten: level_overwrites(&extra.tape, reference),
        });
    }

    TowerDeltaJson {
        steps: total_steps,
        guest_steps,
        steps_per_sec,
        tower: levels,
    }
}

fn broadcast(snapshot: &Mutex<String>, sse_clients: &Mutex<Vec<mpsc::Sender<String>>>, json_str: String) {
    {
        let mut clients = sse_clients.lock().unwrap();
        clients.retain(|tx| tx.send(json_str.clone()).is_ok());
    }
    *snapshot.lock().unwrap() = json_str;
}

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

fn tower_thread(
    snapshot: Arc<Mutex<String>>,
    sse_clients: SseClients,
    savepoint_path: Option<String>,
) {
    let utm = &*UTM_SPEC;
    let compiled = CompiledTuringMachineSpec::compile(utm).expect("UTM should compile");

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

    if let Some(ref sp_path) = savepoint_path {
        if let Some((sp_steps, sp_guest, sp_state, sp_pos, sp_tape)) = load_savepoint(sp_path) {
            total_steps = sp_steps;
            guest_steps = sp_guest;
            tm.state = sp_state;
            tm.pos = sp_pos;
            tm.tape = sp_tape;
            let tape_len = tm.tape.len();
            extender.extend(&mut tm.tape, tape_len);
            eprintln!(
                "Loaded savepoint from {}: step {}, {} guest steps, tape len {}",
                sp_path, total_steps, guest_steps, tm.tape.len()
            );
        }
    }

    let mut base_max_pos: usize = tm.pos;
    let mut inf_extender = InfiniteTapeExtender;
    let mut last_savepoint_step = total_steps;

    // Reference tape for overwrite comparison (same for all levels due to fixed point)
    let mut reference: Vec<Symbol> = Vec::new();

    let mut tower: Vec<TowerLevel> = vec![TowerLevel::new(compiled.decompile(&tm))];
    if tm.state == init_cstate {
        update_tower(utm, &mut tower, &mut inf_extender);
    }
    tower[0].max_head_pos = base_max_pos;

    let snapshot_interval = Duration::from_millis(100);
    let mut last_snapshot = Instant::now();
    let start_time = Instant::now();
    let mut prev_cstate = tm.state;

    // Profiling: time spent in the snapshot block
    let mut snapshot_time_accum = Duration::ZERO;
    let mut last_profile_print = Instant::now();

    // Initial snapshot
    {
        let delta = build_delta(
            &mut tower, total_steps, guest_steps, 0.0, utm, &mut inf_extender, &mut reference,
        );
        broadcast(&snapshot, &sse_clients, serde_json::to_string(&delta).unwrap());
    }

    loop {
        if tm.pos >= tm.tape.len() {
            extender.extend(&mut tm.tape, tm.pos + 1);
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
            if tm.pos > base_max_pos {
                base_max_pos = tm.pos;
            }
        } else {
            // Halted
            tower[0].update_machine(compiled.decompile(&tm));
            tower[0].max_head_pos = base_max_pos;
            update_tower(utm, &mut tower, &mut inf_extender);
            let delta = build_delta(
                &mut tower, total_steps, guest_steps, 0.0, utm, &mut inf_extender, &mut reference,
            );
            broadcast(&snapshot, &sse_clients, serde_json::to_string(&delta).unwrap());
            if let Some(ref sp_path) = savepoint_path {
                save_savepoint(sp_path, total_steps, guest_steps, &tm);
            }
            return;
        }

        if tm.state != prev_cstate {
            if tm.state == init_cstate {
                guest_steps += 1;
                tower[0].update_machine(compiled.decompile(&tm));
                tower[0].max_head_pos = base_max_pos;
                update_tower(utm, &mut tower, &mut inf_extender);
            }
            prev_cstate = tm.state;
        }

        if total_steps % 100_000 == 0 {
            if let Some(ref sp_path) = savepoint_path {
                if total_steps - last_savepoint_step >= 1_000_000_000 {
                    save_savepoint(sp_path, total_steps, guest_steps, &tm);
                    last_savepoint_step = total_steps;
                }
            }
        }

        if total_steps % 100_000 == 0 && last_snapshot.elapsed() >= snapshot_interval {
            let snap_start = Instant::now();
            tower[0].update_machine(compiled.decompile(&tm));
            tower[0].max_head_pos = base_max_pos;
            let wall_secs = start_time.elapsed().as_secs_f64().max(0.001);
            let steps_per_sec = total_steps as f64 / wall_secs / 1_000_000.0;
            let delta = build_delta(
                &mut tower, total_steps, guest_steps, steps_per_sec, utm, &mut inf_extender, &mut reference,
            );
            broadcast(&snapshot, &sse_clients, serde_json::to_string(&delta).unwrap());
            last_snapshot = Instant::now();
            snapshot_time_accum += snap_start.elapsed();
        }

        if last_profile_print.elapsed() >= Duration::from_secs(10) {
            let elapsed = last_profile_print.elapsed();
            eprintln!(
                "[profile] snapshot block: {:.1}ms / {:.1}s ({:.2}%)",
                snapshot_time_accum.as_secs_f64() * 1000.0,
                elapsed.as_secs_f64(),
                snapshot_time_accum.as_secs_f64() / elapsed.as_secs_f64() * 100.0,
            );
            snapshot_time_accum = Duration::ZERO;
            last_profile_print = Instant::now();
        }
    }
}

fn content_type_for(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".js") {
        "application/javascript; charset=utf-8"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".wasm") {
        "application/wasm"
    } else {
        "application/octet-stream"
    }
}

fn get_flag(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .map(|i| args[i + 1].clone())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let savepoint_path = get_flag(&args, "--savepoint");
    let port = get_flag(&args, "--port")
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(8080);

    // Pre-compute the unblemished infinite tape (first 1M symbols)
    let mut unblemished_syms: Vec<Symbol> = Vec::new();
    InfiniteTapeExtender.extend(&mut unblemished_syms, 1_000_000);
    let unblemished_tape: Arc<String> = Arc::new(
        unblemished_syms
            .iter()
            .map(|s| format!("{}", s))
            .collect(),
    );

    let snapshot: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let sse_clients: SseClients = Arc::new(Mutex::new(Vec::new()));

    // Start tower background thread
    let snap_clone = Arc::clone(&snapshot);
    let sse_clone = Arc::clone(&sse_clients);
    thread::spawn(move || tower_thread(snap_clone, sse_clone, savepoint_path));

    let addr = format!("0.0.0.0:{}", port);
    let server = Server::http(&addr).expect("Failed to start HTTP server");
    eprintln!("Serving on http://localhost:{}", port);

    // Find ui/dist relative to the cargo manifest directory or current dir
    let dist_dir = if Path::new("ui/dist").is_dir() {
        Path::new("ui/dist").to_path_buf()
    } else if Path::new("dist").is_dir() {
        Path::new("dist").to_path_buf()
    } else {
        // Fall back to manifest dir
        Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/dist")
    };

    for request in server.incoming_requests() {
        let url = request.url().to_string();

        if url == "/api/tape" {
            let text = (*unblemished_tape).clone();
            let response = Response::from_string(text)
                .with_header(
                    Header::from_bytes("Content-Type", "text/plain; charset=utf-8").unwrap(),
                )
                .with_header(
                    Header::from_bytes("Cache-Control", "public, max-age=86400, immutable")
                        .unwrap(),
                );
            let _ = request.respond(response);
            continue;
        }

        if url == "/api/tower" {
            // SSE: grab the raw socket and stream events
            let snap = Arc::clone(&snapshot);
            let clients = Arc::clone(&sse_clients);
            let mut writer = request.into_writer();

            // Write HTTP response headers for SSE
            let header_ok = write!(
                writer,
                "HTTP/1.1 200 OK\r\n\
                 Content-Type: text/event-stream\r\n\
                 Cache-Control: no-cache\r\n\
                 Connection: keep-alive\r\n\
                 \r\n"
            )
            .is_ok()
                && writer.flush().is_ok();

            if !header_ok {
                continue;
            }

            thread::spawn(move || {
                let (tx, rx) = mpsc::channel();

                // Send current snapshot as first event
                {
                    let current = snap.lock().unwrap().clone();
                    if !current.is_empty() {
                        if write!(writer, "data: {}\n\n", current).is_err() {
                            return;
                        }
                        if writer.flush().is_err() {
                            return;
                        }
                    }
                }

                // Register for future broadcasts
                clients.lock().unwrap().push(tx);

                // Stream events until client disconnects or channel closes
                while let Ok(json) = rx.recv() {
                    if write!(writer, "data: {}\n\n", json).is_err() {
                        break;
                    }
                    if writer.flush().is_err() {
                        break;
                    }
                }
            });
            continue;
        }

        // Serve static files from ui/dist/
        let file_path = if url == "/" {
            dist_dir.join("index.html")
        } else {
            dist_dir.join(url.trim_start_matches('/'))
        };

        if file_path.is_file() {
            match std::fs::read(&file_path) {
                Ok(data) => {
                    let ct = content_type_for(file_path.to_str().unwrap_or(""));
                    let response = Response::from_data(data)
                        .with_header(Header::from_bytes("Content-Type", ct).unwrap());
                    let _ = request.respond(response);
                }
                Err(_) => {
                    let _ = request.respond(Response::from_string("500").with_status_code(500));
                }
            }
        } else {
            // SPA fallback: serve index.html for unmatched routes
            let index = dist_dir.join("index.html");
            match std::fs::read(&index) {
                Ok(data) => {
                    let response = Response::from_data(data).with_header(
                        Header::from_bytes("Content-Type", "text/html; charset=utf-8").unwrap(),
                    );
                    let _ = request.respond(response);
                }
                Err(_) => {
                    let _ = request.respond(Response::from_string("404").with_status_code(404));
                }
            }
        }
    }
}
