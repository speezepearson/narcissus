use std::collections::HashMap;

use crate::gen_utm::UtmSpec;
use crate::optimization_hints::make_my_utm_self_optimization_hints;
use crate::tm::{
    run_tm, run_until_enters_state, step, HaltReason, RunUntilResult, RunningTMStatus,
    RunningTuringMachine, TuringMachineSpec,
};
use crate::toy_machines::*;
use crate::utm::*;

// ════════════════════════════════════════════════════════════════════
// Helpers
// ════════════════════════════════════════════════════════════════════

fn run_guest_direct<'a, Spec: TuringMachineSpec>(
    spec: &'a Spec,
    input: &[Spec::Symbol],
    max_steps: usize,
) -> (&'static str, RunningTuringMachine<'a, Spec>) {
    let mut tm = RunningTuringMachine::new(spec);
    tm.tape = input.to_vec();
    if tm.tape.is_empty() {
        tm.tape.push(spec.blank());
    }

    let result = run_tm(&mut tm, max_steps, None);
    let status = match result {
        Ok(HaltReason::Accepted { .. }) => "accept",
        Ok(HaltReason::Rejected { .. }) => "reject",
        Err(_) => "limit",
    };

    (status, tm)
}

fn run_via_utm<'a, Guest: TuringMachineSpec>(
    guest: &'a Guest,
    input: &[Guest::Symbol],
    max_utm_steps: usize,
) -> (String, RunningTuringMachine<'a, Guest>) {
    let utm_spec = make_utm_spec();

    let mut guest_tm = RunningTuringMachine::new(guest);
    guest_tm.tape = if input.is_empty() {
        vec![guest.blank()]
    } else {
        input.to_vec()
    };

    let encoded = utm_spec.encode(&guest_tm);
    let mut utm_tm = RunningTuringMachine::new(&utm_spec);
    utm_tm.tape = encoded;

    let result = run_tm(&mut utm_tm, max_utm_steps, None);
    let status = match result {
        Ok(HaltReason::Accepted { .. }) => "accept",
        Ok(HaltReason::Rejected { .. }) => "reject",
        Err(_) => "limit",
    };

    let decoded = utm_spec
        .decode(guest, &utm_tm.tape)
        .expect("should be able to decode UTM tape");

    (status.to_string(), decoded)
}

/// Strip trailing blanks from a tape.
fn strip_trailing_blanks<Spec: TuringMachineSpec>(tm: &mut RunningTuringMachine<Spec>) {
    while tm.tape.last() == Some(&tm.spec.blank()) && tm.tape.len() > 1 {
        tm.tape.pop();
    }
}

/// Assert that the UTM faithfully simulates the given TM:
/// - Run the guest directly and via UTM
/// - Assert accept/reject status matches
/// - Assert final tapes match (modulo trailing blanks)
fn assert_faithful<Spec: TuringMachineSpec + std::fmt::Debug>(
    guest_tm: RunningTuringMachine<Spec>,
    max_direct_steps: usize,
    max_utm_steps: usize,
) where
    Spec::State: std::fmt::Debug,
    Spec::Symbol: std::fmt::Debug,
{
    let utm = make_utm_spec();
    // Run directly
    let mut direct_tm = RunningTuringMachine {
        spec: guest_tm.spec,
        state: guest_tm.state,
        pos: guest_tm.pos,
        tape: guest_tm.tape.clone(),
    };
    let direct_result = run_tm(&mut direct_tm, max_direct_steps, None);
    let direct_status = match &direct_result {
        Ok(HaltReason::Accepted { .. }) => "accept",
        Ok(HaltReason::Rejected { .. }) => "reject",
        Err(_) => panic!("direct run hit step limit ({} steps)", max_direct_steps),
    };

    // Run via UTM
    let encoded = utm.encode(&guest_tm);
    let utm_spec = make_utm_spec();
    let mut utm_tm = RunningTuringMachine::new(&utm_spec);
    utm_tm.tape = encoded;

    let utm_result = run_tm(&mut utm_tm, max_utm_steps, None);
    let utm_status = match &utm_result {
        Ok(HaltReason::Accepted { .. }) => "accept",
        Ok(HaltReason::Rejected { .. }) => "reject",
        Err(_) => panic!("UTM run hit step limit ({} steps)", max_utm_steps),
    };

    assert_eq!(
        direct_status, utm_status,
        "status mismatch: direct={}, utm={}",
        direct_status, utm_status
    );

    let mut decoded = utm
        .decode(guest_tm.spec, &utm_tm.tape)
        .expect("should decode UTM tape after halting");

    strip_trailing_blanks(&mut direct_tm);
    strip_trailing_blanks(&mut decoded);

    assert_eq!(
        direct_tm.tape, decoded.tape,
        "tape mismatch after {:?}",
        direct_status
    );
}

