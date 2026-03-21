use crate::toy_machines::*;
use crate::utm::*;

// ════════════════════════════════════════════════════════════════════
// Helpers
// ════════════════════════════════════════════════════════════════════

/// Run a guest TM directly, returning (final_state, status, tape_contents, head_pos).
fn run_guest_direct(
    spec: &TuringMachineSpec,
    input: &[u8],
    max_steps: i64,
) -> (u8, &'static str, Vec<u8>, i64) {
    let blank = spec.blank;
    let mut tape = InfiniteTape::new(input, blank);
    let mut head: i64 = 0;
    let mut state = spec.initial;

    let result = run_tm(spec, &mut tape, &mut head, &mut state, max_steps);
    let status = match result {
        RunResult::Accepted(_) => "accept",
        RunResult::Rejected(_) => "reject",
        RunResult::StepLimit(_) => "limit",
    };

    // Extract tape from 0 to max written position
    let max_pos = tape.right.len() as i64;
    let contents = tape.extract(0, max_pos - 1);
    (state, status, contents, head)
}

/// Run a guest TM via the UTM, returning the decoded guest state.
fn run_via_utm(
    guest: &TuringMachineSpec,
    input: &[u8],
    max_utm_steps: i64,
) -> (String, Vec<usize>) {
    let utm = build_utm_spec();
    let encoded = encode_tape(guest, input, 0, None);

    let mut tape = InfiniteTape::new(&encoded, SYM_BLANK);
    let mut head: i64 = 0;
    let mut state = utm.initial;

    let result = run_tm(&utm, &mut tape, &mut head, &mut state, max_utm_steps);

    let status = match result {
        RunResult::Accepted(_) => "accept",
        RunResult::Rejected(_) => "reject",
        RunResult::StepLimit(_) => "limit",
    };

    // Extract the UTM tape for decoding
    let min_pos = -(tape.left.len() as i64);
    let max_pos = tape.right.len() as i64 - 1;
    let flat: Vec<u8> = tape.extract(min_pos, max_pos);

    // Adjust: the tape starts at min_pos, so offset everything
    let offset = (-min_pos) as usize;

    if status == "accept" || status == "reject" {
        // Decode the guest state from the UTM tape
        // The UTM tape in memory starts from position min_pos
        // We need the portion from position 0 onward (where $ starts)
        let decoded = decode_tape(&flat[offset..], guest);
        (status.to_string(), decoded.tape)
    } else {
        (status.to_string(), vec![])
    }
}

// ════════════════════════════════════════════════════════════════════
// Tests: Direct guest TM execution
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_accept_immediately() {
    let spec = accept_immediately_spec();
    let (_, status, _, _) = run_guest_direct(&spec, &[], 100);
    assert_eq!(status, "accept");
}

#[test]
fn test_reject_immediately() {
    let spec = reject_immediately_spec();
    let (_, status, _, _) = run_guest_direct(&spec, &[], 100);
    assert_eq!(status, "reject");
}

#[test]
fn test_flip_bits_direct() {
    let spec = flip_bits_spec();
    // symbols: _ = 0, 0 = 1, 1 = 2
    let input = vec![1u8, 2u8]; // "0", "1"
    let (_, status, tape, _) = run_guest_direct(&spec, &input, 100);
    assert_eq!(status, "accept"); // halts when hitting blank (no rule for blank)
    // After flipping: "1", "0"
    assert_eq!(tape[0], 2); // "1"
    assert_eq!(tape[1], 1); // "0"
}

#[test]
fn test_palindrome_direct() {
    let spec = check_palindrome_spec();
    // symbols: _ = 0, a = 1, b = 2, c = 3
    // "aa" should be a palindrome
    let (_, status, _, _) = run_guest_direct(&spec, &[1, 1], 1000);
    assert_eq!(status, "accept");

    // "ab" should not be a palindrome
    let (_, status, _, _) = run_guest_direct(&spec, &[1, 2], 1000);
    assert_eq!(status, "reject");

    // "aba" should be a palindrome
    let (_, status, _, _) = run_guest_direct(&spec, &[1, 2, 1], 1000);
    assert_eq!(status, "accept");

    // empty should be a palindrome
    let (_, status, _, _) = run_guest_direct(&spec, &[], 1000);
    assert_eq!(status, "accept");
}

#[test]
fn test_double_x_direct() {
    let spec = double_x_spec();
    // symbols: _ = 0, $ = 1, X = 2, Y = 3, Z = 4
    // Input: "$XX" -> should produce "$XXXX"
    let input = vec![1u8, 2, 2]; // $, X, X
    let (_, status, tape, _) = run_guest_direct(&spec, &input, 1000);
    assert_eq!(status, "accept");
    // First cell should be $
    assert_eq!(tape[0], 1);
    // Next 4 cells should be X
    assert_eq!(tape[1], 2);
    assert_eq!(tape[2], 2);
    assert_eq!(tape[3], 2);
    assert_eq!(tape[4], 2);
}

