import type { TuringMachineSnapshot, TuringMachineSpec } from "./types";
import { makeArrayTapeOverlay } from "./util";

// ════════════════════════════════════════════════════════════════════
// Branded index types
// ════════════════════════════════════════════════════════════════════
declare const StateIdxBrand: unique symbol;
declare const SymbolIdxBrand: unique symbol;
export type StateIdx = number & { readonly [StateIdxBrand]: true };
export type SymbolIdx = number & { readonly [SymbolIdxBrand]: true };

// ════════════════════════════════════════════════════════════════════
// Compiled machine
// ════════════════════════════════════════════════════════════════════

const NO_RULE = -1;
// Packing: newState (bits 0-15) | newSymbol (bits 16-23) | dir (bit 24, 0=L 1=R)
const SYMBOL_SHIFT = 16;
const DIR_SHIFT = 24;
const STATE_MASK = 0xffff;
const SYMBOL_MASK = 0xff;

export type CompiledMachine = {
  readonly numStates: number;
  readonly numSymbols: number;
  /** Flat transition table indexed by stateIdx * numSymbols + symbolIdx. Packed int32 or NO_RULE. */
  readonly rules: Int32Array;
  /** 1 if the state is accepting, 0 otherwise. Indexed by stateIdx. */
  readonly accepting: Uint8Array;
  readonly blankIdx: SymbolIdx;
  readonly stateNames: readonly string[];
  readonly symbolNames: readonly string[];
};

export type CompiledSnapshot = {
  readonly machine: CompiledMachine;
  state: StateIdx;
  tape: Int32Array;
  /** Number of valid cells in tape (tape.length may be larger due to pre-allocation). */
  tapeLen: number;
  pos: number;
};

// ════════════════════════════════════════════════════════════════════
// Compile
// ════════════════════════════════════════════════════════════════════

const compileCache = new WeakMap<
  TuringMachineSpec<string, string>,
  CompiledMachine
>();

export function compile<State extends string, Symbol extends string>(
  spec: TuringMachineSpec<State, Symbol>,
): CompiledMachine {
  const cached = compileCache.get(spec);
  if (cached) return cached;
  const { allStates, allSymbols, rules, acceptingStates, blank } = spec;
  const numStates = allStates.length;
  const numSymbols = allSymbols.length;

  const stateIndex = new Map<State, StateIdx>();
  for (let i = 0; i < numStates; i++) {
    stateIndex.set(allStates[i], i as StateIdx);
  }

  const symbolIndex = new Map<Symbol, SymbolIdx>();
  for (let i = 0; i < numSymbols; i++) {
    symbolIndex.set(allSymbols[i], i as SymbolIdx);
  }

  const flatRules = new Int32Array(numStates * numSymbols).fill(NO_RULE);

  for (const [state, symbolMap] of rules) {
    const si = stateIndex.get(state)!;
    for (const [symbol, [newState, newSymbol, dir]] of symbolMap) {
      const syi = symbolIndex.get(symbol)!;
      const nsi = stateIndex.get(newState as State)!;
      const nsyi = symbolIndex.get(newSymbol as Symbol)!;
      const d = dir === "R" ? 1 : 0;
      flatRules[si * numSymbols + syi] =
        nsi | (nsyi << SYMBOL_SHIFT) | (d << DIR_SHIFT);
    }
  }

  const accepting = new Uint8Array(numStates);
  for (const state of acceptingStates) {
    accepting[stateIndex.get(state)!] = 1;
  }

  const blankIdx = symbolIndex.get(blank)!;

  const result: CompiledMachine = {
    numStates,
    numSymbols,
    rules: flatRules,
    accepting,
    blankIdx,
    stateNames: [...allStates],
    symbolNames: [...allSymbols],
  };
  compileCache.set(spec, result);
  return result;
}

// ════════════════════════════════════════════════════════════════════
// Compile / decompile snapshots
// ════════════════════════════════════════════════════════════════════

/**
 * Convert a snapshot to compiled form.
 *
 * `tapeExtent` specifies how many cells to read from the tape overlay.
 * This is required because TapeOverlay can be infinite — we cannot iterate
 * until `get()` returns undefined. If omitted, falls back to scanning until
 * undefined (only safe for finite overlays like makeArrayTapeOverlay).
 */
export function compileSnapshot<State extends string, Symbol extends string>(
  snapshot: TuringMachineSnapshot<State, Symbol>,
  machine: CompiledMachine,
  tapeExtent?: number,
): CompiledSnapshot {
  const { state, tape, pos } = snapshot;

  // Build symbol index for the spec's symbols → machine's symbol indices
  const symbolIndex = new Map<string, SymbolIdx>();
  for (let i = 0; i < machine.symbolNames.length; i++) {
    symbolIndex.set(machine.symbolNames[i], i as SymbolIdx);
  }

  const stateIndex = new Map<string, StateIdx>();
  for (let i = 0; i < machine.stateNames.length; i++) {
    stateIndex.set(machine.stateNames[i], i as StateIdx);
  }

  // Determine how many cells to read
  let numCells: number;
  if (tapeExtent !== undefined) {
    numCells = tapeExtent;
  } else {
    // Scan until undefined — only safe for finite overlays
    numCells = 0;
    while (tape.get(numCells) !== undefined) numCells++;
  }
  numCells = Math.max(numCells, pos + 1);

  // Pre-allocate with some extra capacity
  const capacity = Math.max(numCells * 2, 64);
  const compiledTape = new Int32Array(capacity).fill(machine.blankIdx);
  for (let i = 0; i < numCells; i++) {
    const sym = tape.get(i);
    if (sym !== undefined) {
      compiledTape[i] = symbolIndex.get(sym)!;
    }
  }

  return {
    machine,
    state: stateIndex.get(state)!,
    tape: compiledTape,
    tapeLen: numCells,
    pos,
  };
}

