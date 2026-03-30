use utmmmmm::gen_utm::{Encoder, UtmSpec as _};
use utmmmmm::tm::RunningTuringMachine;
use utmmmmm::toy_machines::{FlipBitsSymbol, FLIP_BITS_SPEC};
use utmmmmm::utm::{make_utm_spec, Symbol};

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
    let utm_spec = make_utm_spec();
    let spec = &*FLIP_BITS_SPEC;
    let mut tm = RunningTuringMachine::new(spec);
    // Empty tape = just a blank
    tm.tape = vec![FlipBitsSymbol::Blank];

    let encoded = utm_spec.encoder(spec).encode(&tm);
    let tape_str: String = encoded.iter().map(|s| symbol_to_char(*s)).collect();

    println!("{}", tape_str);
}
