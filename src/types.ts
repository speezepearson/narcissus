export type Dir = "L" | "R";

export type TuringMachineSpec<State extends string, Symbol extends string> = {
  readonly allStates: ReadonlyArray<State>;
  readonly allSymbols: ReadonlyArray<Symbol>;
  readonly initial: State;
  readonly acceptingStates: ReadonlySet<State>;
  readonly blank: Symbol;
  // Not necessarily total; machine halts when no rule is applicable
  readonly rules: ReadonlyMap<State, ReadonlyMap<Symbol, [State, Symbol, Dir]>>;
};

export type TuringMachineSnapshot<
  State extends string,
  Symbol extends string,
> = {
  spec: TuringMachineSpec<State, Symbol>;
  state: State;
  tape: Symbol[];
  pos: number;
};

export function makeInitSnapshot<State extends string, Symbol extends string>(
  spec: TuringMachineSpec<State, Symbol>,
  tape: readonly Symbol[] = [],
): TuringMachineSnapshot<State, Symbol> {
  return {
    spec,
    state: spec.initial,
    tape: tape.slice(),
    pos: 0,
  };
}
export function copySnapshot<State extends string, Symbol extends string>(
  snapshot: TuringMachineSnapshot<State, Symbol>,
): TuringMachineSnapshot<State, Symbol> {
  return {
    spec: snapshot.spec,
    state: snapshot.state,
    tape: snapshot.tape.slice(),
    pos: snapshot.pos,
  };
}

export function getRule<State extends string, Symbol extends string>(
  snapshot: TuringMachineSnapshot<State, Symbol>,
): [State, Symbol, Dir] | undefined {
  while (snapshot.tape.length <= snapshot.pos) {
    snapshot.tape.push(snapshot.spec.blank);
  }
  return snapshot.spec.rules
    .get(snapshot.state)
    ?.get(snapshot.tape[snapshot.pos]);
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

  const pos = tm.pos;

  const [newState, newSymbol, dir] = rule;
  tm.state = newState;
  tm.tape[pos] = newSymbol;
  if (dir === "L" && pos === 0) {
    throw new Error("Can't step machine; already at left edge of tape");
  }
  tm.pos = { L: pos - 1, R: pos + 1 }[dir];

  return tm;
}
export function run<State extends string, Symbol extends string>(
  snapshot: TuringMachineSnapshot<State, Symbol>,
  { gas = 1e10 }: { gas?: number } = {},
): TuringMachineSnapshot<State, Symbol> {
  while (getStatus(snapshot) === "running") {
    if (gas <= 0) {
      throw new Error("Gas limit exceeded");
    }
    step(snapshot);
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
  ): USymbol[];

  /** Decodes the tape of a running UTM into a snapshot of the simulated machine. May return undefined if the UTM is mid-operation.
   * Should always yield the simulated TM's snapshots in-order: so if the simulated TM is in state X, then steps to Y, then steps to Z,
   * then, when we simulate it with a UTM, `decode()`ing the UTM's tape should return:
   * - (X/undefined) for a while...
   * - then (Y/undefined) for a while...
   * - then (Z/undefined) for a while...
   */
  decode<SimState extends string, SimSymbol extends string>(
    spec: TuringMachineSpec<SimState, SimSymbol>,
    utm: TuringMachineSnapshot<UState, USymbol>,
  ): undefined | TuringMachineSnapshot<SimState, SimSymbol>;
};

export function assertNever(x: never): never {
  throw new Error(`Unexpected value: ${x}`);
}
