use utmmmmm::tm::RunningTuringMachine;
use utmmmmm::toy_machines::*;
use utmmmmm::utm::{format_tape, MyUtmEncodingScheme, UtmEncodingScheme};

fn main() {
    // encode(flipBits, 01101)
    {
        use FlipBitsSymbol::*;
        let spec = &*FLIP_BITS_SPEC;
        let mut tm = RunningTuringMachine::new(spec);
        tm.tape = vec![Zero, One, One, Zero, One];
        let encoded = MyUtmEncodingScheme::encode(&tm);
        println!("encode(flipBits, 01101):");
        println!("{}", format_tape(&encoded));
        println!();
    }

    // encode(palindromeChecker, ABBABA)
    {
        use CheckPalindromeSymbol::*;
        let spec = &*CHECK_PALINDROME_SPEC;
        let mut tm = RunningTuringMachine::new(spec);
        tm.tape = vec![A, B, B, A, B, A];
        let encoded = MyUtmEncodingScheme::encode(&tm);
        println!("encode(palindromeChecker, ABBABA):");
        println!("{}", format_tape(&encoded));
    }
}
