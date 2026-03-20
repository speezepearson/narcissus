import {
  myUtmSpec,
  type MyUtmState,
  type MyUtmSymbol,
} from "../src/my-utm-spec";
import myUtmOptimizationHints from "../src/my-utm-spec-transition-optimization-hints";
import { flipBitsSpec } from "../src/toy-machines";
import {
  getRule,
  makeInitSnapshot,
  step,
  type TuringMachineSnapshot,
  type UtmSpec,
} from "../src/types";
import { makeArrayTapeOverlay } from "../src/util";

type Stats<State extends string, Symbol extends string> = {
  steps: number;
  transitions: {
    counts: Map<State, Map<Symbol, number>>;
    sortedByFreqAsc: Array<[State, Symbol]>;
    freqStats: Map<State, Map<Symbol, { freq: number; cumFreq: number }>>;
  };
};

function getFreqInfo<State extends string, Symbol extends string>(
  counts: Map<State, Map<Symbol, number>>,
): Map<State, Map<Symbol, { freq: number; cumFreq: number }>> {
  const sum = deepEntries(counts).reduce((acc, [, , count]) => acc + count, 0);
  const freqs = new Map(
    deepEntries(counts).map(([state, sym, count]) => [
      state,
      new Map([[sym, count / sum] as const]),
    ]),
  );

  const transitions: Array<readonly [State, Symbol]> = deepEntries(counts).map(
    ([st, sym]) => [st, sym],
  );
  transitions.sort(
    (a, b) => getCount(counts, a[0], a[1]) - getCount(counts, b[0], b[1]),
  );
  const result = new Map<
    State,
    Map<Symbol, { freq: number; cumFreq: number }>
  >();
  let cumFreq = 0;
  for (const [state, sym] of transitions) {
    cumFreq += getCount(freqs, state, sym) / sum;
    if (!result.has(state)) result.set(state, new Map());
    result
      .get(state)!
      .set(sym, { freq: getCount(counts, state, sym) / sum, cumFreq });
  }

  return result;
}

function getCount<K1, K2>(
  m: Map<K1, Map<K2, number>>,
  state: K1,
  sym: K2,
): number {
  return m.get(state)?.get(sym) ?? 0;
}
function incrementCount<K1, K2>(
  counts: Map<K1, Map<K2, number>>,
  state: K1,
  sym: K2,
) {
  if (!counts.has(state)) counts.set(state, new Map());
  counts.get(state)!.set(sym, (counts.get(state)!.get(sym) ?? 0) + 1);
}
function deepEntries<K1, K2, V>(
  m: Map<K1, Map<K2, V>>,
): Array<readonly [K1, K2, V]> {
  return [...m.entries()].flatMap(([state, syms]) =>
    [...syms.entries()].map(([sym, v]) => [state, sym, v] as const),
  );
}

function getStats<State extends string, Symbol extends string>(
  utmSpec: UtmSpec<State, Symbol>,
  maxSteps: number,
  optimizationHints: Array<[State, Symbol]> = [],
): Stats<State, Symbol> {
  let base;
  {
    base = makeInitSnapshot(flipBitsSpec, makeArrayTapeOverlay(["0", "1"]));
    base = utmSpec.encode(base);
    base = utmSpec.encode(base, { optimizationHints });
  }
  const simulator = utmSpec.encode(base, { optimizationHints });
  const doubleSimulator = utmSpec.encode(simulator, { optimizationHints });

  const transitionCounts = new Map<State, Map<Symbol, number>>();

  const snap: TuringMachineSnapshot<State, Symbol> = doubleSimulator;
  for (let i = 0; i < maxSteps; i++) {
    const sym = snap.tape.get(snap.pos) ?? snap.spec.blank;
    incrementCount(transitionCounts, snap.state, sym);
    if (!getRule(snap)) break;
    step(snap);
  }

  const transitionFreqs = getFreqInfo(transitionCounts);

  return {
    steps: maxSteps,
    transitions: {
      counts: transitionCounts,
      sortedByFreqAsc: deepEntries(transitionFreqs)
        .sort(([, , a], [, , b]) => a.freq - b.freq)
        .map(([st, sym]) => [st, sym]),
      freqStats: transitionFreqs,
    },
  };
}

let lastStats: Stats<MyUtmState, MyUtmSymbol> | undefined;
let maxSteps = 1e10;
while (true) {
  const t0 = performance.now();
  const stats = getStats(
    myUtmSpec,
    maxSteps,
    lastStats?.transitions.sortedByFreqAsc ?? myUtmOptimizationHints,
  );
  const t1 = performance.now();
  console.log(stats);
  console.log(`took ${t1 - t0}ms`);
  console.log("transitions recommendation:");
  console.log("  ", JSON.stringify(stats.transitions.sortedByFreqAsc));
  lastStats = stats;
  maxSteps *= 2;
}
