use utmmmmm::json_export::export_spec;
use utmmmmm::toy_machines::*;
use utmmmmm::utm;

fn main() {
    let specs = vec![
        export_spec(
            &*ACCEPT_IMMEDIATELY_SPEC,
            "Accept Immediately",
            "Immediately accepts (initial state is accepting).",
            |s| format!("{:?}", s),
            |s| format!("{:?}", s),
            |s| match s {
                AccImmSymbol::Blank => '_',
                AccImmSymbol::One => '1',
            },
        ),
        export_spec(
            &*REJECT_IMMEDIATELY_SPEC,
            "Reject Immediately",
            "Immediately rejects (no transition, non-accepting).",
            |s| format!("{:?}", s),
            |s| format!("{:?}", s),
            |s| match s {
                RejImmSymbol::Blank => '_',
                RejImmSymbol::One => '1',
            },
        ),
        export_spec(
            &*FLIP_BITS_SPEC,
            "Flip Bits",
            "Flips 0s to 1s and vice versa, then halts at blank.",
            |s| format!("{:?}", s),
            |s| format!("{:?}", s),
            |s| match s {
                FlipBitsSymbol::Blank => '_',
                FlipBitsSymbol::Zero => '0',
                FlipBitsSymbol::One => '1',
            },
        ),
        export_spec(
            &*CHECK_PALINDROME_SPEC,
            "Check Palindrome",
            "Checks if the input (over {A, B, C}) is a palindrome.",
            |s| format!("{:?}", s),
            |s| format!("{:?}", s),
            |s| match s {
                CheckPalindromeSymbol::Blank => '_',
                CheckPalindromeSymbol::A => 'a',
                CheckPalindromeSymbol::B => 'b',
                CheckPalindromeSymbol::C => 'c',
            },
        ),
        export_spec(
            &*DOUBLE_X_SPEC,
            "Double X",
            "Doubles a string of X's preceded by $. E.g. $XXX -> $XXXXXX.",
            |s| format!("{:?}", s),
            |s| format!("{:?}", s),
            |s| match s {
                DoubleXSymbol::Blank => '_',
                DoubleXSymbol::Dollar => '$',
                DoubleXSymbol::X => 'X',
                DoubleXSymbol::Y => 'Y',
                DoubleXSymbol::Z => 'Z',
            },
        ),
        export_spec(
            &*utm::UTM_SPEC,
            "Universal Turing Machine",
            "A universal Turing machine that can simulate any other TM given an encoded description on its tape.",
            |s| format!("{:?}", s),
            |s| format!("{:?}", s),
            |s| match s {
                utm::Symbol::Blank => '_',
                utm::Symbol::Zero => '0',
                utm::Symbol::One => '1',
                utm::Symbol::X => 'X',
                utm::Symbol::Y => 'Y',
                utm::Symbol::Hash => '#',
                utm::Symbol::Pipe => '|',
                utm::Symbol::Semi => ';',
                utm::Symbol::Comma => ',',
                utm::Symbol::Caret => '^',
                utm::Symbol::L => 'L',
                utm::Symbol::R => 'R',
                utm::Symbol::Dot => '.',
                utm::Symbol::Star => '*',
                utm::Symbol::Gt => '>',
                utm::Symbol::Dollar => '$',
            },
        ),
    ];

    println!("{}", serde_json::to_string_pretty(&specs).unwrap());
}
