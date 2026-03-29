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

// ── Decode ──

export function decodeFromUtm(
  guestSpec: TuringMachineSpec,
  utmTape: readonly Symbol[],
): TuringMachineSnapshot {
  const guestStates = guestSpec.allStates;
  const guestSymbols = guestSpec.allSymbols;
  const nStateBits = numBits(guestStates.length);
  const nSymBits = numBits(guestSymbols.length);

  // Find # delimiters
  const hashes: number[] = [];
  for (let i = 0; i < utmTape.length; i++) {
    if (utmTape[i] === "#") hashes.push(i);
  }
  if (hashes.length < 5) {
    throw new Error(`Expected at least 5 # delimiters, found ${hashes.length}`);
  }

  // STATE section: between hashes[2] and hashes[3]
  const stateStart = hashes[2] + 1;
  const state = guestStates[fromBinary(utmTape, stateStart, nStateBits)];

  // TAPE section: after hashes[4]
  const tapeStart = hashes[4] + 1;
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
