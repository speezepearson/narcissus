import {
  getStatus,
  makeSimpleTapeOverlay,
  step,
  type StateIdx,
  type SymbolIdx,
  type TapeIdx,
  type TapeOverlay,
  type TuringMachineSnapshot,
  type UtmSnapshot,
} from "./types";

export function tapesEqual<Tape extends readonly string[]>(
  a: Tape,
  b: Tape,
  blank: Tape[TapeIdx],
): boolean {
  if (a.length > b.length) return tapesEqual(b, a, blank);
  if (a.length < b.length) {
    for (let i = a.length; i < b.length; i++) {
      if (b[i] !== blank) return false;
    }
  }
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return false;
  }
  return true;
}

export function must<T>(x: T | undefined): T {
  if (x === undefined) {
    throw new Error("expected non-undefined");
  }
  return x;
}

export function indexOf<Arr extends ReadonlyArray<unknown>>(
  array: Arr,
  value: Arr[TapeIdx],
  start?: TapeIdx,
): TapeIdx | undefined {
  const index = array.indexOf(value, start);
  if (index === -1) {
    return undefined;
  }
  return index;
}

export function makeArrayTapeOverlay<Symbol extends string>(
  array: readonly Symbol[],
): TapeOverlay<Symbol> {
  array = array.slice();
  return makeSimpleTapeOverlay((i) => array[i]);
}

export function runUntilInnerStep<
  UState extends string,
  USymbol extends string,
  SimState extends string,
  SimSymbol extends string,
>(
  utm: UtmSnapshot<UState, USymbol, SimState, SimSymbol>,
):
  | { type: "halt"; status: "accept" | "reject" }
  | { type: "stepped"; decoded: TuringMachineSnapshot<SimState, SimSymbol> } {
  const initPos = utm.decode()?.pos;
  let status: "accept" | "reject" | "running" = "running";
  while (status === "running") {
    status = getStatus(utm);
    const decoded = utm.decode();
    if (decoded && decoded.pos !== initPos) {
      return { type: "stepped", decoded };
    }
    step(utm);
  }
  return { type: "halt", status };
}

/** Search a TapeOverlay for `value` starting at index `start`, scanning until `get()` returns undefined. */
export function tapeIndexOf<Symbol extends string>(
  tape: TapeOverlay<Symbol>,
  value: Symbol,
  start: number = 0,
): number | undefined {
  for (let i = start; ; i++) {
    const v = tape.get(i);
    if (v === undefined) return undefined;
    if (v === value) return i;
  }
}

export function mustStateIndex<State extends string>(
  states: readonly State[],
  state: State,
): StateIdx {
  return must(indexOf(states, state)) as StateIdx;
}

export function mustSymbolIndex<Symbol extends string>(
  symbols: readonly Symbol[],
  symbol: Symbol,
): SymbolIdx {
  return must(indexOf(symbols, symbol)) as SymbolIdx;
}
