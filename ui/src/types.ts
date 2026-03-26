import { z } from "zod";
import { makeBreaker } from "./test-util";

export type Dir = "L" | "R";

export const State = z.string().brand<"State">();
export type State = z.infer<typeof State>;
export const Symbol = z.string().brand<"Symbol">();
export type Symbol = z.infer<typeof Symbol>;

export interface TuringMachineSpec {
  readonly allStates: ReadonlyArray<State>;
  readonly allSymbols: ReadonlyArray<Symbol>;
  readonly initial: State;
  readonly acceptingStates: ReadonlySet<State>;
  readonly blank: Symbol;
  // Not necessarily total; machine halts when no rule is applicable
  readonly rules: ReadonlyMap<State, ReadonlyMap<Symbol, [State, Symbol, Dir]>>;
}

export type TapeIdx = number;

export type TuringMachineSnapshot = {
  spec: TuringMachineSpec;
  state: State;
  tape: Symbol[];
  pos: TapeIdx;
};

export function makeInitSnapshot(
  spec: TuringMachineSpec,
  tape: readonly Symbol[],
): TuringMachineSnapshot {
  return {
    spec,
    state: spec.initial,
    tape: tape.slice(),
    pos: 0,
  };
}
export function copySnapshot(
  snapshot: TuringMachineSnapshot,
): TuringMachineSnapshot {
  return {
    spec: snapshot.spec,
    state: snapshot.state,
    tape: snapshot.tape.slice(),
    pos: snapshot.pos,
  };
}

export function getRule(
  snapshot: TuringMachineSnapshot,
): [State, Symbol, Dir] | undefined {
  return snapshot.spec.rules
    .get(snapshot.state)
    ?.get(snapshot.tape[snapshot.pos] ?? snapshot.spec.blank);
}
export function getStatus(
  snapshot: TuringMachineSnapshot,
): "accept" | "reject" | "running" {
  const rule = getRule(snapshot);
  if (rule) return "running";
  if (snapshot.spec.acceptingStates.has(snapshot.state)) return "accept";
  return "reject";
}

export function step(tm: TuringMachineSnapshot): TuringMachineSnapshot {
  const rule = getRule(tm);
  if (!rule) return tm;

  const { pos } = tm;

  const [newState, newSymbol, dir] = rule;
  tm.state = newState;
  tm.tape[pos] = newSymbol;
  if (dir === "L" && pos === 0) {
    throw new Error("Can't step machine; already at left edge of tape");
  }
  tm.pos = pos + { L: -1, R: +1 }[dir];

  return tm;
}
export async function run(
  snapshot: TuringMachineSnapshot,
  { gas = 1e10 }: { gas?: number } = {},
): Promise<TuringMachineSnapshot> {
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
