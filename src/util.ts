import {
  getStatus,
  step,
  type TapeOverlay,
  type TuringMachineSnapshot,
  type UtmSnapshot,
} from "./types";

export function tapesEqual<Tape extends readonly string[]>(
  a: Tape,
  b: Tape,
  blank: Tape[number],
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

export function indexOf<T>(
  array: readonly T[],
  value: T,
  start?: number,
): number | undefined {
  const index = array.indexOf(value, start);
  if (index === -1) {
    return undefined;
  }
  return index;
}

export function makeArrayTapeOverlay<Symbol extends string>(
  array: Symbol[],
): TapeOverlay<Symbol> {
  return {
    get(i: number): Symbol | undefined {
      return array[i];
    },
    set(i: number, sym: Symbol): void {
      array[i] = sym;
    },
    clone(): TapeOverlay<Symbol> {
      return makeArrayTapeOverlay(array);
    },
  };
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