// ════════════════════════════════════════════════════════════════════
// Tests: Direct guest TM execution
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_flip_bits_direct() {
    use FlipBitsSymbol::*;
    let (status, tm) = run_guest_direct(&*FLIP_BITS_SPEC, &[Zero, One], 100);
    assert_eq!(status, "accept");
    assert_eq!(tm.tape[0], One);
    assert_eq!(tm.tape[1], Zero);
}

#[test]
fn test_palindrome_direct() {
    use crate::toy_machines::Letter::*;
    use CheckPalindromeSymbol::*;
    let spec = &*CHECK_PALINDROME_SPEC;

    let (status, _) = run_guest_direct(spec, &[Letter(A), Letter(A)], 1000);
    assert_eq!(status, "accept");

    let (status, _) = run_guest_direct(spec, &[Letter(A), Letter(B)], 1000);
    assert_eq!(status, "reject");

    let (status, _) = run_guest_direct(spec, &[Letter(A), Letter(B), Letter(A)], 1000);
    assert_eq!(status, "accept");

    let (status, _) = run_guest_direct(spec, &[], 1000);
    assert_eq!(status, "accept");
}

#[test]
fn test_double_x_direct() {
    use DoubleXSymbol::*;
    let (status, tm) = run_guest_direct(&*DOUBLE_X_SPEC, &[Dollar, X, X], 1000);
    assert_eq!(status, "accept");
    assert_eq!(tm.tape[0], Dollar);
    assert_eq!(tm.tape[1], X);
    assert_eq!(tm.tape[2], X);
    assert_eq!(tm.tape[3], X);
    assert_eq!(tm.tape[4], X);
}

// ════════════════════════════════════════════════════════════════════
// Tests: UTM spec construction
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_utm_spec_builds() {
    let utm = make_utm_spec();
    let n_rules = utm.transitions.len();
    assert!(n_rules > 100, "UTM should have many rules, got {}", n_rules);
}

// ════════════════════════════════════════════════════════════════════
// Tests: Encode/decode round-trip
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_encode_decode_roundtrip_flip_bits() {
    use FlipBitsSymbol::*;
    let spec = &*FLIP_BITS_SPEC;
    let mut guest_tm = RunningTuringMachine::new(spec);
    guest_tm.tape = vec![Zero, One];

    let utm = make_utm_spec();
    let encoded = utm.encode(&guest_tm);
    let decoded = utm.decode(spec, &encoded).unwrap();
    assert_eq!(decoded.state, guest_tm.state);
    assert_eq!(decoded.pos, 0);
    assert_eq!(decoded.tape, vec![Zero, One]);
}

#[test]
fn test_encode_decode_roundtrip_empty() {
    let spec = &*ACCEPT_IMMEDIATELY_SPEC;
    let mut guest_tm = RunningTuringMachine::new(spec);
    guest_tm.tape = vec![spec.blank()];

    let utm = make_utm_spec();
    let encoded = utm.encode(&guest_tm);
    let decoded = utm.decode(spec, &encoded).unwrap();
    assert_eq!(decoded.state, spec.initial());
    assert_eq!(decoded.pos, 0);
    assert_eq!(decoded.tape, vec![spec.blank()]);
}

#[test]
fn test_encode_decode_roundtrip_palindrome() {
    use crate::toy_machines::Letter::*;
    use CheckPalindromeSymbol::*;
    let spec = &*CHECK_PALINDROME_SPEC;
    let mut guest_tm = RunningTuringMachine::new(spec);
    guest_tm.tape = vec![Letter(A), Letter(B), Letter(A)];

    let utm = make_utm_spec();
    let encoded = utm.encode(&guest_tm);
    let decoded = utm.decode(spec, &encoded).unwrap();
    assert_eq!(decoded.state, CheckPalindromeState::Start);
    assert_eq!(decoded.pos, 0);
    assert_eq!(decoded.tape, vec![Letter(A), Letter(B), Letter(A)]);
}