// ════════════════════════════════════════════════════════════════════
// Tests: UTM spec construction
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_utm_spec_builds() {
    let utm = build_utm_spec();
    assert_eq!(utm.n_states, N_UTM_STATES);
    assert_eq!(utm.n_symbols, N_SYMBOLS);
    assert!(utm.ordered_rules.len() > 100, "UTM should have many rules");
}

// ════════════════════════════════════════════════════════════════════
// Tests: Encode/decode round-trip
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_encode_decode_roundtrip_flip_bits() {
    let guest = flip_bits_spec();
    let input = vec![1u8, 2u8]; // "0", "1"
    let encoded = encode_tape(&guest, &input, 0, None);
    let decoded = decode_tape(&encoded, &guest);
    assert_eq!(decoded.state, guest.initial as usize);
    assert_eq!(decoded.head_pos, 0);
    assert_eq!(decoded.tape, vec![1, 2]); // "0", "1"
}

#[test]
fn test_encode_decode_roundtrip_empty() {
    let guest = accept_immediately_spec();
    let encoded = encode_tape(&guest, &[], 0, None);
    let decoded = decode_tape(&encoded, &guest);
    assert_eq!(decoded.state, guest.initial as usize);
    assert_eq!(decoded.head_pos, 0);
    assert_eq!(decoded.tape, vec![0]); // blank
}

#[test]
fn test_encode_decode_roundtrip_palindrome() {
    let guest = check_palindrome_spec();
    let input = vec![1u8, 2u8, 1u8]; // "a", "b", "a"
    let encoded = encode_tape(&guest, &input, 0, None);
    let decoded = decode_tape(&encoded, &guest);
    assert_eq!(decoded.state, 0); // "start" is index 0
    assert_eq!(decoded.head_pos, 0);
    assert_eq!(decoded.tape, vec![1, 2, 1]);
}

// ════════════════════════════════════════════════════════════════════
// Tests: UTM simulation of guest TMs
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_utm_accept_immediately() {
    let guest = accept_immediately_spec();
    let (status, _) = run_via_utm(&guest, &[], 10_000);
    assert_eq!(status, "accept");
}

#[test]
fn test_utm_reject_immediately() {
    let guest = reject_immediately_spec();
    let (status, _) = run_via_utm(&guest, &[], 10_000);
    assert_eq!(status, "reject");
}

#[test]
fn test_utm_flip_bits() {
    let guest = flip_bits_spec();
    let input = vec![1u8, 2u8]; // "0", "1"
    let (status, tape) = run_via_utm(&guest, &input, 1_000_000);
    assert_eq!(status, "accept");
    // After flipping: "1"=2, "0"=1
    assert_eq!(tape[0], 2);
    assert_eq!(tape[1], 1);
}

#[test]
fn test_utm_flip_bits_5() {
    let guest = flip_bits_spec();
    // "01011" -> "10100"
    let input = vec![1u8, 2, 1, 2, 2]; // 0,1,0,1,1
    let (status, tape) = run_via_utm(&guest, &input, 10_000_000);
    assert_eq!(status, "accept");
    assert_eq!(tape[0], 2); // 1
    assert_eq!(tape[1], 1); // 0
    assert_eq!(tape[2], 2); // 1
    assert_eq!(tape[3], 1); // 0
    assert_eq!(tape[4], 1); // 0
}

#[test]
fn test_utm_palindrome_accept() {
    let guest = check_palindrome_spec();
    // "aa" is a palindrome
    let (status, _) = run_via_utm(&guest, &[1, 1], 10_000_000);
    assert_eq!(status, "accept");
}

#[test]
fn test_utm_palindrome_reject() {
    let guest = check_palindrome_spec();
    // "ab" is not a palindrome
    let (status, _) = run_via_utm(&guest, &[1, 2], 10_000_000);
    assert_eq!(status, "reject");
}

#[test]
fn test_utm_double_x() {
    let guest = double_x_spec();
    // "$XX" -> "$XXXX"
    let input = vec![1u8, 2, 2]; // $, X, X
    let (status, tape) = run_via_utm(&guest, &input, 50_000_000);
    assert_eq!(status, "accept");
    assert_eq!(tape[0], 1); // $
    assert_eq!(tape[1], 2); // X
    assert_eq!(tape[2], 2); // X
    assert_eq!(tape[3], 2); // X
    assert_eq!(tape[4], 2); // X
}

// ════════════════════════════════════════════════════════════════════
// Tests: Infinite UTM tape header
// ════════════════════════════════════════════════════════════════════

#[test]
fn test_infinite_tape_header() {
    let header = infinite_utm_tape_header();
    // Should start with $
    assert_eq!(header[0], SYM_DOLLAR);
    // Should end with the last # before the tape section
    // The header should be non-trivially long (UTM encoding is large)
    assert!(header.len() > 100, "header should be substantial: got {}", header.len());
}
