use utmmmmm::tm::RunningTuringMachine;
use utmmmmm::toy_machines::{FlipBitsSymbol, FLIP_BITS_SPEC};
use utmmmmm::utm::{MyUtmEncodingScheme, Symbol, UtmEncodingScheme};

fn symbol_to_char(s: Symbol) -> char {
    match s {
        Symbol::Blank => '_',
        Symbol::Zero => '0',
        Symbol::One => '1',
        Symbol::X => 'X',
        Symbol::Y => 'Y',
        Symbol::Hash => '#',
        Symbol::Pipe => '|',
        Symbol::Semi => ';',
        Symbol::Comma => ',',
        Symbol::Caret => '^',
        Symbol::L => 'L',
        Symbol::R => 'R',
        Symbol::Dot => '.',
        Symbol::Star => '*',
        Symbol::Gt => '>',
        Symbol::Dollar => '$',
    }
}

fn main() {
    let spec = &*FLIP_BITS_SPEC;
    let mut tm = RunningTuringMachine::new(spec);
    // Empty tape = just a blank
    tm.tape = vec![FlipBitsSymbol::Blank];

    let encoded = MyUtmEncodingScheme::encode(&tm);
    let tape_str: String = encoded.iter().map(|s| symbol_to_char(*s)).collect();

    println!("{}", tape_str);
}
