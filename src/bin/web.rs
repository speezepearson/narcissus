use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::{extract::{Query, State}, response::Html, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use utmmmmm::tm::{Dir, TuringMachine};
use utmmmmm::utm::{self, UtmSym, UtmState};

// ---- write1 TM (must match state_bits=2, symbol_bits=1) ----

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum W1State { Start, Accept, Reject }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum W1Sym { Blank, One }

fn write1_tm() -> TuringMachine<W1State, W1Sym> {
    let mut t = std::collections::HashMap::new();
    t.insert(
        (W1State::Start, W1Sym::Blank),
        (W1State::Accept, W1Sym::One, Dir::Right),
    );
    TuringMachine {
        initial: W1State::Start,
        accept: W1State::Accept,
        reject: W1State::Reject,
        blank: W1Sym::Blank,
        transitions: t,
    }
}

// ---- Shared state ----

struct App {
    outer_sym_table: Vec<UtmSym>,
    inner_sym_table: Vec<W1Sym>,
    initial_tape: Vec<UtmSym>,
    sim: Mutex<UtmState>,
    /// Steps per second (0 = unlimited).
    sps: AtomicU64,
}

impl App {
    fn new() -> Self {
        let w1 = write1_tm();
        let inner_tape = utm::encode(&w1, &[]);
        let utm_tm = utm::build_utm_tm();
        let outer_tape = utm::encode(&utm_tm, &inner_tape);

        let outer_sym_table = build_sym_table(&utm_tm);
        let inner_sym_table = build_sym_table(&w1);

        let sim = Mutex::new(UtmState::new(&outer_tape));
        App {
            outer_sym_table, inner_sym_table, initial_tape: outer_tape, sim,
            sps: AtomicU64::new(0), // unlimited
        }
    }

    fn reset(&self) {
        *self.sim.lock().unwrap() = UtmState::new(&self.initial_tape);
    }
}

fn build_sym_table<S, A>(tm: &TuringMachine<S, A>) -> Vec<A>
where
    S: Eq + std::hash::Hash + Clone + std::fmt::Debug,
    A: Eq + std::hash::Hash + Clone + std::fmt::Debug,
{
    let mut symbols: Vec<A> = Vec::new();
    fn add_unique<T: Eq + Clone>(vec: &mut Vec<T>, item: &T) {
        if !vec.contains(item) { vec.push(item.clone()); }
    }
    for ((_, a), (_, a2, _)) in &tm.transitions {
        add_unique(&mut symbols, a);
        add_unique(&mut symbols, a2);
    }
    add_unique(&mut symbols, &tm.blank);
    symbols
}

type SharedApp = Arc<App>;

// ---- JSON types ----

#[derive(Serialize)]
struct TapeView {
    state: String,
    head: i64,
    len: i64,
    /// For display centering (may differ from head for raw tape views).
    #[serde(skip)]
    display_center: i64,
    tape: Vec<Option<String>>,
}

#[derive(Serialize)]
struct StateResponse {
    steps: u64,
    halted: bool,
    accepted: bool,
    sps: u64,
    outer: TapeView,
    middle: Option<TapeView>,
    inner: Option<TapeView>,
}

// ---- Symbol display ----

fn utmsym_char(s: UtmSym) -> char {
    match s {
        UtmSym::Zero => '0',
        UtmSym::One => '1',
        UtmSym::LBracket => '[',
        UtmSym::RBracket => ']',
        UtmSym::Pipe => '|',
        UtmSym::Semi => ';',
        UtmSym::Hash => '#',
        UtmSym::D => 'D',
        UtmSym::Blank => '_',
        UtmSym::Dot0 => 'a',
        UtmSym::Dot1 => 'b',
        UtmSym::MarkLBracket => '(',
    }
}

const WINDOW: i64 = 20;

fn build_tape_view(
    state: String,
    head: i64,
    len: i64,
    sym_fn: impl Fn(usize) -> String,
) -> TapeView {
    build_tape_view_centered(state, head, len, head, len, sym_fn)
}

/// Like build_tape_view but the tape window is centered on `display_center`
/// (in a tape of `display_len` symbols), while head/len report different values.
fn build_tape_view_centered(
    state: String,
    head: i64,
    len: i64,
    display_center: i64,
    display_len: i64,
    sym_fn: impl Fn(usize) -> String,
) -> TapeView {
    let tape: Vec<Option<String>> = (display_center - WINDOW..=display_center + WINDOW)
        .map(|pos| {
            if pos < 0 || pos >= display_len {
                None
            } else {
                Some(sym_fn(pos as usize))
            }
        })
        .collect();
    TapeView { state, head, len, display_center, tape }
}

fn w1sym_char(s: W1Sym) -> char {
    match s {
        W1Sym::Blank => '_',
        W1Sym::One => '1',
    }
}

// ---- Handlers ----

async fn handle_state(State(app): State<SharedApp>) -> Json<StateResponse> {
    let sim = app.sim.lock().unwrap();

    let outer = build_tape_view(
        format!("{}", sim.current_state),
        sim.head_pos as i64,
        sim.cells.len() as i64,
        |pos| {
            let idx = sim.cells[pos].1 as usize;
            app.outer_sym_table.get(idx)
                .map(|s| utmsym_char(*s).to_string())
                .unwrap_or("?".to_string())
        },
    );

    let middle_tape: Vec<UtmSym> = sim.cells.iter()
        .map(|(_, sym_idx)| {
            app.outer_sym_table.get(*sym_idx as usize).copied().unwrap_or(UtmSym::Blank)
        })
        .collect();

    let middle_decoded = utm::decode_running_state(&middle_tape);

    let middle = middle_decoded.as_ref().map(|md| {
        // Center the display on the raw UtmSym position of the head cell,
        // but report head/len as cell indices.
        let hash_pos = middle_tape.iter().position(|s| matches!(s, UtmSym::Hash));
        let cell_width = 1 + md.state_bits + 1 + md.symbol_bits + 1;
        let data_start = hash_pos.map(|h| h + 1).unwrap_or(0);
        let raw_head = data_start + md.head_pos * cell_width;

        build_tape_view_centered(
            format!("{}", md.state),
            md.head_pos as i64,
            md.tape_syms.len() as i64,
            raw_head as i64,
            middle_tape.len() as i64,
            |pos| utmsym_char(middle_tape[pos]).to_string(),
        )
    });

    let inner = middle_decoded.as_ref().map(|md| {
        build_tape_view(
            format!("{}", md.state),
            md.head_pos as i64,
            md.tape_syms.len() as i64,
            |pos| {
                let idx = md.tape_syms[pos] as usize;
                app.inner_sym_table.get(idx)
                    .map(|s| w1sym_char(*s).to_string())
                    .unwrap_or("?".to_string())
            },
        )
    });

    Json(StateResponse {
        steps: sim.steps,
        halted: sim.halted,
        accepted: sim.accepted,
        sps: app.sps.load(Ordering::Relaxed),
        outer,
        middle,
        inner,
    })
}

#[derive(Deserialize)]
struct SpeedParams {
    sps: u64,
}

async fn handle_speed(State(app): State<SharedApp>, Query(params): Query<SpeedParams>) -> &'static str {
    app.sps.store(params.sps, Ordering::Relaxed);
    "ok"
}

