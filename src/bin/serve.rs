use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::path::Path;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use tiny_http::{Header, Response, Server};
use utmmmmm::compiled::{CState, CompiledTapeExtender, CompiledTuringMachineSpec};
use utmmmmm::delta::{compute_new_overwrites, current_overwrites, ClientLevelState};
use utmmmmm::infinity::InfiniteTapeExtender;
use utmmmmm::savepoint::{load_savepoint, save_savepoint};
use utmmmmm::tm::{
    Dir, RunningTMStatus, RunningTuringMachine, SimpleTuringMachineSpec, TapeExtender,
    TuringMachineSpec,
};
use utmmmmm::tower::Tower;
use utmmmmm::utm::{State, Symbol, UTM_SPEC};

// ── Snapshot: shared between tower thread and SSE client threads ──

struct Snapshot {
    levels: Vec<TowerLevelJson>,
}

type SseClient = mpsc::Sender<Arc<Snapshot>>;
type SseClients = Arc<Mutex<Vec<SseClient>>>;

// ── JSON event types ──

#[derive(Serialize, Clone)]
struct TowerLevelJson {
    steps: u64,
    head_pos: usize,
    state: String,
    overwrites: HashMap<usize, char>,
}

#[derive(Serialize)]
struct TotalEventJson {
    #[serde(rename = "type")]
    event_type: &'static str,
    unblemished: String,
    utm_states: Vec<String>,
    utm_symbol_chars: String,

    levels: Vec<TowerLevelJson>,
}

#[derive(Serialize)]
struct DeltaEventJson {
    #[serde(rename = "type")]
    event_type: &'static str,
    levels: Vec<TowerLevelJson>,
}

// ── Build snapshot from decompiled L0 machine ──

fn build_snapshot(
    decompiled: &RunningTuringMachine<SimpleTuringMachineSpec<State, Symbol>>,
    total_steps: u64,
    inf_extender: &mut InfiniteTapeExtender,
    reference: &mut Vec<Symbol>,
) -> Snapshot {
    inf_extender.extend(reference, decompiled.tape.len());
    Snapshot {
        levels: vec![TowerLevelJson {
            steps: total_steps,
            head_pos: decompiled.pos,
            state: format!("{:?}", decompiled.state),
            overwrites: current_overwrites(&decompiled.tape, &reference)
                .iter()
                .map(|(&i, s)| (i, s.to_string().chars().next().unwrap()))
                .collect::<HashMap<_, _>>(),
        }],
    }
}

fn publish(
    latest: &Mutex<Option<Arc<Snapshot>>>,
    sse_clients: &Mutex<Vec<SseClient>>,
    snap: Arc<Snapshot>,
) {
    *latest.lock().unwrap() = Some(Arc::clone(&snap));
    let mut clients = sse_clients.lock().unwrap();
    clients.retain(|tx| tx.send(Arc::clone(&snap)).is_ok());
}

// ── Main simulation thread ──

fn sim_thread(
    latest: Arc<Mutex<Option<Arc<Snapshot>>>>,
    sse_clients: SseClients,
    savepoint_path: Option<String>,
) {
    let utm = &*UTM_SPEC;
    let compiled = CompiledTuringMachineSpec::compile(utm).expect("UTM should compile");

    let mut extender = CompiledTapeExtender::new(&compiled, Box::new(InfiniteTapeExtender));

    let mut tower = Tower::new(RunningTuringMachine::new(&compiled));
    // if tower.base.tm.pos >= tower.base.tm.tape.len() {
    //     extender.extend(&mut tower.base.tm.tape, 10);
    // }

    if let Some(ref _sp_path) = savepoint_path {
        todo!();
    }

    let mut base_max_pos: usize = tower.base.tm.pos;
    let mut inf_extender = InfiniteTapeExtender;
    let mut last_savepoint_step = tower.base.total_steps;

    // Reference tape for overwrite comparison
    let mut reference: Vec<Symbol> = Vec::new();

    let snapshot_interval = Duration::from_millis(30);
    let mut last_snapshot = Instant::now();
    let start_time = Instant::now();
    let mut prev_cstate = tower.base.tm.state;

    // Profiling: time spent doing things other than the hot loop
    let mut total_overhead = Duration::ZERO;
    let mut last_profile_print = Instant::now();

    // Initial snapshot
    {
        let decompiled = compiled.decompile(&tower.base.tm);
        let snap = Arc::new(build_snapshot(
            &decompiled,
            tower.base.total_steps,
            &mut inf_extender,
            &mut reference,
        ));
        publish(&latest, &sse_clients, snap);
    }

    loop {
        if let RunningTMStatus::Accepted | RunningTMStatus::Rejected = tower.step(&mut extender) {
            panic!("infinite machine should never halt");
        }

        if tower.base.tm.pos > base_max_pos {
            base_max_pos = tower.base.tm.pos;
        }

        if tower.base.tm.state != prev_cstate {
            prev_cstate = tower.base.tm.state;
        }

        let total_steps = tower.base.total_steps;
        if total_steps % 100_000 == 0 {
            let overhead_start_at = Instant::now();
            if total_steps - last_savepoint_step >= 1_000_000_000 {
                if let Some(ref sp_path) = savepoint_path {
                    save_savepoint(sp_path, total_steps, &tower.base.tm);
                    last_savepoint_step = total_steps;
                }
            }

            if last_snapshot.elapsed() >= snapshot_interval {
                let decompiled = compiled.decompile(&tower.base.tm);
                let snap = Arc::new(build_snapshot(
                    &decompiled,
                    total_steps,
                    &mut inf_extender,
                    &mut reference,
                ));
                publish(&latest, &sse_clients, snap);
                last_snapshot = Instant::now();
            }

            if last_profile_print.elapsed() >= Duration::from_secs(10) {
                let total_overhead_ms = total_overhead.as_secs_f64() * 1000.0;
                let total_runtime_ms = start_time.elapsed().as_secs_f64() * 1000.0;
                eprintln!(
                    "[profile] overhead: {:.1}ms / {:.1}s ({:.2}%)",
                    total_overhead_ms,
                    total_runtime_ms,
                    total_overhead_ms / total_runtime_ms * 100.0,
                );
                last_profile_print = Instant::now();
            }

            total_overhead += overhead_start_at.elapsed();
        }
    }
}

