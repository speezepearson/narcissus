import { makeBreaker } from "./test-util";

export type Dir = "L" | "R";

export interface TuringMachineSpec<
  State extends string,
  Symbol extends string,
> {
  readonly allStates: ReadonlyArray<State>;
  readonly allSymbols: ReadonlyArray<Symbol>;
  readonly initial: State;
  readonly acceptingStates: ReadonlySet<State>;
  readonly blank: Symbol;
  // Not necessarily total; machine halts when no rule is applicable
  readonly rules: ReadonlyMap<State, ReadonlyMap<Symbol, [State, Symbol, Dir]>>;
}

export type TapeIdx = number;

/** A TapeOverlay represents an overlay on top of a tape full of blanks.
 * It can be lazily loaded; it can stretch off to infinity; it can have holes in it.
 * (The overlay does not know what the blank symbol is: that's the job of the TuringMachineSpec.)
 *
 * An expected normal use case:
 * - a TapeOverlay is instantiated to describe a Turing machine's initial tape
 *   (e.g. for "the machine flipBitsSpec starting with tape `01101____...`," the TapeOverlay is where the "01101" lives.)
 * - The machine runs, `set()`ting  symbols in the overlay.
 *   (e.g. after three steps, ^that machine has flipped the first three bits of the tape, so the TapeOverlay now "contains" "10001";
 *    that is, `overlay.get(0) === 1`, `overlay.get(1) === 0`, etc.)
 */
export type TapeOverlay<Symbol extends string> = {
  get(i: TapeIdx): Symbol | undefined;
  set(i: TapeIdx, sym: Symbol): void;
  clone(): TapeOverlay<Symbol>;
};
export function makeSimpleTapeOverlay<Symbol extends string>(
  background: (idx: TapeIdx) => Symbol | undefined,
  writes: Map<TapeIdx, Symbol> = new Map(),
): TapeOverlay<Symbol> {
  return {
    get(i: TapeIdx): Symbol | undefined {
      return writes.get(i) ?? background(i);
    },
    set(i: TapeIdx, sym: Symbol): void {
      if (sym === background(i)) {
        writes.delete(i);
      } else {
        writes.set(i, sym);
      }
    },
    clone(): TapeOverlay<Symbol> {
      return makeSimpleTapeOverlay(background, new Map(writes));
    },
  };
}

export type TuringMachineSnapshot<
  State extends string,
  Symbol extends string,
> = {
  spec: TuringMachineSpec<State, Symbol>;
  state: State;
  tape: TapeOverlay<Symbol>;
  pos: TapeIdx;
};

export function makeInitSnapshot<State extends string, Symbol extends string>(
  spec: TuringMachineSpec<State, Symbol>,
  tape: TapeOverlay<Symbol>,
): TuringMachineSnapshot<State, Symbol> {
  return {
    spec,
    state: spec.initial,
    tape,
    pos: 0,
  };
}
export function copySnapshot<State extends string, Symbol extends string>(
  snapshot: TuringMachineSnapshot<State, Symbol>,
): TuringMachineSnapshot<State, Symbol> {
  return {
    spec: snapshot.spec,
    state: snapshot.state,
    tape: snapshot.tape.clone(),
    pos: snapshot.pos,
  };
}

export function getRule<State extends string, Symbol extends string>(
  snapshot: TuringMachineSnapshot<State, Symbol>,
): [State, Symbol, Dir] | undefined {
  return snapshot.spec.rules
    .get(snapshot.state)
    ?.get(snapshot.tape.get(snapshot.pos) ?? snapshot.spec.blank);
}
export function getStatus<State extends string, Symbol extends string>(
  snapshot: TuringMachineSnapshot<State, Symbol>,
): "accept" | "reject" | "running" {
  const rule = getRule(snapshot);
  if (rule) return "running";
  if (snapshot.spec.acceptingStates.has(snapshot.state)) return "accept";
  return "reject";
}

export function step<State extends string, Symbol extends string>(
  tm: TuringMachineSnapshot<State, Symbol>,
): TuringMachineSnapshot<State, Symbol> {
  const rule = getRule(tm);
  if (!rule) return tm;

  const { pos } = tm;

  const [newState, newSymbol, dir] = rule;
  tm.state = newState;
  tm.tape.set(pos, newSymbol);
  if (dir === "L" && pos === 0) {
    throw new Error("Can't step machine; already at left edge of tape");
  }
  tm.pos = pos + { L: -1, R: +1 }[dir];

  return tm;
}
export async function run<TM extends TuringMachineSnapshot<string, string>>(
  snapshot: TM,
  { gas = 1e10 }: { gas?: number } = {},
): Promise<TM> {
  const breaker = makeBreaker();
  while (getStatus(snapshot) === "running") {
    if (gas <= 0) {
      throw new Error("Gas limit exceeded");
    }
    step(snapshot);
    await breaker();
    gas--;
  }

  return snapshot;
}

export type UtmSpec<
  UState extends string,
  USymbol extends string,
> = TuringMachineSpec<UState, USymbol> & {
  encode<SimState extends string, SimSymbol extends string>(
    snapshot: TuringMachineSnapshot<SimState, SimSymbol>,
    opts?: { optimizationHints?: Array<[SimState, SimSymbol]> },
  ): UtmSnapshot<UState, USymbol, SimState, SimSymbol>;
};

export type UtmSnapshot<
  UState extends string,
  USymbol extends string,
  SimState extends string,
  SimSymbol extends string,
> = TuringMachineSnapshot<UState, USymbol> & {
  /** Decodes the tape of a running UTM into a snapshot of the simulated machine. May return undefined if the UTM is mid-operation.
   * Should always yield the simulated TM's snapshots in-order: so if the simulated TM is in state X, then steps to Y, then steps to Z,
   * then, when we simulate it with a UTM, `decode()`ing the UTM's tape should return:
   * - (X/undefined) for a while...
   * - then (Y/undefined) for a while...
   * - then (Z/undefined) for a while...
   *
   * If `optimizationHints.sparse` is true (the default), the method should try to return undefined more often (but still at least once per simulated step),
   * in order to reduce the amount of decoding work done.
   */
  decode(optimizationHints?: {
    sparse?: boolean;
  }): undefined | TuringMachineSnapshot<SimState, SimSymbol>;
};

export function assertNever(x: never): never {
  throw new Error(`Unexpected value: ${x}`);
}

// ════════════════════════════════════════════════════════════════════
// Branded index types
// ════════════════════════════════════════════════════════════════════

declare const StateIdxBrand: unique symbol;
declare const SymbolIdxBrand: unique symbol;
export type StateIdx = number & { readonly [StateIdxBrand]: true };
export type SymbolIdx = number & { readonly [SymbolIdxBrand]: true };