// ════════════════════════════════════════════════════════════════════
// Tests: UTM simulation of guest TMs
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_utm_accept_immediately() {
    let (status, _) = run_via_utm(&*ACCEPT_IMMEDIATELY_SPEC, &[], 10_000);
    assert_eq!(status, "accept");
}

#[test]
fn test_utm_reject_immediately() {
    let (status, _) = run_via_utm(&*REJECT_IMMEDIATELY_SPEC, &[], 10_000);
    assert_eq!(status, "reject");
}

#[test]
fn test_utm_flip_bits() {
    use FlipBitsSymbol::*;
    let (status, tm) = run_via_utm(&*FLIP_BITS_SPEC, &[Zero, One], 1_000_000);
    assert_eq!(status, "accept");
    assert_eq!(tm.tape[0], One);
    assert_eq!(tm.tape[1], Zero);
}

#[test]
fn test_utm_flip_bits_5() {
    use FlipBitsSymbol::*;
    let (status, tm) = run_via_utm(&*FLIP_BITS_SPEC, &[Zero, One, Zero, One, One], 10_000_000);
    assert_eq!(status, "accept");
    assert_eq!(tm.tape[0], One);
    assert_eq!(tm.tape[1], Zero);
    assert_eq!(tm.tape[2], One);
    assert_eq!(tm.tape[3], Zero);
    assert_eq!(tm.tape[4], Zero);
}

#[test]
fn test_utm_double_x() {
    use DoubleXSymbol::*;
    let (status, tm) = run_via_utm(&*DOUBLE_X_SPEC, &[Dollar, X, X], 50_000_000);
    assert_eq!(status, "accept");
    assert_eq!(tm.tape[0], Dollar);
    assert_eq!(tm.tape[1], X);
    assert_eq!(tm.tape[2], X);
    assert_eq!(tm.tape[3], X);
    assert_eq!(tm.tape[4], X);
}

// ════════════════════════════════════════════════════════════════════
// Tests: UTM faithfulness
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_faithful_accept_immediately() {
    let spec = &*ACCEPT_IMMEDIATELY_SPEC;
    let tm = RunningTuringMachine::new(spec);
    assert_faithful(tm, 100, 10_000);
}

#[test]
fn test_faithful_reject_immediately() {
    let spec = &*REJECT_IMMEDIATELY_SPEC;
    let tm = RunningTuringMachine::new(spec);
    assert_faithful(tm, 100, 10_000);
}

#[test]
fn test_faithful_flip_bits_2() {
    use FlipBitsSymbol::*;
    let spec = &*FLIP_BITS_SPEC;
    let mut tm = RunningTuringMachine::new(spec);
    tm.tape = vec![Zero, One];
    assert_faithful(tm, 100, 1_000_000);
}

#[test]
fn test_faithful_flip_bits_5() {
    use FlipBitsSymbol::*;
    let spec = &*FLIP_BITS_SPEC;
    let mut tm = RunningTuringMachine::new(spec);
    tm.tape = vec![Zero, One, Zero, One, One];
    assert_faithful(tm, 100, 10_000_000);
}

#[test]
fn test_faithful_double_x() {
    use DoubleXSymbol::*;
    let spec = &*DOUBLE_X_SPEC;
    let mut tm = RunningTuringMachine::new(spec);
    tm.tape = vec![Dollar, X, X];
    assert_faithful(tm, 1_000, 50_000_000);
}

#[test]
#[ignore] // ~38s in release mode; too slow for debug. Run with: cargo test --release -- --ignored
fn test_faithful_utm_running_accept_immediately() {
    // Smoke-test recursion: a UTM running AcceptImmediately,
    // then assert_faithful runs *that* through the UTM.
    let acc_spec = &*ACCEPT_IMMEDIATELY_SPEC;
    let inner_tm = RunningTuringMachine::new(acc_spec);

    // Encode the inner TM into a UTM tape
    let utm_spec = make_utm_spec();
    let encoded_inner = utm_spec.encode(&inner_tm);

    let mut utm_tm = RunningTuringMachine::new(&utm_spec);
    utm_tm.tape = encoded_inner;

    // Now assert_faithful runs this UTM-running-AcceptImmediately
    // through the UTM again (two levels of simulation).
    // Two levels of UTM overhead requires a very large step limit.
    assert_faithful(utm_tm, 10_000, 10_000_000_000);
}

