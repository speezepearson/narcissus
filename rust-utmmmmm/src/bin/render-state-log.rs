use std::io::{Read, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input = get_flag(&args, "--input").unwrap_or_else(|| {
        eprintln!("Usage: render-state-log --input PATH --output PATH [--width N] [--height N]");
        std::process::exit(1);
    });
    let output = get_flag(&args, "--output").unwrap_or_else(|| {
        eprintln!("Usage: render-state-log --input PATH --output PATH [--width N] [--height N]");
        std::process::exit(1);
    });
    let width: usize = get_flag(&args, "--width")
        .map(|s| s.parse().expect("--width must be a number"))
        .unwrap_or(1920);
    let height: usize = get_flag(&args, "--height")
        .map(|s| s.parse().expect("--height must be a number"))
        .unwrap_or(1080);

    let mut file = std::fs::File::open(&input).expect("open state-log");

    // ── Read header ──
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic).expect("read magic");
    assert_eq!(&magic, b"SLOG", "not a SLOG file");

    let mut buf2 = [0u8; 2];
    file.read_exact(&mut buf2).expect("read num_states");
    let num_states = u16::from_le_bytes(buf2) as usize;

    let mut state_names: Vec<String> = Vec::with_capacity(num_states);
    for _ in 0..num_states {
        file.read_exact(&mut buf2).expect("read name_len");
        let name_len = u16::from_le_bytes(buf2) as usize;
        let mut name_buf = vec![0u8; name_len];
        file.read_exact(&mut name_buf).expect("read name");
        state_names.push(String::from_utf8(name_buf).expect("state name utf8"));
    }
    eprintln!("Read {} state names", state_names.len());

    // ── First pass: count records, find ranges ──
    let mut record_buf = [0u8; 17]; // u64 steps + u8 state + u64 pos
    let mut min_pos: u64 = u64::MAX;
    let mut max_pos: u64 = 0;
    let mut min_steps: u64 = u64::MAX;
    let mut max_steps: u64 = 0;
    let mut num_records: u64 = 0;

    loop {
        match file.read_exact(&mut record_buf) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => panic!("read error: {}", e),
        }
        let steps = u64::from_le_bytes(record_buf[0..8].try_into().unwrap());
        let pos = u64::from_le_bytes(record_buf[9..17].try_into().unwrap());
        min_pos = min_pos.min(pos);
        max_pos = max_pos.max(pos);
        min_steps = min_steps.min(steps);
        max_steps = max_steps.max(steps);
        num_records += 1;
    }

    if num_records == 0 {
        eprintln!("No records found");
        return;
    }

    eprintln!(
        "{} records, steps [{}, {}], pos [{}, {}]",
        num_records, min_steps, max_steps, min_pos, max_pos
    );

    // ── Second pass: bin into image ──
    let pos_range = (max_pos - min_pos + 1) as f64;
    let step_range = (max_steps - min_steps + 1) as f64;

    // Image buffer: RGB
    let mut pixels = vec![0u8; width * height * 3];

    // Re-read records
    let mut file = std::fs::File::open(&input).expect("reopen");
    // Skip header
    file.read_exact(&mut [0u8; 4]).unwrap(); // magic
    file.read_exact(&mut buf2).unwrap(); // num_states
    let ns = u16::from_le_bytes(buf2) as usize;
    for _ in 0..ns {
        file.read_exact(&mut buf2).unwrap();
        let nl = u16::from_le_bytes(buf2) as usize;
        let mut skip = vec![0u8; nl];
        file.read_exact(&mut skip).unwrap();
    }

    // Color palette for states (up to 256)
    let palette = build_palette(num_states);

    loop {
        match file.read_exact(&mut record_buf) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => panic!("read error: {}", e),
        }
        let steps = u64::from_le_bytes(record_buf[0..8].try_into().unwrap());
        let state_idx = record_buf[8] as usize;
        let pos = u64::from_le_bytes(record_buf[9..17].try_into().unwrap());

        let x = ((pos - min_pos) as f64 / pos_range * width as f64) as usize;
        let y = ((steps - min_steps) as f64 / step_range * height as f64) as usize;
        let x = x.min(width - 1);
        let y = y.min(height - 1);

        let color = &palette[state_idx % palette.len()];
        let idx = (y * width + x) * 3;
        pixels[idx] = color[0];
        pixels[idx + 1] = color[1];
        pixels[idx + 2] = color[2];
    }

    // ── Write PPM ──
    let mut out = std::io::BufWriter::new(std::fs::File::create(&output).expect("create output"));
    write!(out, "P6\n{} {}\n255\n", width, height).unwrap();
    out.write_all(&pixels).unwrap();
    drop(out);

    eprintln!("Wrote {}x{} PPM to {}", width, height, output);

    // ── Print legend ──
    eprintln!("\nState color legend:");
    for (i, name) in state_names.iter().enumerate() {
        let c = &palette[i % palette.len()];
        eprintln!("  {:3}: #{:02x}{:02x}{:02x} {}", i, c[0], c[1], c[2], name);
    }
}

fn get_flag(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .map(|i| args[i + 1].clone())
}

fn build_palette(num_states: usize) -> Vec<[u8; 3]> {
    let mut palette = Vec::with_capacity(num_states);
    for i in 0..num_states {
        // Use golden-ratio hue spacing for distinct colors
        let hue = (i as f64 * 0.618033988749895) % 1.0;
        let (r, g, b) = hsv_to_rgb(hue, 0.8, 0.9);
        palette.push([(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]);
    }
    palette
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f64;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        5 => (v, p, q),
        _ => unreachable!(),
    }
}
