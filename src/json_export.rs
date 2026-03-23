/// Serializes Turing machine specs to JSON compatible with the TypeScript
/// `TuringMachineSpec<State, Symbol>` interface in `ui/src/types.ts`.

use std::collections::HashMap;
use std::hash::Hash;

use serde::Serialize;

use crate::tm::{Dir, TuringMachineSpec};

#[derive(Serialize)]
pub struct JsonTuringMachineSpec {
    pub name: String,
    pub description: String,
    #[serde(rename = "allStates")]
    pub all_states: Vec<String>,
    #[serde(rename = "allSymbols")]
    pub all_symbols: Vec<String>,
    pub initial: String,
    #[serde(rename = "acceptingStates")]
    pub accepting_states: Vec<String>,
    pub blank: String,
    /// Keyed by state -> symbol -> [new_state, new_symbol, dir]
    pub rules: HashMap<String, HashMap<String, (String, String, String)>>,
    /// Maps each symbol's string name to a display character.
    #[serde(rename = "symbolChars")]
    pub symbol_chars: HashMap<String, String>,
}

pub fn export_spec<Spec: TuringMachineSpec>(
    spec: &Spec,
    name: &str,
    description: &str,
    state_name: impl Fn(Spec::State) -> String,
    symbol_name: impl Fn(Spec::Symbol) -> String,
    symbol_char: impl Fn(Spec::Symbol) -> char,
) -> JsonTuringMachineSpec
where
    Spec::State: Eq + Hash,
    Spec::Symbol: Eq + Hash,
{
    let all_states: Vec<String> = spec.iter_states().map(&state_name).collect();
    let all_symbols: Vec<String> = spec.iter_symbols().map(&symbol_name).collect();
    let initial = state_name(spec.initial());
    let blank = symbol_name(spec.blank());

    let accepting_states: Vec<String> = spec
        .iter_states()
        .filter(|s| spec.is_accepting(*s))
        .map(&state_name)
        .collect();

    let mut rules: HashMap<String, HashMap<String, (String, String, String)>> = HashMap::new();
    for (st, sym, nst, nsym, dir) in spec.iter_rules() {
        let dir_str = match dir {
            Dir::Left => "L".to_string(),
            Dir::Right => "R".to_string(),
        };
        rules
            .entry(state_name(st))
            .or_default()
            .insert(symbol_name(sym), (state_name(nst), symbol_name(nsym), dir_str));
    }

    let symbol_chars: HashMap<String, String> = spec
        .iter_symbols()
        .map(|s| (symbol_name(s), symbol_char(s).to_string()))
        .collect();

    JsonTuringMachineSpec {
        name: name.to_string(),
        description: description.to_string(),
        all_states,
        all_symbols,
        initial,
        accepting_states,
        blank,
        rules,
        symbol_chars,
    }
}