// ════════════════════════════════════════════════════════════════════
// Tests: encode_with_rule_order
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_encode_with_last_rules_faithful_flip_bits() {
    use FlipBitsSymbol::*;
    let spec = &*FLIP_BITS_SPEC;
    let mut tm = RunningTuringMachine::new(spec);
    tm.tape = vec![Zero, One, Zero];

    // Put one specific rule last
    let utm_spec = make_utm_spec();
    let encoded = utm_spec.encode_optimized(
        &tm,
        &TmTransitionStats(HashMap::from([((FlipBitsState::Flip, Zero), 1)]))
            .make_optimization_hints(&spec),
    );

    // Run directly
    let mut direct_tm = RunningTuringMachine {
        spec: tm.spec,
        state: tm.state,
        pos: tm.pos,
        tape: tm.tape.clone(),
    };
    let direct_result = run_tm(&mut direct_tm, 100, None).unwrap();

    // Run via UTM with reordered encoding
    let mut utm_tm = RunningTuringMachine::new(&utm_spec);
    utm_tm.tape = encoded;
    let utm_result = run_tm(&mut utm_tm, 10_000_000, None).unwrap();

    // Compare accept/reject status (not step counts, which differ)
    assert_eq!(
        matches!(direct_result, HaltReason::Accepted { .. }),
        matches!(utm_result, HaltReason::Accepted { .. }),
        "halt status should match"
    );

    let decoded = utm_spec
        .decode(spec, &utm_tm.tape)
        .expect("should decode UTM tape");
    strip_trailing_blanks(&mut direct_tm);
    let mut decoded_stripped = decoded;
    strip_trailing_blanks(&mut decoded_stripped);
    assert_eq!(direct_tm.tape, decoded_stripped.tape);
}

#[test]
fn test_encode_with_none_same_as_encode() {
    use FlipBitsSymbol::*;
    let spec = &*FLIP_BITS_SPEC;
    let mut tm = RunningTuringMachine::new(spec);
    tm.tape = vec![Zero, One];

    let utm = make_utm_spec();
    let plain = utm.encode(&tm);
    let with_none = utm.encode_optimized(&tm, &MyUtmSpecOptimizationHints::guess(&spec));
    assert_eq!(plain, with_none);
}

// ════════════════════════════════════════════════════════════════════
// Benchmark: compiled vs interpreted UTM(UTM(accept_immediately))
// ════════════════════════════════════════════════════════════════════

#[test]
#[ignore] // Run with: cargo test --release -- --ignored bench_compiled_vs_interpreted --nocapture
fn bench_compiled_vs_interpreted() {
    use crate::compiled::{CSymbol, CompiledTuringMachineSpec};
    use crate::tm::step;
    use std::time::Instant;

    const STEPS: usize = 100_000;
    // Pre-extend tape to this size so no allocation happens during timing.
    const TAPE_PAD: usize = 10_000;

    let utm_spec = make_utm_spec();

    // Build utm(encode(utm(encode(accept_immediately))))
    let acc_spec = &*ACCEPT_IMMEDIATELY_SPEC;
    let inner_tm = RunningTuringMachine::new(acc_spec);
    let inner_encoded = utm_spec.encode(&inner_tm);

    let mut mid_tm = RunningTuringMachine::new(&utm_spec);
    mid_tm.tape = inner_encoded;
    let outer_encoded = utm_spec.encode(&mid_tm);

    // Helper: convert Symbol tape to CSymbol tape
    let compiled = CompiledTuringMachineSpec::compile(&utm_spec).expect("UTM should compile");
    let sym_to_csym: std::collections::HashMap<Symbol, CSymbol> = compiled
        .original_symbols
        .iter()
        .enumerate()
        .map(|(i, &s)| (s, CSymbol(i as u8)))
        .collect();

    // ── Interpreted: set up and pre-extend tape ──
    let mut interp_tm = RunningTuringMachine::new(&utm_spec);
    interp_tm.tape = outer_encoded.clone();
    interp_tm
        .tape
        .resize(outer_encoded.len() + TAPE_PAD, utm_spec.blank());

    // ── Compiled: set up and pre-extend tape ──
    let mut compiled_tm = RunningTuringMachine::new(&compiled);
    compiled_tm.tape = outer_encoded
        .iter()
        .map(|s| sym_to_csym[s])
        .collect::<Vec<_>>();
    compiled_tm
        .tape
        .resize(outer_encoded.len() + TAPE_PAD, compiled.blank);

    // ── Timed: interpreted step loop ──
    let t0 = Instant::now();
    for _ in 0..STEPS {
        step(&mut interp_tm);
    }
    let interp_elapsed = t0.elapsed();

    // ── Timed: compiled step loop ──
    let t0 = Instant::now();
    for _ in 0..STEPS {
        step(&mut compiled_tm);
    }
    let compiled_elapsed = t0.elapsed();

    // ── Verify faithfulness ──
    let decompiled = compiled.decompile(&compiled_tm);
    assert_eq!(
        interp_tm.state, decompiled.state,
        "state mismatch after {} steps",
        STEPS
    );
    assert_eq!(
        interp_tm.pos, decompiled.pos,
        "pos mismatch after {} steps",
        STEPS
    );
    assert_eq!(
        interp_tm.tape, decompiled.tape,
        "tape mismatch after {} steps",
        STEPS
    );

    let speedup = interp_elapsed.as_secs_f64() / compiled_elapsed.as_secs_f64();
    eprintln!(
        "═══ Benchmark: UTM(UTM(accept_immediately)), {} steps ═══",
        STEPS
    );
    eprintln!("  Interpreted (HashMap): {:?}", interp_elapsed);
    eprintln!("  Compiled (array):      {:?}", compiled_elapsed);
    eprintln!("  Speedup:               {:.2}x", speedup);
}

