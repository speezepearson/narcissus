use crate::tm::{run_tm, HaltReason, RunningTuringMachine, TuringMachineSpec};
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
    let utm = &*UTM_SPEC;

    let mut guest_tm = RunningTuringMachine::new(guest);
    guest_tm.tape = if input.is_empty() {
        vec![guest.blank()]
    } else {
        input.to_vec()
    };

    let encoded = MyUtmEncodingScheme::encode(&guest_tm);
    let mut utm_tm = RunningTuringMachine::new(utm);
    utm_tm.tape = encoded;

    let result = run_tm(&mut utm_tm, max_utm_steps, None);
    let status = match result {
        Ok(HaltReason::Accepted { .. }) => "accept",
        Ok(HaltReason::Rejected { .. }) => "reject",
        Err(_) => "limit",
    };

    let decoded = MyUtmEncodingScheme::decode(guest, &utm_tm.tape)
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
    mut guest_tm: RunningTuringMachine<Spec>,
    max_direct_steps: usize,
    max_utm_steps: usize,
) where
    Spec::State: std::fmt::Debug,
    Spec::Symbol: std::fmt::Debug,
{
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
    let encoded = MyUtmEncodingScheme::encode(&guest_tm);
    let utm = &*UTM_SPEC;
    let mut utm_tm = RunningTuringMachine::new(utm);
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

    let mut decoded = MyUtmEncodingScheme::decode(guest_tm.spec, &utm_tm.tape)
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
fn test_accept_immediately() {
    use AccImmSymbol::*;
    // This spec accepts when it encounters a symbol with no transition.
    // (Init, One) has no rule, so it halts in the accepting state Init.
    let (status, _) = run_guest_direct(&*ACCEPT_IMMEDIATELY_SPEC, &[One], 100);
    assert_eq!(status, "accept");
}

#[test]
fn test_reject_immediately() {
    use RejImmSymbol::*;
    // Same structure but Init is not in the accepting set.
    let (status, _) = run_guest_direct(&*REJECT_IMMEDIATELY_SPEC, &[One], 100);
    assert_eq!(status, "reject");
}

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
    use CheckPalindromeSymbol::*;
    let spec = &*CHECK_PALINDROME_SPEC;

    let (status, _) = run_guest_direct(spec, &[A, A], 1000);
    assert_eq!(status, "accept");

    let (status, _) = run_guest_direct(spec, &[A, B], 1000);
    assert_eq!(status, "reject");

    let (status, _) = run_guest_direct(spec, &[A, B, A], 1000);
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
    let utm = &*UTM_SPEC;
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

    let encoded = MyUtmEncodingScheme::encode(&guest_tm);
    let decoded = MyUtmEncodingScheme::decode(spec, &encoded).unwrap();
    assert_eq!(decoded.state, guest_tm.state);
    assert_eq!(decoded.pos, 0);
    assert_eq!(decoded.tape, vec![Zero, One]);
}

#[test]
fn test_encode_decode_roundtrip_empty() {
    let spec = &*ACCEPT_IMMEDIATELY_SPEC;
    let mut guest_tm = RunningTuringMachine::new(spec);
    guest_tm.tape = vec![spec.blank()];

    let encoded = MyUtmEncodingScheme::encode(&guest_tm);
    let decoded = MyUtmEncodingScheme::decode(spec, &encoded).unwrap();
    assert_eq!(decoded.state, spec.initial());
    assert_eq!(decoded.pos, 0);
    assert_eq!(decoded.tape, vec![spec.blank()]);
}

#[test]
fn test_encode_decode_roundtrip_palindrome() {
    use CheckPalindromeSymbol::*;
    let spec = &*CHECK_PALINDROME_SPEC;
    let mut guest_tm = RunningTuringMachine::new(spec);
    guest_tm.tape = vec![A, B, A];

    let encoded = MyUtmEncodingScheme::encode(&guest_tm);
    let decoded = MyUtmEncodingScheme::decode(spec, &encoded).unwrap();
    assert_eq!(decoded.state, CheckPalindromeState::Start);
    assert_eq!(decoded.pos, 0);
    assert_eq!(decoded.tape, vec![A, B, A]);
}

// ════════════════════════════════════════════════════════════════════
// Tests: UTM simulation of guest TMs
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_utm_accept_immediately() {
    use AccImmSymbol::*;
    let (status, _) = run_via_utm(&*ACCEPT_IMMEDIATELY_SPEC, &[One], 10_000);
    assert_eq!(status, "accept");
}

