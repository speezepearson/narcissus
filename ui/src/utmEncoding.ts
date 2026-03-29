/**
 * TypeScript port of the UTM tape encoding/decoding logic from Rust (utm.rs).
 *
 * Tape layout: # ACCEPTSTATES # BLANK # RULES $ STATE # TAPE
 *
 * RULES:   dot-separated entries, each = stateBits | symBits | newStateBits | newSymBits | dir
 * ACCEPTSTATES: semicolon-separated state encodings
 * STATE:   current state bits
 * BLANK:   blank symbol bits
 * TAPE:    comma-separated cells, head cell prefixed with ^
 */

import {
  type State,
  State as StateSchema,
  type Symbol,
  Symbol as SymbolSchema,
  type TuringMachineSnapshot,
  type TuringMachineSpec,
} from "./types";

// ── Helpers ──

export function numBits(count: number): number {
  return Math.max(1, Math.ceil(Math.log2(Math.max(count, 2))));
}

function toBinary(index: number, width: number): Symbol[] {
  const bits: Symbol[] = [];
  for (let i = width - 1; i >= 0; i--) {
    bits.push(SymbolSchema.parse((index >> i) & 1 ? "1" : "0"));
  }
  return bits;
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

const S = (c: string) => SymbolSchema.parse(c);

// ── Encode ──

export function encodeForUtm(
  guestSpec: TuringMachineSpec,
  guestSnapshot: TuringMachineSnapshot,
): Symbol[] {
  const stateToIdx = new Map<State, number>();
  guestSpec.allStates.forEach((s, i) => stateToIdx.set(s, i));

  const symToIdx = new Map<Symbol, number>();
  guestSpec.allSymbols.forEach((s, i) => symToIdx.set(s, i));

  const nStateBits = numBits(guestSpec.allStates.length);
  const nSymBits = numBits(guestSpec.allSymbols.length);

  // New layout: # ACC # BLANK # RULES $ STATE # TAPE
  const tape: Symbol[] = [];

  // # ACCEPTSTATES
  tape.push(S("#"));
  let firstAcc = true;
  for (const state of guestSpec.allStates) {
    if (!guestSpec.acceptingStates.has(state)) continue;
    if (!firstAcc) tape.push(S(";"));
    firstAcc = false;
    tape.push(...toBinary(stateToIdx.get(state)!, nStateBits));
  }

  // # BLANK
  tape.push(S("#"));
  tape.push(...toBinary(symToIdx.get(guestSpec.blank)!, nSymBits));

  // # RULES
  tape.push(S("#"));
  let firstRule = true;
  for (const [state, innerMap] of guestSpec.rules) {
    for (const [sym, [nst, nsym, dir]] of innerMap) {
      if (!firstRule) tape.push(S(";"));
      firstRule = false;
      tape.push(S("."));
      tape.push(...toBinary(stateToIdx.get(state)!, nStateBits));
      tape.push(S("|"));
      tape.push(...toBinary(symToIdx.get(sym)!, nSymBits));
      tape.push(S("|"));
      tape.push(...toBinary(stateToIdx.get(nst)!, nStateBits));
      tape.push(S("|"));
      tape.push(...toBinary(symToIdx.get(nsym)!, nSymBits));
      tape.push(S("|"));
      tape.push(S(dir));
    }
  }

  // $ STATE
  tape.push(S("$"));
  tape.push(...toBinary(stateToIdx.get(guestSnapshot.state)!, nStateBits));

  // # TAPE
  tape.push(S("#"));
  const guestTape =
    guestSnapshot.tape.length === 0 ? [guestSpec.blank] : guestSnapshot.tape;
  const caretStart = tape.length;
  for (const sym of guestTape) {
    tape.push(S(","));
    tape.push(...toBinary(symToIdx.get(sym)!, nSymBits));
  }
  // Replace the comma at head position with ^
  let commaIdx = caretStart;
  for (let i = 0; i < guestSnapshot.pos; i++) {
    commaIdx += 1 + nSymBits; // skip comma + bits
  }
  tape[commaIdx] = S("^");

  return tape;
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

  // New layout: # ACC # BLANK # RULES $ STATE # TAPE
  // Find $ to locate STATE, find first # after $ for TAPE
  const dollarPos = utmTape.indexOf(S("$"));
  if (dollarPos < 0) {
    throw new Error("No $ delimiter found");
  }

  // STATE section: right after $
  const stateStart = dollarPos + 1;
  const state = guestStates[fromBinary(utmTape, stateStart, nStateBits)];

  // TAPE section: after first # following $
  const tapeHash = utmTape.indexOf(S("#"), dollarPos);
  if (tapeHash < 0) {
    throw new Error("No # after $ for TAPE section");
  }
  const tapeStart = tapeHash + 1;
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