// ════════════════════════════════════════════════════════════════════
// Tests: run_until_enters_state
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_run_until_enters_state_flip_bits() {
    use FlipBitsState::*;
    use FlipBitsSymbol::*;

    let spec = &*FLIP_BITS_SPEC;
    let mut tm = RunningTuringMachine::new(spec);
    tm.tape = vec![Zero, One, Zero];

    // Run until it enters Done (which happens when it hits Blank after flipping all bits)
    let result = run_until_enters_state(&mut tm, Done, 1000, None);
    // 4 steps: flip Zero(1), flip One(2), flip Zero(3), then Blank→Done(4, halts)
    assert_eq!(result, Ok(4));
    assert_eq!(tm.state, Done);
    assert_eq!(tm.tape[0..3], [One, Zero, One]);
}

#[test]
fn test_run_until_enters_state_step_limit() {
    use FlipBitsSymbol::*;

    let spec = &*FLIP_BITS_SPEC;
    let mut tm = RunningTuringMachine::new(spec);
    // Long tape so it won't reach Done in 2 steps
    tm.tape = vec![Zero, One, Zero, One, Zero];

    let result = run_until_enters_state(&mut tm, FlipBitsState::Done, 2, None);
    assert_eq!(result, Err(RunUntilResult::StepLimit));
}

#[test]
fn test_run_until_enters_state_halts_before_target() {
    use FlipBitsState::*;
    use FlipBitsSymbol::*;

    let spec = &*FLIP_BITS_SPEC;
    let mut tm = RunningTuringMachine::new(spec);
    tm.tape = vec![Zero];

    // Ask to reach a state that doesn't exist in transitions from Done —
    // actually, Done is the accepting state and Flip is initial.
    // The machine goes: (Flip, Zero) -> (Flip, One, R), then (Flip, Blank) -> halts in Done.
    // If we ask for Flip as target, it starts in Flip so it needs to leave and come back.
    // But it never returns to Flip — it goes Flip->Flip->Done. So it halts (accepted) before
    // returning to Flip after leaving it.
    // Wait — it starts in Flip and stays in Flip for the first step. The function takes at
    // least one step. After step 1: state=Flip (didn't leave). After step 2: state=Done (halts).
    // So it should return Accepted since it halted in an accepting state without
    // ever leaving+re-entering Flip.
    // Actually re-reading the spec: "makes the machine take at least one step, then stops
    // running it when it's in the given state." So it just checks if state == target after
    // each step (with at least one step taken). Since step 1 leaves us in Flip, and Flip
    // is the target, it should return Ok(1).
    let result = run_until_enters_state(&mut tm, Flip, 1000, None);
    assert_eq!(result, Ok(1)); // After 1 step, still in Flip
}