export function decompileSnapshot<State extends string, Symbol extends string>(
  compiled: CompiledSnapshot,
  spec: TuringMachineSpec<State, Symbol>,
): TuringMachineSnapshot<State, Symbol> {
  const { machine, state, tape, tapeLen, pos } = compiled;

  // Build the tape array, trimming trailing blanks
  const symbols: Symbol[] = [];
  let lastNonBlank = -1;
  for (let i = 0; i < tapeLen; i++) {
    if (tape[i] !== machine.blankIdx) lastNonBlank = i;
  }
  for (let i = 0; i <= lastNonBlank; i++) {
    symbols.push(machine.symbolNames[tape[i]] as Symbol);
  }

  return {
    spec,
    state: spec.allStates[state] as State,
    tape: makeArrayTapeOverlay(symbols),
    pos,
  };
}

/**
 * Write a CompiledSnapshot's state back onto an existing TuringMachineSnapshot,
 * mutating it in place. Useful when the snapshot is a subclass (e.g. MyUtmSnapshot)
 * and you want to preserve the instance identity and its methods.
 */
export function writeBack<State extends string, Symbol extends string>(
  compiled: CompiledSnapshot,
  target: TuringMachineSnapshot<State, Symbol>,
): void {
  const { machine, state, tape, tapeLen, pos } = compiled;
  target.state = machine.stateNames[state] as State;
  target.pos = pos;
  for (let i = 0; i < tapeLen; i++) {
    target.tape.set(i, machine.symbolNames[tape[i]] as Symbol);
  }
}

// ════════════════════════════════════════════════════════════════════
// Fast run
// ════════════════════════════════════════════════════════════════════

export type FastRunResult =
  | { halted: true; status: "accept" | "reject"; steps: number }
  | { halted: false; steps: number };

export function fastRun(
  snapshot: CompiledSnapshot,
  { gas = 1e10 }: { gas?: number } = {},
): FastRunResult {
  const { machine } = snapshot;
  const { rules, accepting, numSymbols, blankIdx } = machine;
  let { state, pos, tape, tapeLen } = snapshot;
  let remaining = gas;
  const startGas = gas;

  while (true) {
    // Ensure tape covers current position
    if (pos >= tapeLen) {
      tapeLen = pos + 1;
    }

    // Grow tape buffer if needed
    if (pos >= tape.length) {
      const newCapacity = Math.max(tape.length * 2, pos + 1);
      const newTape = new Int32Array(newCapacity).fill(blankIdx);
      newTape.set(tape);
      tape = newTape;
    }

    const sym = tape[pos];
    const packed = rules[state * numSymbols + sym];

    if (packed === NO_RULE) {
      snapshot.state = state as StateIdx;
      snapshot.pos = pos;
      snapshot.tape = tape;
      snapshot.tapeLen = tapeLen;
      const status = accepting[state] ? "accept" : "reject";
      return { halted: true, status, steps: startGas - remaining } as const;
    }

    if (remaining <= 0) {
      snapshot.state = state as StateIdx;
      snapshot.pos = pos;
      snapshot.tape = tape;
      snapshot.tapeLen = tapeLen;
      return { halted: false, steps: startGas - remaining } as const;
    }

    const newState = packed & STATE_MASK;
    const newSymbol = (packed >>> SYMBOL_SHIFT) & SYMBOL_MASK;
    const dir = (packed >>> DIR_SHIFT) & 1;

    tape[pos] = newSymbol;
    state = newState as StateIdx;

    if (dir === 0) {
      // Left
      if (pos === 0) {
        snapshot.state = state;
        snapshot.pos = pos;
        snapshot.tape = tape;
        snapshot.tapeLen = tapeLen;
        throw new Error("Can't step machine; already at left edge of tape");
      }
      pos--;
    } else {
      pos++;
    }

    remaining--;
  }
}

/** Execute a single transition. Returns true if a step was taken, false if the machine is halted. */
export function fastStep(snapshot: CompiledSnapshot): boolean {
  const { machine } = snapshot;
  const { rules, numSymbols, blankIdx } = machine;
  const { state } = snapshot;
  let { pos, tape, tapeLen } = snapshot;

  if (pos >= tapeLen) {
    tapeLen = pos + 1;
  }
  if (pos >= tape.length) {
    const newCapacity = Math.max(tape.length * 2, pos + 1);
    const newTape = new Int32Array(newCapacity).fill(blankIdx);
    newTape.set(tape);
    tape = newTape;
  }

  const sym = tape[pos];
  const packed = rules[state * numSymbols + sym];

  if (packed === NO_RULE) {
    return false;
  }

  const newState = packed & STATE_MASK;
  const newSymbol = (packed >>> SYMBOL_SHIFT) & SYMBOL_MASK;
  const dir = (packed >>> DIR_SHIFT) & 1;

  tape[pos] = newSymbol;

  if (dir === 0) {
    if (pos === 0) {
      throw new Error("Can't step machine; already at left edge of tape");
    }
    pos--;
  } else {
    pos++;
  }

  snapshot.state = newState as StateIdx;
  snapshot.pos = pos;
  snapshot.tape = tape;
  snapshot.tapeLen = tapeLen;
  return true;
}
