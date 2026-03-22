// use crate::tm::run_tm;
// use crate::tm::TuringMachineSpec;
// use crate::toy_machines::*;
// use crate::utm::*;

// // ════════════════════════════════════════════════════════════════════
// // Helpers
// // ════════════════════════════════════════════════════════════════════

// fn sym(v: u8) -> Symbol {
//     Symbol(v)
// }

// fn run_guest_direct<Spec: TuringMachineSpec>(
//     spec: Spec,
//     input: &[Spec::Symbol],
//     max_steps: i64,
// ) -> (State, &'static str, Vec<Symbol>, i64) {
//     let blank = spec.blank;
//     let mut tape = InfiniteTape::new(input, blank);
//     let mut head: i64 = 0;
//     let mut state = spec.initial;

//     let result = run_tm(spec, &mut tape, &mut head, &mut state, max_steps);
//     let status = match result {
//         RunResult::Accepted(_) => "accept",
//         RunResult::Rejected(_) => "reject",
//         RunResult::StepLimit(_) => "limit",
//     };

//     let max_pos = tape.right.len() as i64;
//     let contents = tape.extract(0, max_pos - 1);
//     (state, status, contents, head)
// }

// fn run_via_utm(
//     guest: &TuringMachineSpec,
//     input: &[Symbol],
//     max_utm_steps: i64,
// ) -> (String, Vec<usize>) {
//     let utm = build_utm_spec();
//     let encoded = encode_tape(guest, input, 0, None);

//     let mut tape = InfiniteTape::new(&encoded, SYM_BLANK);
//     let mut head: i64 = 0;
//     let mut state = utm.initial;

//     let result = run_tm(&utm, &mut tape, &mut head, &mut state, max_utm_steps);

//     let status = match result {
//         RunResult::Accepted(_) => "accept",
//         RunResult::Rejected(_) => "reject",
//         RunResult::StepLimit(_) => "limit",
//     };

//     let min_pos = -(tape.left.len() as i64);
//     let max_pos = tape.right.len() as i64 - 1;
//     let flat: Vec<Symbol> = tape.extract(min_pos, max_pos);

//     let offset = (-min_pos) as usize;

//     if status == "accept" || status == "reject" {
//         let decoded = decode_tape(&flat[offset..], guest);
//         (status.to_string(), decoded.tape)
//     } else {
//         (status.to_string(), vec![])
//     }
// }

// // ════════════════════════════════════════════════════════════════════
// // Tests: Direct guest TM execution
// // ════════════════════════════════════════════════════════════════════

// #[test]
// fn test_accept_immediately() {
//     let spec = accept_immediately_spec();
//     let (_, status, _, _) = run_guest_direct(&spec, &[], 100);
//     assert_eq!(status, "accept");
// }

// #[test]
// fn test_reject_immediately() {
//     let spec = reject_immediately_spec();
//     let (_, status, _, _) = run_guest_direct(&spec, &[], 100);
//     assert_eq!(status, "reject");
// }

// #[test]
// fn test_flip_bits_direct() {
//     let spec = flip_bits_spec();
//     let input = vec![sym(1), sym(2)]; // "0", "1"
//     let (_, status, tape, _) = run_guest_direct(&spec, &input, 100);
//     assert_eq!(status, "accept");
//     assert_eq!(tape[0], sym(2)); // "1"
//     assert_eq!(tape[1], sym(1)); // "0"
// }

// #[test]
// fn test_palindrome_direct() {
//     let spec = check_palindrome_spec();
//     let (_, status, _, _) = run_guest_direct(&spec, &[sym(1), sym(1)], 1000);
//     assert_eq!(status, "accept");

//     let (_, status, _, _) = run_guest_direct(&spec, &[sym(1), sym(2)], 1000);
//     assert_eq!(status, "reject");

//     let (_, status, _, _) = run_guest_direct(&spec, &[sym(1), sym(2), sym(1)], 1000);
//     assert_eq!(status, "accept");

//     let (_, status, _, _) = run_guest_direct(&spec, &[], 1000);
//     assert_eq!(status, "accept");
// }

// #[test]
// fn test_double_x_direct() {
//     let spec = double_x_spec();
//     let input = vec![sym(1), sym(2), sym(2)]; // $, X, X
//     let (_, status, tape, _) = run_guest_direct(&spec, &input, 1000);
//     assert_eq!(status, "accept");
//     assert_eq!(tape[0], sym(1)); // $
//     assert_eq!(tape[1], sym(2)); // X
//     assert_eq!(tape[2], sym(2)); // X
//     assert_eq!(tape[3], sym(2)); // X
//     assert_eq!(tape[4], sym(2)); // X
// }

// // ════════════════════════════════════════════════════════════════════
// // Tests: UTM spec construction
// // ════════════════════════════════════════════════════════════════════

