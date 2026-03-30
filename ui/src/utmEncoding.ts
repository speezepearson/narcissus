/**
 * UTM tape decoding logic (ported from Rust utm.rs).
 *
 * Tape layout: $ ACC #[0] BLANK #[1] RULES #[2] STATE #[3] SYMCACHE #[4] TAPE
 */

import {
  State as StateSchema,
  type Symbol,
  type TuringMachineSnapshot,
  type TuringMachineSpec,
} from "./types";

// ── Helpers ──

export function numBits(count: number): number {
  return Math.max(1, Math.ceil(Math.log2(Math.max(count, 2))));
}

function fromBinary(
  tape: readonly Symbol[],
  start: number,
  width: number,
): number {
  let val = 0;
  for (let i = 0; i < width; i++) {
    const b = tape[start + i];
    if (b === "1" || b === "Y") val = val * 2 + 1;
    else if (b === "0" || b === "X") val = val * 2;
    else throw new Error(`Invalid binary symbol at ${start + i}: ${b}`);
  }
  return val;
}

function toBinary(index: number, width: number): boolean[] {
  const bits: boolean[] = [];
  for (let i = width - 1; i >= 0; i--) {
    bits.push(((index >> i) & 1) === 1);
  }
  return bits;
}

function bitsToKey(bits: boolean[]): string {
  return bits.map((b) => (b ? "1" : "0")).join("");
}

/**
 * Build a reverse mapping from bit-reversed binary encoding to state,
 * matching the Rust greedy assignment in from_transition_stats with empty stats.
 *
 * With empty stats every rule has count 0, so group_rules preserves input order.
 * We walk the guest rules, group noops, then iterate from last (most frequent)
 * to first, assigning bit-reversed indices.
 */
function buildStateEncodingMap(
  guestSpec: TuringMachineSpec,
): Map<string, (typeof guestSpec.allStates)[number]> {
  type S = (typeof guestSpec.allStates)[number];
  const nStateBits = numBits(guestSpec.allStates.length);

  // Replicate group_rules with empty stats: collect rules in iteration order,
  // group noops (same state+dir where new_state==state && new_sym==sym),
  // emit each group once in the position of its first member.
  // With zero stats the sort is stable, so order is preserved.
  const ruleStatesInOrder: S[] = [];
  const noopEmitted = new Set<string>();

  for (const [state, symMap] of guestSpec.rules) {
    for (const [sym, [newState, newSym, dir]] of symMap) {
      const isNoop = newState === state && newSym === sym;
      if (isNoop) {
        const key = `${state}|${dir}`;
        if (noopEmitted.has(key)) continue;
        noopEmitted.add(key);
      }
      ruleStatesInOrder.push(state);
    }
  }

  // Walk from last to first, assign bit-reversed encodings
  const encodingToState = new Map<string, S>();
  const assignedStates = new Set<S>();
  let nextIndex = 0;

  for (let i = ruleStatesInOrder.length - 1; i >= 0; i--) {
    const st = ruleStatesInOrder[i];
    if (assignedStates.has(st)) continue;
    assignedStates.add(st);
    const bits = toBinary(nextIndex, nStateBits);
    bits.reverse();
    encodingToState.set(bitsToKey(bits), st);
    nextIndex++;
  }

  // Assign remaining states not in any rule
  for (const st of guestSpec.allStates) {
    if (assignedStates.has(st)) continue;
    assignedStates.add(st);
    const bits = toBinary(nextIndex, nStateBits);
    bits.reverse();
    encodingToState.set(bitsToKey(bits), st);
    nextIndex++;
  }

  return encodingToState;
}

// ── Decode ──

export function decodeFromUtm(
  guestSpec: TuringMachineSpec,
  utmTape: readonly Symbol[],
): TuringMachineSnapshot {
  const guestSymbols = guestSpec.allSymbols;
  const nStateBits = numBits(guestSpec.allStates.length);
  const nSymBits = numBits(guestSymbols.length);

  const stateEncodings = buildStateEncodingMap(guestSpec);

  // Find # delimiters
  const hashes: number[] = [];
  for (let i = 0; i < utmTape.length; i++) {
    if (utmTape[i] === "#") hashes.push(i);
  }
  if (hashes.length < 4) {
    throw new Error(`Expected at least 4 # delimiters, found ${hashes.length}`);
  }

  // STATE section: between hashes[2] and hashes[3]
  const stateStart = hashes[2] + 1;
  const stateBitsVal = fromBinary(utmTape, stateStart, nStateBits);
  const stateBits = toBinary(stateBitsVal, nStateBits);
  const state = stateEncodings.get(bitsToKey(stateBits));
  if (state === undefined) {
    throw new Error(`Unknown state encoding: ${bitsToKey(stateBits)}`);
  }

  // TAPE section: after hashes[3]
  const tapeStart = hashes[3] + 1;
  const tapeSection = utmTape.slice(tapeStart);

  const cells: number[] = [];
  let headPos = 0;
  let i = 0;
  let cellIdx = 0;
  while (i < tapeSection.length) {
    const s = tapeSection[i];
    if (s === "_" || s === "$") break;
    if (s === ",") {
      i++;
      cellIdx++;
      continue;
    }
    if (s === "^" || s === ">") {
      if (s === "^") headPos = cellIdx;
      i++;
      continue;
    }
    if (i + nSymBits > tapeSection.length) break;
    cells.push(fromBinary(tapeSection, i, nSymBits));
    i += nSymBits;
  }

  return {
    spec: guestSpec,
    state: StateSchema.parse(state),
    pos: headPos,
    tape: cells.map((idx) => guestSymbols[idx]),
  };
}
