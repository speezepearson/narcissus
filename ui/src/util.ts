import { type StateIdx, type SymbolIdx, type TapeIdx } from "./types";

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