async fn handle_reset(State(app): State<SharedApp>) -> &'static str {
    app.reset();
    "ok"
}

async fn handle_index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

const INDEX_HTML: &str = r##"<!DOCTYPE html>
<html>
<head>
<title>UTM on UTM</title>
<style>
body { background: #111; color: #0f0; font-family: monospace; font-size: 14px; padding: 20px; }
h1 { color: #0f0; font-size: 18px; }
h2 { color: #0a0; font-size: 14px; margin-top: 20px; margin-bottom: 4px; }
pre { margin: 0; line-height: 1.4; }
.tape-line { color: #0f0; letter-spacing: 1px; }
.head-line { color: #f80; letter-spacing: 1px; }
.controls { margin: 10px 0; }
button { background: #333; color: #0f0; border: 1px solid #0f0; padding: 5px 15px;
         cursor: pointer; font-family: monospace; margin-right: 10px; }
button:hover { background: #050; }
.info { color: #888; margin-bottom: 10px; }
label { color: #888; margin-right: 5px; }
input[type=range] { vertical-align: middle; }
.speed-val { color: #0f0; margin-left: 5px; }
</style>
</head>
<body>
<h1>UTM simulating UTM simulating write1</h1>
<div class="controls">
  <button onclick="doReset()">Reset</button>
  <span class="info" id="info">steps: 0</span>
  <br><br>
  <label>Speed:</label>
  <input type="range" id="speed" min="0" max="7" value="0" oninput="setSpeed()">
  <span class="speed-val" id="speed-val">unlimited</span>
</div>

<h2>Outer: UTM TM state (being simulated by interpreter)</h2>
<pre><span class="tape-line" id="outer-tape"></span>
<span class="head-line" id="outer-head"></span></pre>

<h2>Middle: UTM TM's tape (raw UtmSym encoding of write1)</h2>
<pre><span class="tape-line" id="middle-tape"></span>
<span class="head-line" id="middle-head"></span></pre>

<h2>Inner: write1's tape (decoded from middle)</h2>
<pre><span class="tape-line" id="inner-tape"></span>
<span class="head-line" id="inner-head"></span></pre>

<script>
// Speed steps: 0=unlimited, 1=1, 2=10, 3=100, 4=1000, 5=10000, 6=100000, 7=1000000
const SPEED_LEVELS = [0, 1, 10, 100, 1000, 10000, 100000, 1000000];
const SPEED_LABELS = ['unlimited', '1/s', '10/s', '100/s', '1k/s', '10k/s', '100k/s', '1M/s'];

function setSpeed() {
    const idx = parseInt(document.getElementById('speed').value);
    const sps = SPEED_LEVELS[idx];
    document.getElementById('speed-val').textContent = SPEED_LABELS[idx];
    fetch('/speed?sps=' + sps);
}

function renderTape(tapeArr) {
    return tapeArr.map(s => {
        if (s === null) return ' ';
        return s.charAt(0);
    }).join('');
}

function renderHead(state, pos, len) {
    return ' '.repeat(20) + '^ (state=' + state + ') (pos=' + pos + '/' + len + ')';
}

function renderView(prefix, view) {
    if (!view) {
        document.getElementById(prefix + '-tape').textContent = '  (not yet decodable)';
        document.getElementById(prefix + '-head').textContent = '';
        return;
    }
    document.getElementById(prefix + '-tape').textContent = renderTape(view.tape);
    document.getElementById(prefix + '-head').textContent = renderHead(view.state, view.head, view.len);
}

async function refresh() {
    try {
        const resp = await fetch('/state');
        const data = await resp.json();
        let info = 'steps: ' + data.steps;
        if (data.halted) info += (data.accepted ? ' (ACCEPTED)' : ' (REJECTED)');
        document.getElementById('info').textContent = info;
        renderView('outer', data.outer);
        renderView('middle', data.middle);
        renderView('inner', data.inner);
    } catch(e) {
        console.error(e);
    }
}

async function doReset() {
    await fetch('/reset');
    refresh();
}

refresh();
setInterval(refresh, 200);
</script>
</body>
</html>
"##;

// ---- Main ----

#[tokio::main]
async fn main() {
    let app = Arc::new(App::new());
    let app_clone = Arc::clone(&app);

    // Stepping thread: step the UTM interpreter with rate limiting
    std::thread::spawn(move || {
        let mut last_step = Instant::now();
        let mut steps_this_second: u64 = 0;
        let mut second_start = Instant::now();

        loop {
            let sps = app_clone.sps.load(Ordering::Relaxed);

            {
                let mut sim = app_clone.sim.lock().unwrap();
                if sim.halted {
                    drop(sim);
                    std::thread::sleep(Duration::from_millis(100));
                    steps_this_second = 0;
                    second_start = Instant::now();
                    continue;
                }

                if sps == 0 {
                    // Unlimited: step in batches
                    for _ in 0..1000 {
                        if !sim.step() { break; }
                    }
                } else {
                    // Rate limited: step up to sps per second
                    let now = Instant::now();
                    if now.duration_since(second_start) >= Duration::from_secs(1) {
                        steps_this_second = 0;
                        second_start = now;
                    }

                    if steps_this_second < sps {
                        // Step a batch proportional to sps, but not more than remaining budget
                        let remaining = sps - steps_this_second;
                        let batch = remaining.min(100).max(1);
                        for _ in 0..batch {
                            if !sim.step() { break; }
                            steps_this_second += 1;
                        }
                    }
                }
            }

            let sps = app_clone.sps.load(Ordering::Relaxed);
            if sps == 0 {
                std::thread::yield_now();
            } else {
                // Sleep to pace steps. For low sps, sleep longer.
                let sleep_us = if sps <= 100 {
                    1_000_000 / sps.max(1)
                } else {
                    1_000 // 1ms for higher rates
                };
                std::thread::sleep(Duration::from_micros(sleep_us));
            }
        }
    });

    let router = Router::new()
        .route("/", get(handle_index))
        .route("/state", get(handle_state))
        .route("/speed", get(handle_speed))
        .route("/reset", get(handle_reset))
        .with_state(app);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Listening on http://localhost:8080");
    axum::serve(listener, router).await.unwrap();
}