#[test]
fn test_utm_reject_immediately() {
    use RejImmSymbol::*;
    let (status, _) = run_via_utm(&*REJECT_IMMEDIATELY_SPEC, &[One], 10_000);
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
fn test_utm_palindrome_accept() {
    use CheckPalindromeSymbol::*;
    let (status, _) = run_via_utm(&*CHECK_PALINDROME_SPEC, &[A, A], 10_000_000);
    assert_eq!(status, "accept");
}

#[test]
fn test_utm_palindrome_reject() {
    use CheckPalindromeSymbol::*;
    let (status, _) = run_via_utm(&*CHECK_PALINDROME_SPEC, &[A, B], 10_000_000);
    assert_eq!(status, "reject");
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
// Tests: Infinite UTM tape
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_infinite_tape_initial() {
    use crate::infinity::InfiniteTapeExtender;
    use crate::tm::TapeExtender;

    let mut tape: Vec<Symbol> = Vec::new();
    InfiniteTapeExtender.extend(&mut tape, 100);
    assert_eq!(tape[0], Symbol::Dollar);
    assert!(
        tape.len() >= 100,
        "tape should be at least 100 symbols: got {}",
        tape.len()
    );
}

#[test]
fn test_infinite_tape_self_similar() {
    use crate::infinity::InfiniteTapeExtender;
    use crate::tm::TapeExtender;

    let mut outer_tape: Vec<Symbol> = Vec::new();
    InfiniteTapeExtender.extend(&mut outer_tape, 100_000);

    let utm = &*UTM_SPEC;
    // Decode the outer tape to get the inner (guest) TM state.
    // The guest is also a UTM, and its tape contains guest-level symbols.
    let decoded =
        MyUtmEncodingScheme::decode(utm, &outer_tape).expect("should decode the infinite UTM tape");

    // Re-encode the decoded guest TM back into a UTM tape.
    // This should reproduce the outer tape (since UTM simulates itself).
    let mut re_encoded = MyUtmEncodingScheme::encode(&decoded);
    InfiniteTapeExtender.extend(&mut re_encoded, 100_000);

    assert_eq!(
        re_encoded[..100_000],
        outer_tape[..100_000],
        "re-encoded inner tape should equal the outer tape"
    );
}

#[test]
fn test_decoded_tape_no_leading_blanks() {
    use crate::infinity::InfiniteTapeExtender;
    use crate::tm::TapeExtender;

    let utm = &*UTM_SPEC;

    // Build the infinite UTM tape and decode it
    let mut outer_tape: Vec<Symbol> = Vec::new();
    InfiniteTapeExtender.extend(&mut outer_tape, 100_000);

    let decoded =
        MyUtmEncodingScheme::decode(utm, &outer_tape).expect("should decode the infinite UTM tape");

    // The decoded tape is the guest UTM's tape. Since the guest is a fresh UTM
    // simulating itself, its tape should start with $, not blanks.
    assert_eq!(
        decoded.tape[0],
        Symbol::Dollar,
        "decoded tape should start with $, not {:?}",
        decoded.tape[0]
    );
}

/// Regression: MIN_DISPLAY_TAPE_LEN = 10_000 was too small to contain the full
/// UTM header (which has 5 # delimiters). Decoding a tape of only 10k symbols
/// failed with "expected at least 5 # delimiters, found 1".
#[test]
fn test_decode_needs_more_than_10k_symbols() {
    use crate::infinity::{header_len, InfiniteTapeExtender};
    use crate::tm::TapeExtender;

    let utm = &*UTM_SPEC;

    // 10k is NOT enough — the header alone is larger than that.
    let mut small_tape: Vec<Symbol> = Vec::new();
    InfiniteTapeExtender.extend(&mut small_tape, 10_000);
    assert!(
        MyUtmEncodingScheme::decode(utm, &small_tape).is_err(),
        "10k symbols should NOT be enough to decode (header_len = {})",
        header_len(),
    );

    // header_len + 1000 IS enough.
    let min_len = header_len() + 1_000;
    let mut big_tape: Vec<Symbol> = Vec::new();
    InfiniteTapeExtender.extend(&mut big_tape, min_len);
    MyUtmEncodingScheme::decode(utm, &big_tape)
        .expect("should be able to decode a tape extended to header_len + 1000");
}

// ════════════════════════════════════════════════════════════════════
// Tests: UTM faithfulness
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_faithful_accept_immediately() {
    use AccImmSymbol::*;
    let spec = &*ACCEPT_IMMEDIATELY_SPEC;
    let mut tm = RunningTuringMachine::new(spec);
    tm.tape = vec![One];
    assert_faithful(tm, 100, 10_000);
}

#[test]
fn test_faithful_reject_immediately() {
    use RejImmSymbol::*;
    let spec = &*REJECT_IMMEDIATELY_SPEC;
    let mut tm = RunningTuringMachine::new(spec);
    tm.tape = vec![One];
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
fn test_faithful_palindrome_accept() {
    use CheckPalindromeSymbol::*;
    let spec = &*CHECK_PALINDROME_SPEC;
    let mut tm = RunningTuringMachine::new(spec);
    tm.tape = vec![A, B, A];
    assert_faithful(tm, 1_000, 10_000_000);
}

#[test]
fn test_faithful_palindrome_reject() {
    use CheckPalindromeSymbol::*;
    let spec = &*CHECK_PALINDROME_SPEC;
    let mut tm = RunningTuringMachine::new(spec);
    tm.tape = vec![A, B, C];
    assert_faithful(tm, 1_000, 10_000_000);
}

#[test]
fn test_faithful_palindrome_empty() {
    let spec = &*CHECK_PALINDROME_SPEC;
    let mut tm = RunningTuringMachine::new(spec);
    tm.tape = vec![spec.blank()];
    assert_faithful(tm, 1_000, 10_000_000);
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
    use AccImmSymbol::*;
    let acc_spec = &*ACCEPT_IMMEDIATELY_SPEC;
    let mut inner_tm = RunningTuringMachine::new(acc_spec);
    inner_tm.tape = vec![One];

    // Encode the inner TM into a UTM tape
    let encoded_inner = MyUtmEncodingScheme::encode(&inner_tm);

    let utm = &*UTM_SPEC;
    let mut utm_tm = RunningTuringMachine::new(utm);
    utm_tm.tape = encoded_inner;

    // Now assert_faithful runs this UTM-running-AcceptImmediately
    // through the UTM again (two levels of simulation).
    // Two levels of UTM overhead requires a very large step limit.
    assert_faithful(utm_tm, 10_000, 10_000_000_000);
}