// #[test]
// fn test_utm_spec_builds() {
//     let utm = build_utm_spec();
//     assert_eq!(utm.n_states, N_UTM_STATES);
//     assert_eq!(utm.n_symbols, N_SYMBOLS);
//     assert!(utm.ordered_rules.len() > 100, "UTM should have many rules");
// }

// // ════════════════════════════════════════════════════════════════════
// // Tests: Encode/decode round-trip
// // ════════════════════════════════════════════════════════════════════

// #[test]
// fn test_encode_decode_roundtrip_flip_bits() {
//     let guest = flip_bits_spec();
//     let input = vec![sym(1), sym(2)]; // "0", "1"
//     let encoded = encode_tape(&guest, &input, 0, None);
//     let decoded = decode_tape(&encoded, &guest);
//     assert_eq!(decoded.state, guest.initial.0 as usize);
//     assert_eq!(decoded.head_pos, 0);
//     assert_eq!(decoded.tape, vec![1, 2]);
// }

// #[test]
// fn test_encode_decode_roundtrip_empty() {
//     let guest = accept_immediately_spec();
//     let encoded = encode_tape(&guest, &[], 0, None);
//     let decoded = decode_tape(&encoded, &guest);
//     assert_eq!(decoded.state, guest.initial.0 as usize);
//     assert_eq!(decoded.head_pos, 0);
//     assert_eq!(decoded.tape, vec![0]); // blank
// }

// #[test]
// fn test_encode_decode_roundtrip_palindrome() {
//     let guest = check_palindrome_spec();
//     let input = vec![sym(1), sym(2), sym(1)]; // "a", "b", "a"
//     let encoded = encode_tape(&guest, &input, 0, None);
//     let decoded = decode_tape(&encoded, &guest);
//     assert_eq!(decoded.state, 0); // "start" is index 0
//     assert_eq!(decoded.head_pos, 0);
//     assert_eq!(decoded.tape, vec![1, 2, 1]);
// }

// // ════════════════════════════════════════════════════════════════════
// // Tests: UTM simulation of guest TMs
// // ════════════════════════════════════════════════════════════════════

// #[test]
// fn test_utm_accept_immediately() {
//     let guest = accept_immediately_spec();
//     let (status, _) = run_via_utm(&guest, &[], 10_000);
//     assert_eq!(status, "accept");
// }

// #[test]
// fn test_utm_reject_immediately() {
//     let guest = reject_immediately_spec();
//     let (status, _) = run_via_utm(&guest, &[], 10_000);
//     assert_eq!(status, "reject");
// }

// #[test]
// fn test_utm_flip_bits() {
//     let guest = flip_bits_spec();
//     let input = vec![sym(1), sym(2)]; // "0", "1"
//     let (status, tape) = run_via_utm(&guest, &input, 1_000_000);
//     assert_eq!(status, "accept");
//     assert_eq!(tape[0], 2);
//     assert_eq!(tape[1], 1);
// }

// #[test]
// fn test_utm_flip_bits_5() {
//     let guest = flip_bits_spec();
//     let input = vec![sym(1), sym(2), sym(1), sym(2), sym(2)];
//     let (status, tape) = run_via_utm(&guest, &input, 10_000_000);
//     assert_eq!(status, "accept");
//     assert_eq!(tape[0], 2);
//     assert_eq!(tape[1], 1);
//     assert_eq!(tape[2], 2);
//     assert_eq!(tape[3], 1);
//     assert_eq!(tape[4], 1);
// }

// #[test]
// fn test_utm_palindrome_accept() {
//     let guest = check_palindrome_spec();
//     let (status, _) = run_via_utm(&guest, &[sym(1), sym(1)], 10_000_000);
//     assert_eq!(status, "accept");
// }

// #[test]
// fn test_utm_palindrome_reject() {
//     let guest = check_palindrome_spec();
//     let (status, _) = run_via_utm(&guest, &[sym(1), sym(2)], 10_000_000);
//     assert_eq!(status, "reject");
// }

// #[test]
// fn test_utm_double_x() {
//     let guest = double_x_spec();
//     let input = vec![sym(1), sym(2), sym(2)]; // $, X, X
//     let (status, tape) = run_via_utm(&guest, &input, 50_000_000);
//     assert_eq!(status, "accept");
//     assert_eq!(tape[0], 1); // $
//     assert_eq!(tape[1], 2); // X
//     assert_eq!(tape[2], 2); // X
//     assert_eq!(tape[3], 2); // X
//     assert_eq!(tape[4], 2); // X
// }

// // ════════════════════════════════════════════════════════════════════
// // Tests: Infinite UTM tape header
// // ════════════════════════════════════════════════════════════════════

// #[test]
// fn test_infinite_tape_header() {
//     let header = infinite_utm_tape_header();
//     assert_eq!(header[0], SYM_DOLLAR);
//     assert!(
//         header.len() > 100,
//         "header should be substantial: got {}",
//         header.len()
//     );
// }