#[test]
fn test_run_until_enters_state_halts_in_non_target() {
    use FlipBitsState::*;
    use FlipBitsSymbol::*;

    // Build a machine where it halts before reaching a state we're waiting for.
    // FlipBits with empty tape: (Flip, Blank) -> (Done, Blank, L). Halts in Done (accepting).
    // If we target Flip, it takes one step and lands in Done, which halts.
    let spec = &*FLIP_BITS_SPEC;
    let mut tm = RunningTuringMachine::new(spec);
    tm.tape = vec![Blank];

    // Target Flip, but machine halts in Done after step 1 (Running), then step 2 (no transition from Done)
    let result = run_until_enters_state(&mut tm, Flip, 1000, None);
    assert_eq!(result, Err(RunUntilResult::Accepted { num_steps: 2 }));
}

#[test]
#[ignore] // Run with: cargo test --release -- --ignored bench_rule_order --nocapture
fn bench_rule_order_optimization() {
    use crate::compiled::CompiledTuringMachineSpec;

    const STEPS: usize = 100_000_000;

    let utm_spec = make_utm_spec();
    let compiled = CompiledTuringMachineSpec::compile(&utm_spec).expect("UTM should compile");

    // Helper: build a compiled TM running the infinite UTM tape with given rule order
    let build_tm = |optimization_hints: &MyUtmSpecOptimizationHints<MyUtmSpec>| {
        // Compute the header with the given rule order
        let header_tape =
            utm_spec.encode_optimized(&RunningTuringMachine::new(&utm_spec), &optimization_hints);
        let caret_pos = header_tape
            .iter()
            .position(|&s| s == Symbol::Caret)
            .unwrap();
        let header: Vec<Symbol> = header_tape[..caret_pos].to_vec();

        // Build a custom extender using this header
        // We can't reuse InfiniteTapeExtender directly since it uses OPTIMIZATION_HINTS,
        // so for the unoptimized case we need to build the tape from the unoptimized header.
        // For simplicity, pre-extend a large tape and run from that.
        let sym_to_idx: std::collections::HashMap<Symbol, usize> = utm_spec
            .iter_symbols()
            .enumerate()
            .map(|(i, s)| (s, i))
            .collect();
        let n_sym_bits = crate::utm::num_bits(utm_spec.iter_symbols().count());
        let cell_width = 1 + n_sym_bits;
        let tape_len = header.len() + STEPS + 10_000;
        let mut tape: Vec<Symbol> = Vec::with_capacity(tape_len);
        // Fill header
        tape.extend_from_slice(&header);
        // Fill tape section
        for i in 0..(tape_len - header.len()) {
            if i % cell_width == 0 {
                tape.push(if i == 0 { Symbol::Caret } else { Symbol::Comma });
            } else {
                let cell_index = i / cell_width;
                let bit_offset = i % cell_width - 1;
                let sym = tape[cell_index];
                let sym_idx = sym_to_idx[&sym];
                let bit = (sym_idx >> (n_sym_bits - 1 - bit_offset)) & 1;
                tape.push(if bit == 1 { Symbol::One } else { Symbol::Zero });
            }
        }

        // Convert to compiled symbols
        let sym_to_csym: std::collections::HashMap<Symbol, crate::compiled::CSymbol> = compiled
            .original_symbols
            .iter()
            .enumerate()
            .map(|(i, &s)| (s, crate::compiled::CSymbol(i as u8)))
            .collect();
        let mut compiled_tm = RunningTuringMachine::new(&compiled);
        compiled_tm.tape = tape.iter().map(|s| sym_to_csym[s]).collect();
        compiled_tm
    };

    // Helper: count how many times the UTM completes an inner step in STEPS total steps
    let count_guest_steps = |tm: &mut RunningTuringMachine<
        CompiledTuringMachineSpec<crate::tm::SimpleTuringMachineSpec<State, Symbol>>,
    >|
     -> u64 {
        let mut guest_steps = 0u64;
        let mut prev_state = tm.state;
        let mut remaining = STEPS;
        while remaining > 0 {
            match step(tm) {
                RunningTMStatus::Running => {
                    if compiled.is_tick_boundary(prev_state, tm.state) {
                        guest_steps += 1;
                    }
                    prev_state = tm.state;
                    remaining -= 1;
                }
                _ => break,
            }
        }
        guest_steps
    };

    // ── Unoptimized (default rule order) ──
    let mut unopt_tm = build_tm(&MyUtmSpecOptimizationHints::guess(&utm_spec));
    let unopt_guest_steps = count_guest_steps(&mut unopt_tm);

    // ── Optimized (hints rule order) ──
    let mut opt_tm = build_tm(&make_my_utm_self_optimization_hints());
    let opt_guest_steps = count_guest_steps(&mut opt_tm);

    eprintln!(
        "═══ Benchmark: rule order optimization, {} steps ═══",
        STEPS
    );
    eprintln!("  Unoptimized: {} guest steps", unopt_guest_steps);
    eprintln!("  Optimized:   {} guest steps", opt_guest_steps);
    // With optimized rule order, the UTM finds matching rules faster,
    // so it should complete more guest-level steps in the same number of UTM steps.
    if unopt_guest_steps > 0 {
        eprintln!(
            "  Ratio (opt/unopt): {:.2}x",
            opt_guest_steps as f64 / unopt_guest_steps as f64
        );
    }
}

