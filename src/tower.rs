use std::cmp::max;
use std::fmt::Write;

use serde::Serialize;

use crate::infinity::{header_len, InfiniteTapeExtender};
use crate::tm::{RunningTuringMachine, SimpleTuringMachineSpec, TapeExtender, TuringMachineSpec};
use crate::utm::{MyUtmEncodingScheme, State, Symbol, UtmEncodingScheme};

pub type UtmTm<'a> = RunningTuringMachine<'a, SimpleTuringMachineSpec<State, Symbol>>;

pub struct TowerLevel<'a> {
    pub machine: UtmTm<'a>,
    pub max_head_pos: usize,
    prev_state: Option<State>,
}

impl<'a> TowerLevel<'a> {
    pub fn new(machine: UtmTm<'a>) -> Self {
        let max_head_pos = machine.pos;
        Self {
            machine,
            max_head_pos,
            prev_state: None,
        }
    }

    pub fn update_machine(&mut self, machine: UtmTm<'a>) {
        self.machine = machine;
        if self.machine.pos > self.max_head_pos {
            self.max_head_pos = self.machine.pos;
        }
    }

    pub fn snapshot_state(&mut self) {
        self.prev_state = Some(self.machine.state);
    }

    pub fn entered_init(&self) -> bool {
        match self.prev_state {
            Some(prev) => self.machine.state == State::Init && prev != State::Init,
            None => false,
        }
    }
}

#[derive(Serialize)]
pub struct TowerLevelJson {
    pub tape: String,
    pub head_pos: usize,
    pub state: String,
    pub tape_len: usize,
}

#[derive(Serialize)]
pub struct TowerJson {
    pub steps: u64,
    pub guest_steps: u64,
    pub steps_per_sec: f64,
    pub tower: Vec<TowerLevelJson>,
}

fn tape_string(tm: &UtmTm, end: usize) -> String {
    let mut out = String::new();
    let blank = tm.spec.blank();
    for i in 0..end {
        let sym = if i < tm.tape.len() { tm.tape[i] } else { blank };
        write!(out, "{}", sym).unwrap();
    }
    if end < tm.tape.len() {
        out.push_str(" ...");
    }
    out
}

fn level_to_json(tm: &UtmTm, end: usize) -> TowerLevelJson {
    TowerLevelJson {
        tape: tape_string(tm, end),
        head_pos: tm.pos,
        state: format!("{:?}", tm.state),
        tape_len: tm.tape.len(),
    }
}

pub fn tower_to_json<'a>(
    tower: &mut [TowerLevel<'a>],
    total_steps: u64,
    guest_steps: u64,
    steps_per_sec: f64,
    utm: &'a SimpleTuringMachineSpec<State, Symbol>,
    extender: &mut InfiniteTapeExtender,
) -> TowerJson {
    let mut levels: Vec<TowerLevelJson> = tower
        .iter()
        .map(|tl| level_to_json(&tl.machine, tl.max_head_pos + 10))
        .collect();

    // Decode and include one more level beyond the tower.
    let last = tower.last_mut().unwrap();
    if let Some(extra) = decode_next_level(utm, &mut last.machine, extender) {
        levels.push(level_to_json(&extra, extra.pos + 10));
    }

    TowerJson {
        steps: total_steps,
        guest_steps,
        steps_per_sec,
        tower: levels,
    }
}

/// Format a tape slice as plain text. Shows tape[0..end].
pub fn tape_view_range(tm: &UtmTm, end: usize) -> String {
    let mut out = String::from("    ");
    let blank = tm.spec.blank();

    for i in 0..end {
        let sym = if i < tm.tape.len() {
            tm.tape[i]
        } else {
            blank
        };
        write!(out, "{}", sym).unwrap();
    }

    if end < tm.tape.len() {
        out.push_str(" ...");
    }
    write!(out, " (state={:?}, pos={})", tm.state, tm.pos).unwrap();
    out
}

/// Add ANSI color codes to tower output.
/// Light red background on head cells, light green on *, X, Y, ^, >.
pub fn colorize_ansi(plain: &str) -> String {
    let mut out = String::with_capacity(plain.len() * 2);
    for line in plain.lines() {
        // Parse head position from "(state=..., pos=N)" suffix
        let head_col = parse_head_col(line);
        for (i, ch) in line.char_indices() {
            if Some(i) == head_col {
                write!(out, "\x1b[101m{}\x1b[0m", ch).unwrap();
            } else if matches!(ch, '*' | 'X' | 'Y' | '^' | '>') {
                write!(out, "\x1b[102m{}\x1b[0m", ch).unwrap();
            } else {
                out.push(ch);
            }
        }
        out.push('\n');
    }
    out
}

/// Find the character column of the head cell in a tape_view_range line.
/// Lines start with "    " (4 spaces), then tape symbols, so head is at column 4 + pos.
fn parse_head_col(line: &str) -> Option<usize> {
    let marker = "pos=";
    let pos_start = line.rfind(marker)?;
    let after = &line[pos_start + marker.len()..];
    let end = after.find(')')?;
    let pos: usize = after[..end].parse().ok()?;
    // The tape starts at column 4 (the "    " prefix)
    Some(4 + pos)
}

/// Decode the next level from a parent machine, extending the tape as needed.
/// Returns None if decoding fails (tape too short, etc.)
pub fn decode_next_level<'a>(
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
pub fn update_tower<'a>(
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

pub fn format_tower<'a>(
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
    }

    buf
}
