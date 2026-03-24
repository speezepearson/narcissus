use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use tiny_http::{Header, Response, Server};
use utmmmmm::compiled::{CState, CompiledTapeExtender, CompiledTuringMachineSpec};
use utmmmmm::infinity::InfiniteTapeExtender;
use utmmmmm::tm::{Dir, RunningTuringMachine, TuringMachineSpec};
use utmmmmm::tower::{format_tower, update_tower, TowerLevel};
use utmmmmm::utm::{State, UTM_SPEC};

fn tower_thread(snapshot: Arc<Mutex<String>>) {
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
    let mut base_max_pos: usize = tm.pos;
    let mut inf_extender = InfiniteTapeExtender;

    let mut tower: Vec<TowerLevel> = vec![TowerLevel::new(compiled.decompile(&tm))];
    if tm.state == init_cstate {
        update_tower(utm, &mut tower, &mut inf_extender);
    }
    tower[0].max_head_pos = base_max_pos;

    let snapshot_interval = Duration::from_millis(100);
    let mut last_snapshot = Instant::now();
    let start_time = Instant::now();
    let mut prev_cstate = tm.state;

    // Initial snapshot
    {
        let rendered = format_tower(&mut tower, total_steps, utm, &mut inf_extender);
        let text = format!(
            "{}  ({} guest steps, 0.0M steps/s)\n",
            rendered, guest_steps
        );
        *snapshot.lock().unwrap() = text;
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
            let rendered = format_tower(&mut tower, total_steps, utm, &mut inf_extender);
            let text = format!(
                "{}  ({} guest steps)\nHalted.\n",
                rendered, guest_steps
            );
            *snapshot.lock().unwrap() = text;
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

        if total_steps % 100_000 == 0 && last_snapshot.elapsed() >= snapshot_interval {
            tower[0].update_machine(compiled.decompile(&tm));
            tower[0].max_head_pos = base_max_pos;
            let wall_secs = start_time.elapsed().as_secs_f64().max(0.001);
            let rendered = format_tower(&mut tower, total_steps, utm, &mut inf_extender);
            let text = format!(
                "{}  ({} guest steps, {:.1}M steps/s)\n",
                rendered,
                guest_steps,
                total_steps as f64 / wall_secs / 1_000_000.0
            );
            *snapshot.lock().unwrap() = text;
            last_snapshot = Instant::now();
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

fn main() {
    let port = std::env::args()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(8080);

    let snapshot: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

    // Start tower background thread
    let snap_clone = Arc::clone(&snapshot);
    thread::spawn(move || tower_thread(snap_clone));

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

        if url == "/api/tower" {
            let text = snapshot.lock().unwrap().clone();
            let response = Response::from_string(text)
                .with_header(
                    Header::from_bytes("Content-Type", "text/plain; charset=utf-8").unwrap(),
                )
                .with_header(
                    Header::from_bytes("Cache-Control", "no-cache").unwrap(),
                );
            let _ = request.respond(response);
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