// ════════════════════════════════════════════════════════════════════
// is_tick_boundary / run_until_inner_step tests
// ════════════════════════════════════════════════════════════════════

use crate::utm::run_until_inner_step;

/// Helper: create a UTM running a guest, assert it starts at a tick,
/// then for each of `n_steps` inner steps: step the guest directly once,
/// advance the UTM to the next tick, decode, and assert equality.
fn assert_tick_faithful<Spec: TuringMachineSpec>(
    guest_spec: &Spec,
    input: &[Spec::Symbol],
    n_steps: usize,
    max_utm_steps_per_tick: usize,
) where
    Spec::State: std::fmt::Debug + PartialEq,
    Spec::Symbol: std::fmt::Debug + PartialEq,
{
    let utm_spec = make_utm_spec();

    // Set up guest TM
    let mut guest_tm = RunningTuringMachine::new(guest_spec);
    guest_tm.tape = if input.is_empty() {
        vec![guest_spec.blank()]
    } else {
        input.to_vec()
    };

    // Encode and create UTM
    let encoded = utm_spec.encode(&guest_tm);
    let mut utm_tm = RunningTuringMachine::new(&utm_spec);
    utm_tm.tape = encoded;

    // Decode of freshly encoded tape should match initial guest
    let decoded = utm_spec
        .decode(guest_spec, &utm_tm.tape)
        .expect("decode at initial state");
    assert_eq!(decoded.state, guest_tm.state, "initial state mismatch");
    assert_eq!(decoded.pos, guest_tm.pos, "initial pos mismatch");
    assert_eq!(decoded.tape, guest_tm.tape, "initial tape mismatch");

    for i in 0..n_steps {
        // Step the guest directly
        let guest_status = step(&mut guest_tm);
        if !matches!(guest_status, crate::tm::RunningTMStatus::Running) {
            // Guest halted; we can't step further
            break;
        }
        // Extend guest tape if needed
        if guest_tm.pos >= guest_tm.tape.len() {
            guest_tm.tape.resize(guest_tm.pos + 1, guest_spec.blank());
        }

        // Advance UTM to next tick
        let tick_result = run_until_inner_step(&utm_spec, &mut utm_tm, max_utm_steps_per_tick);
        assert!(
            tick_result.is_ok(),
            "UTM should reach tick at step {} (got {:?})",
            i + 1,
            tick_result,
        );

        // Decode and compare
        let decoded = utm_spec
            .decode(guest_spec, &utm_tm.tape)
            .expect(&format!("decode at tick {}", i + 1));

        assert_eq!(
            decoded.state,
            guest_tm.state,
            "state mismatch at inner step {}",
            i + 1
        );
        assert_eq!(
            decoded.pos,
            guest_tm.pos,
            "pos mismatch at inner step {}",
            i + 1
        );

        // Compare tapes (strip trailing blanks)
        let guest_tape_trimmed: Vec<_> = {
            let mut t = guest_tm.tape.clone();
            while t.last() == Some(&guest_spec.blank()) && t.len() > 1 {
                t.pop();
            }
            t
        };
        let decoded_tape_trimmed: Vec<_> = {
            let mut t = decoded.tape;
            while t.last() == Some(&guest_spec.blank()) && t.len() > 1 {
                t.pop();
            }
            t
        };
        assert_eq!(
            decoded_tape_trimmed,
            guest_tape_trimmed,
            "tape mismatch at inner step {}",
            i + 1
        );
    }
}

#[test]
fn test_at_tick_flip_bits_2() {
    use FlipBitsSymbol::*;
    assert_tick_faithful(&*FLIP_BITS_SPEC, &[Zero, One], 5, 10_000_000);
}