// ── SSE client thread ──

fn sse_client_thread(
    rx: mpsc::Receiver<Arc<Snapshot>>,
    latest: Arc<Mutex<Option<Arc<Snapshot>>>>,
    unblemished_str: Arc<String>,
    unblemished_syms: Arc<Vec<Symbol>>,
    utm_states: Arc<Vec<String>>,
    utm_symbol_chars: Arc<String>,
    mut writer: Box<dyn IoWrite + Send>,
) {
    // Get initial snapshot (prefer stored latest, fall back to waiting on channel)
    let initial = {
        let stored = latest.lock().unwrap().as_ref().map(Arc::clone);
        match stored {
            Some(snap) => snap,
            None => match rx.recv() {
                Ok(snap) => snap,
                Err(_) => return,
            },
        }
    };

    // Send total event
    let total = TotalEventJson {
        event_type: "total",
        unblemished: (*unblemished_str).clone(),
        utm_states: (*utm_states).clone(),
        utm_symbol_chars: (*utm_symbol_chars).clone(),
        levels: initial.levels.clone(),
    };
    let json = serde_json::to_string(&total).unwrap();
    if write!(writer, "data: {}\n\n", json).is_err() || writer.flush().is_err() {
        return;
    }

    // Initialize client state from the total snapshot's overwrites
    let mut client_state = ClientLevelState {
        overwrites: vec![initial.levels[0].overwrites.clone()],
    };

    // Stream delta events
    while let Ok(snapshot) = rx.recv() {
        let delta = DeltaEventJson {
            event_type: "delta",
            levels: snapshot
                .levels
                .iter()
                .map(|level| {
                    let new_overwrites = compute_new_overwrites(
                        &level.overwrites,
                        &mut client_state,
                        &unblemished_syms
                            .iter()
                            .map(|s| s.to_string().chars().next().unwrap())
                            .collect::<Vec<_>>(),
                    );

                    TowerLevelJson {
                        steps: level.steps,
                        state: level.state.clone(),
                        head_pos: level.head_pos,
                        overwrites: new_overwrites
                            .into_iter()
                            .map(|(pos, s)| (pos, s.to_string().chars().next().unwrap()))
                            .collect::<HashMap<_, _>>(),
                    }
                })
                .collect(),
        };
        let json = serde_json::to_string(&delta).unwrap();
        if write!(writer, "data: {}\n\n", json).is_err() || writer.flush().is_err() {
            break;
        }
    }
}

// ── HTTP ──

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

    // Pre-compute the unblemished infinite tape (1M symbols)
    let unblemished_syms = {
        let mut syms: Vec<Symbol> = Vec::new();
        InfiniteTapeExtender.extend(&mut syms, 1_000_000);
        Arc::new(syms)
    };
    let unblemished_str: Arc<String> =
        Arc::new(unblemished_syms.iter().map(|s| format!("{}", s)).collect());

    // Pre-compute UTM metadata for client-side decoding
    let utm = &*UTM_SPEC;
    let utm_states: Arc<Vec<String>> =
        Arc::new(utm.iter_states().map(|s| format!("{:?}", s)).collect());
    let utm_symbol_chars: Arc<String> =
        Arc::new(utm.iter_symbols().map(|s| format!("{}", s)).collect());

    let latest: Arc<Mutex<Option<Arc<Snapshot>>>> = Arc::new(Mutex::new(None));
    let sse_clients: SseClients = Arc::new(Mutex::new(Vec::new()));

    // Start simulation background thread
    let latest_clone = Arc::clone(&latest);
    let sse_clone = Arc::clone(&sse_clients);
    thread::spawn(move || sim_thread(latest_clone, sse_clone, savepoint_path));

    let addr = format!("0.0.0.0:{}", port);
    let server = Server::http(&addr).expect("Failed to start HTTP server");
    eprintln!("Serving on http://localhost:{}", port);

    // Find ui/dist relative to the cargo manifest directory or current dir
    let dist_dir = if Path::new("ui/dist").is_dir() {
        Path::new("ui/dist").to_path_buf()
    } else if Path::new("dist").is_dir() {
        Path::new("dist").to_path_buf()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/dist")
    };

    for request in server.incoming_requests() {
        let url = request.url().to_string();

        if url == "/api/tower" {
            // SSE: grab the raw socket and stream events
            let latest_c = Arc::clone(&latest);
            let clients_c = Arc::clone(&sse_clients);
            let ub_str = Arc::clone(&unblemished_str);
            let ub_syms = Arc::clone(&unblemished_syms);
            let utm_st = Arc::clone(&utm_states);
            let utm_sc = Arc::clone(&utm_symbol_chars);
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

            // Create channel and register BEFORE reading latest snapshot,
            // so we don't miss any broadcasts between reading latest and subscribing.
            let (tx, rx) = mpsc::channel();
            clients_c.lock().unwrap().push(tx);

            thread::spawn(move || {
                sse_client_thread(rx, latest_c, ub_str, ub_syms, utm_st, utm_sc, writer);
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
