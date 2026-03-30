use std::collections::HashMap;

use crate::infinity::InfiniteTape;
use crate::utm::Symbol;

/// Compute the current set of overwrites (positions differing from background).
pub fn current_overwrites(tape: &[Symbol], background: &InfiniteTape) -> HashMap<usize, Symbol> {
    tape.iter()
        .zip(background.iter_forever())
        .enumerate()
        .filter(|(_, (c, r))| *c != r)
        .map(|(i, (&c, _))| (i, c))
        .collect()
}

/// Diff the current overwrites against a client's known state.
/// Returns the (position, display_string) pairs the client needs to apply,
/// and updates `client` to match `current`.
pub fn compute_new_overwrites(
    current: &HashMap<usize, Symbol>,
    client: &mut HashMap<usize, Symbol>,
    background: &InfiniteTape,
) -> Vec<(usize, Symbol)> {
    let mut new_overwrites = Vec::new();

    // Positions in current that differ from what client knows
    for (&pos, &sym) in current {
        match client.get(&pos) {
            Some(&known) if known == sym => {} // unchanged
            _ => {
                new_overwrites.push((pos, sym));
            }
        }
    }

    // Positions client has that are no longer overwritten (reverted to unblemished)
    for (&pos, _) in client.iter() {
        if !current.contains_key(&pos) {
            let sym = background.get(pos);
            new_overwrites.push((pos, sym));
        }
    }

    // Update client state to match current
    *client = current.clone();

    new_overwrites.sort_by_key(|(pos, _)| *pos);
    new_overwrites
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        optimization_hints::make_my_utm_self_optimization_hints,
        utm::{make_utm_spec, Symbol},
    };

    #[test]
    fn test_total_then_two_deltas() {
        let spec = make_utm_spec();
        let encoder = make_my_utm_self_optimization_hints(&spec);
        let background = InfiniteTape::new(&encoder);

        // === Total event: tape has position 1 changed to X ===
        let mut tape = vec![];
        background.extend(&mut tape, 5);

        tape[1] = Symbol::Caret;
        let mut client = current_overwrites(&tape, &background);
        assert_eq!(client, HashMap::from([(1, Symbol::Caret)]));

        tape[2] = Symbol::Caret;
        let delta = compute_new_overwrites(
            &current_overwrites(&tape, &background),
            &mut client,
            &background,
        );
        assert_eq!(delta, vec![(2, Symbol::Caret)]);
        assert_eq!(
            client,
            HashMap::from([(1, Symbol::Caret), (2, Symbol::Caret)])
        );

        tape[1] = background.get(1);
        tape[3] = Symbol::Caret;
        let delta = compute_new_overwrites(
            &current_overwrites(&tape, &background),
            &mut client,
            &background,
        );
        assert_eq!(delta, vec![(1, background.get(1)), (3, Symbol::Caret)]);
        assert_eq!(
            client,
            HashMap::from([(2, Symbol::Caret), (3, Symbol::Caret)])
        ); // it no longer remembers 1 because that pos is same as background now
    }
}