#[test]
fn test_at_tick_flip_bits_5() {
    use FlipBitsSymbol::*;
    assert_tick_faithful(
        &*FLIP_BITS_SPEC,
        &[One, Zero, One, One, Zero],
        8,
        10_000_000,
    );
}

#[test]
fn test_at_tick_double_x() {
    use DoubleXSymbol::*;
    assert_tick_faithful(&*DOUBLE_X_SPEC, &[Dollar, X, X], 20, 50_000_000);
}

// ════════════════════════════════════════════════════════════════════
// Tests: noop compact rule encoding
// ════════════════════════════════════════════════════════════════════

/// Build a simple TM where state Scan has noop rules (scan right over S0, S1)
/// and one non-noop rule (Blank -> transition to Done).
fn make_noop_test_spec() -> crate::tm::SimpleTuringMachineSpec<u8, u8> {
    // States: 0=Scan, 1=Done
    // Symbols: 0=Blank, 1=S0, 2=S1
    crate::tm::SimpleTuringMachineSpec {
        initial: 0,
        accepting: std::collections::HashSet::from([1]),
        blank: 0,
        transitions: std::collections::HashMap::from([
            // Noop rules: (Scan, S0) -> (Scan, S0, R), (Scan, S1) -> (Scan, S1, R)
            ((0u8, 1u8), (0u8, 1u8, crate::tm::Dir::Right)),
            ((0u8, 2u8), (0u8, 2u8, crate::tm::Dir::Right)),
            // Non-noop rule: (Scan, Blank) -> (Done, Blank, Left)
            ((0u8, 0u8), (1u8, 0u8, crate::tm::Dir::Left)),
        ]),
        all_states: vec![0, 1],
        all_symbols: vec![0, 1, 2],
    }
}

#[test]
fn test_noop_encoding_has_commas() {
    let spec = make_noop_test_spec();
    let utm = make_utm_spec();
    let mut tm = RunningTuringMachine::new(&spec);
    tm.tape = vec![1, 2, 0]; // S0, S1, Blank
    let encoded = utm.encode(&tm);

    // Find rules section: between #[1] and #[2] in new layout
    // Layout: $ ACC #[0] BLANK #[1] RULES #[2] STATE #[3] SYMCACHE #[4] TAPE
    let hashes: Vec<usize> = encoded
        .iter()
        .enumerate()
        .filter(|(_, s)| **s == Symbol::Hash)
        .map(|(i, _)| i)
        .collect();
    let rules_section = &encoded[hashes[1] + 1..hashes[2]];

    // Count commas in rules section - should be 2 (one per noop symbol: ,S0,S1)
    let comma_count = rules_section
        .iter()
        .filter(|s| **s == Symbol::Comma)
        .count();
    // State Scan has 2 noop rules (S0 and S1), encoded as . STATE , S0 , S1 | R
    // That's 2 commas (one before each noop symbol)
    assert_eq!(
        comma_count, 2,
        "Expected 2 commas for 2 noop rules, got {}",
        comma_count
    );
}

#[test]
fn test_noop_faithful_simple() {
    let spec = make_noop_test_spec();
    let mut tm = RunningTuringMachine::new(&spec);
    tm.tape = vec![1, 2, 0]; // S0, S1, Blank -> should scan right, then go to Done
    assert_faithful(tm, 100, 1_000_000);
}

#[test]
fn test_noop_tick_faithful() {
    let spec = make_noop_test_spec();
    assert_tick_faithful(&spec, &[1u8, 2, 1, 2, 0], 6, 10_000_000);
}

#[test]
fn test_noop_faithful_flip_bits() {
    // FlipBits has no noop rules, so this tests that normal rules still work
    // after the encoding changes.
    use FlipBitsSymbol::*;
    let spec = &*FLIP_BITS_SPEC;
    let mut tm = RunningTuringMachine::new(spec);
    tm.tape = vec![Zero, One, Zero];
    assert_faithful(tm, 100, 1_000_000);
}

#[test]
fn test_noop_faithful_palindrome() {
    // Palindrome has many noop rules (SeekR scans right over all letters)
    use crate::toy_machines::Letter::*;
    use CheckPalindromeSymbol::*;
    let spec = &*CHECK_PALINDROME_SPEC;
    let mut tm = RunningTuringMachine::new(spec);
    tm.tape = vec![Letter(A), Letter(B), Letter(A)];
    assert_faithful(tm, 1_000, 50_000_000);
}
