use std::collections::HashMap;
use std::fmt::{Display, Write};
use std::hash::Hash;

/// Per-level client state: the overwrites the client currently has applied.
pub struct ClientLevelState<Sym> {
    pub overwrites: Vec<HashMap<usize, Sym>>,
}

impl<Sym: Copy + Eq + Hash> ClientLevelState<Sym> {
    pub fn new() -> Self {
        Self {
            overwrites: vec![HashMap::new()],
        }
    }

    /// Initialize from a full tape + reference (after sending a `total` event).
    pub fn from_tape(tape: &[Sym], reference: &[Sym]) -> Self {
        let overwrites = tape
            .iter()
            .zip(reference.iter())
            .enumerate()
            .filter(|(_, (c, r))| c != r)
            .map(|(i, (&c, _))| (i, c))
            .collect();
        Self {
            overwrites: vec![overwrites],
        }
    }
}

/// Compute the current set of overwrites (positions differing from reference).
pub fn current_overwrites<Sym: Copy + Eq + Hash>(
    tape: &[Sym],
    reference: &[Sym],
) -> HashMap<usize, Sym> {
    tape.iter()
        .zip(reference.iter())
        .enumerate()
        .filter(|(_, (c, r))| c != r)
        .map(|(i, (&c, _))| (i, c))
        .collect()
}

/// Diff the current overwrites against a client's known state.
/// Returns the (position, display_string) pairs the client needs to apply,
/// and updates `client` to match `current`.
pub fn compute_new_overwrites<Sym: Copy + Eq + Hash + Display>(
    current: &HashMap<usize, Sym>,
    client: &mut HashMap<usize, Sym>,
    reference: &[Sym],
) -> Vec<(usize, String)> {
    let mut new_overwrites = Vec::new();

    // Positions in current that differ from what client knows
    for (&pos, &sym) in current {
        match client.get(&pos) {
            Some(&known) if known == sym => {} // unchanged
            _ => {
                let mut s = String::new();
                write!(s, "{}", sym).unwrap();
                new_overwrites.push((pos, s));
            }
        }
    }

    // Positions client has that are no longer overwritten (reverted to unblemished)
    for (&pos, _) in client.iter() {
        if !current.contains_key(&pos) {
            let sym = reference[pos];
            let mut s = String::new();
            write!(s, "{}", sym).unwrap();
            new_overwrites.push((pos, s));
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
    use crate::utm::Symbol;

    #[test]
    fn test_total_then_two_deltas() {
        let reference = vec![
            Symbol::Zero, // 0
            Symbol::One,  // 1
            Symbol::Zero, // 2
            Symbol::One,  // 3
            Symbol::Zero, // 4
        ];

        // === Total event: tape has position 1 changed to X ===
        let tape_0 = vec![
            Symbol::Zero, // 0: same
            Symbol::X,    // 1: overwritten
            Symbol::Zero, // 2: same
            Symbol::One,  // 3: same
            Symbol::Zero, // 4: same
        ];

        // The total event gives the full tape string
        let total_tape: String = tape_0.iter().map(|s| format!("{}", s)).collect();
        assert_eq!(total_tape, "0X010");

        // Client state after total = overwrites relative to reference
        let mut client = ClientLevelState::from_tape(&tape_0, &reference);
        assert_eq!(client.overwrites[0].len(), 1);
        assert_eq!(client.overwrites[0][&1], Symbol::X);

        // === Delta 1: position 2 also changed to Y ===
        let tape_1 = vec![
            Symbol::Zero, // 0: same
            Symbol::X,    // 1: still overwritten (same)
            Symbol::Y,    // 2: newly overwritten
            Symbol::One,  // 3: same
            Symbol::Zero, // 4: same
        ];

        let cur_1 = current_overwrites(&tape_1, &reference);
        let delta_1 = compute_new_overwrites(&cur_1, &mut client.overwrites[0], &reference);

        // Only position 2 is new
        assert_eq!(delta_1, vec![(2, "Y".to_string())]);
        // Client now knows about {1: X, 2: Y}
        assert_eq!(client.overwrites[0].len(), 2);

        // === Delta 2: position 1 reverted, position 3 changed to Star ===
        let tape_2 = vec![
            Symbol::Zero, // 0: same
            Symbol::One,  // 1: reverted to unblemished
            Symbol::Y,    // 2: still overwritten (same as delta 1)
            Symbol::Star, // 3: newly overwritten
            Symbol::Zero, // 4: same
        ];

        let cur_2 = current_overwrites(&tape_2, &reference);
        let delta_2 = compute_new_overwrites(&cur_2, &mut client.overwrites[0], &reference);

        // Position 1 reverted (client gets unblemished value "1")
        // Position 3 newly overwritten with "*"
        // Position 2 unchanged, not included
        assert_eq!(delta_2, vec![(1, "1".to_string()), (3, "*".to_string())]);
        // Client now knows about {2: Y, 3: Star}
        assert_eq!(client.overwrites[0].len(), 2);
        assert_eq!(client.overwrites[0].get(&1), None);
        assert_eq!(client.overwrites[0][&2], Symbol::Y);
        assert_eq!(client.overwrites[0][&3], Symbol::Star);
    }
}
