import {
  myUtmSpec,
  type MyUtmState,
  type MyUtmSymbol,
} from "../src/my-utm-spec";
import { flipBitsSpec } from "../src/toy-machines";
import { getStatus, makeInitSnapshot, step, type UtmSpec } from "../src/types";
import { makeArrayTapeOverlay } from "../src/util";
import myUtmOptimizationHints from "../src/my-utm-spec-transition-optimization-hints";

type Stats<State extends string, Symbol extends string> = {
  steps: number;
  innerSteps: number;
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
  maxInnerSteps: number,
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
  let steps = 0;
  let innerSteps = 0;
  const transitionCounts: Map<State, Map<Symbol, number>> = new Map();

  while (true) {
    if (getStatus(step(doubleSimulator)) !== "running") {
      break;
    }
    const sym = doubleSimulator.tape.get(doubleSimulator.pos) ?? utmSpec.blank;
    incrementCount(transitionCounts, doubleSimulator.state, sym);

    const decoded = doubleSimulator.decode();
    if (decoded && decoded.pos !== simulator.pos) {
      innerSteps++;
      console.error(`tick ${innerSteps}/${maxInnerSteps}`);
      if (innerSteps === maxInnerSteps) break;
    }
    steps++;
  }

  console.log(`took ${steps} steps`);
  const transitionFreqs = getFreqInfo(transitionCounts);

  return {
    steps,
    innerSteps,
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
let nInnerSteps = 4;
while (true) {
  const t0 = performance.now();
  const stats = getStats(
    myUtmSpec,
    nInnerSteps,
    lastStats?.transitions.sortedByFreqAsc ?? myUtmOptimizationHints,
  );
  const t1 = performance.now();
  console.log(stats);
  console.log(`took ${t1 - t0}ms`);
  console.log("transitions recommendation:");
  console.log("  ", JSON.stringify(stats.transitions.sortedByFreqAsc));
  // if (lastStats) {
  //   const lastStatsConst = lastStats;

  //   // STATES
  //   const lastStateFreq = (st: MyUtmState) =>
  //     lastStatsConst.states.freqStats[st]?.freq ?? 0;
  //   const newStateFreq = (st: MyUtmState) =>
  //     stats.states.freqStats[st]?.freq ?? 0;
  //   const changedStates = spec.allStates.filter(
  //     (st) =>
  //       percentageDiff(lastStateFreq(st), newStateFreq(st)) > 1 &&
  //       newStateFreq(st) > 1 / (10 * spec.allStates.length),
  //   );
  //   console.log(`${changedStates.length} states changed:`);
  //   for (const st of changedStates) {
  //     console.log(
  //       `  ${st.padEnd(20)} ${lastStateFreq(st).toFixed(6)} -> ${newStateFreq(st).toFixed(6)} (${percentageDiff(lastStateFreq(st), newStateFreq(st)).toFixed(2)}%)`,
  //     );
  //   }
  //   if (changedStates.length === 0) {
  //     console.log("reached fixed point, or close enough");
  //     break;
  //   }

  //   // SYMBOLS
  //   const lastSymbolFreq = (st: MyUtmSymbol) =>
  //     lastStatsConst.symbols.freqStats[st]?.freq ?? 0;
  //   const newSymbolFreq = (st: MyUtmSymbol) =>
  //     stats.symbols.freqStats[st]?.freq ?? 0;
  //   const changedSymbols = spec.allSymbols.filter(
  //     (st) =>
  //       percentageDiff(lastSymbolFreq(st), newSymbolFreq(st)) > 1 &&
  //       newSymbolFreq(st) > 1 / (10 * spec.allSymbols.length),
  //   );
  //   console.log(`${changedSymbols.length} symbols changed:`);
  //   for (const st of changedSymbols) {
  //     console.log(
  //       `  ${st.padEnd(20)} ${lastSymbolFreq(st).toFixed(6)} -> ${newSymbolFreq(st).toFixed(6)} (${percentageDiff(lastSymbolFreq(st), newSymbolFreq(st)).toFixed(2)}%)`,
  //     );
  //   }
  //   if (changedSymbols.length === 0) {
  //     console.log("reached fixed point, or close enough");
  //     break;
  //   }
  // }
  lastStats = stats;
  nInnerSteps *= 2;
}
