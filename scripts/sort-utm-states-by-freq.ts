import {
  myUtmSpec,
  type MyUtmState,
  type MyUtmSymbol,
} from "../src/my-utm-spec";
import { flipBitsSpec } from "../src/toy-machines";
import { getStatus, makeInitSnapshot, step, type UtmSpec } from "../src/types";
import { makeArrayTapeOverlay } from "../src/util";

type Stats<State extends string, Symbol extends string> = {
  steps: number;
  innerSteps: number;
  states: {
    counts: Partial<Record<State, number>>;
    sortedByFreqAsc: State[];
    freqStats: Partial<
      Record<
        State,
        {
          freq: number;
          cumFreq: number;
        }
      >
    >;
  };
  symbols: {
    counts: Partial<Record<Symbol, number>>;
    sortedByFreqDesc: Symbol[];
    freqStats: Partial<
      Record<
        Symbol,
        {
          freq: number;
          cumFreq: number;
        }
      >
    >;
  };
};

function getFreqInfo<T extends string>(
  vals: readonly T[],
  counts: Partial<Record<T, number>>,
): Record<T, { freq: number; cumFreq: number }> {
  const sum = [...vals].reduce((acc, b) => acc + (counts[b] ?? 0), 0);
  const freqs = {} as Record<T, number>;
  for (const key of vals) {
    freqs[key as T] = (counts[key] ?? 0) / sum;
  }

  const result = {} as Record<T, { freq: number; cumFreq: number }>;
  let cumFreq = 0;
  for (const key of vals.slice().sort((a, b) => freqs[a] - freqs[b])) {
    cumFreq += freqs[key];
    result[key] = { freq: freqs[key], cumFreq };
  }
  return result;
}

function getStats<State extends string, Symbol extends string>(
  utmSpec: UtmSpec<State, Symbol>,
  maxInnerSteps: number,
): Stats<State, Symbol> {
  let base;
  {
    base = makeInitSnapshot(flipBitsSpec, makeArrayTapeOverlay(["0", "1"]));
    base = myUtmSpec.encode(base);
    base = utmSpec.encode(base);
  }
  const simulator = utmSpec.encode(base);
  const doubleSimulator = utmSpec.encode(simulator);
  let steps = 0;
  let innerSteps = 0;
  const stateCounts: Partial<Record<State, number>> = {};
  const symbolCounts: Partial<Record<Symbol, number>> = {};

  while (true) {
    if (getStatus(step(doubleSimulator)) !== "running") {
      break;
    }
    stateCounts[doubleSimulator.state] =
      (stateCounts[doubleSimulator.state] ?? 0) + 1;
    const sym = doubleSimulator.tape.get(doubleSimulator.pos) ?? utmSpec.blank;
    symbolCounts[sym] = (symbolCounts[sym] ?? 0) + 1;

    const decoded = doubleSimulator.decode();
    if (decoded && decoded.pos !== simulator.pos) {
      innerSteps++;
      console.error(`tick ${innerSteps}/${maxInnerSteps}`);
      if (innerSteps === maxInnerSteps) break;
    }
    steps++;
  }

  console.log(`took ${steps} steps`);
  const stateFreqs = getFreqInfo(utmSpec.allStates, stateCounts);
  const symbolFreqs = getFreqInfo(utmSpec.allSymbols, symbolCounts);

  return {
    steps,
    innerSteps,
    states: {
      counts: stateCounts,
      sortedByFreqAsc: utmSpec.allStates
        .slice()
        .sort((a, b) => stateFreqs[a].freq - stateFreqs[b].freq),
      freqStats: stateFreqs,
    },
    symbols: {
      counts: symbolCounts,
      sortedByFreqDesc: utmSpec.allSymbols
        .slice()
        .sort((a, b) => symbolFreqs[b].freq - symbolFreqs[a].freq),
      freqStats: symbolFreqs,
    },
  };
}

function percentageDiff(a: number, b: number) {
  return (Math.abs(a - b) / Math.max(a, b)) * 100;
}

let spec = myUtmSpec;
let lastStats: Stats<MyUtmState, MyUtmSymbol> | undefined;
let nInnerSteps = 4;
while (true) {
  const t0 = performance.now();
  const stats = getStats(spec, nInnerSteps);
  const t1 = performance.now();
  console.log(stats);
  console.log(`took ${t1 - t0}ms`);
  console.log("states recommendation:");
  console.log("  ", JSON.stringify(stats.states.sortedByFreqAsc));
  console.log("symbols recommendation:");
  console.log("  ", JSON.stringify(stats.symbols.sortedByFreqDesc));
  if (lastStats) {
    const lastStatsConst = lastStats;

    // STATES
    const lastStateFreq = (st: MyUtmState) =>
      lastStatsConst.states.freqStats[st]?.freq ?? 0;
    const newStateFreq = (st: MyUtmState) =>
      stats.states.freqStats[st]?.freq ?? 0;
    const changedStates = spec.allStates.filter(
      (st) =>
        percentageDiff(lastStateFreq(st), newStateFreq(st)) > 1 &&
        newStateFreq(st) > 1 / (10 * spec.allStates.length),
    );
    console.log(`${changedStates.length} states changed:`);
    for (const st of changedStates) {
      console.log(
        `  ${st.padEnd(20)} ${lastStateFreq(st).toFixed(6)} -> ${newStateFreq(st).toFixed(6)} (${percentageDiff(lastStateFreq(st), newStateFreq(st)).toFixed(2)}%)`,
      );
    }
    if (changedStates.length === 0) {
      console.log("reached fixed point, or close enough");
      break;
    }

    // SYMBOLS
    const lastSymbolFreq = (st: MyUtmSymbol) =>
      lastStatsConst.symbols.freqStats[st]?.freq ?? 0;
    const newSymbolFreq = (st: MyUtmSymbol) =>
      stats.symbols.freqStats[st]?.freq ?? 0;
    const changedSymbols = spec.allSymbols.filter(
      (st) =>
        percentageDiff(lastSymbolFreq(st), newSymbolFreq(st)) > 1 &&
        newSymbolFreq(st) > 1 / (10 * spec.allSymbols.length),
    );
    console.log(`${changedSymbols.length} symbols changed:`);
    for (const st of changedSymbols) {
      console.log(
        `  ${st.padEnd(20)} ${lastSymbolFreq(st).toFixed(6)} -> ${newSymbolFreq(st).toFixed(6)} (${percentageDiff(lastSymbolFreq(st), newSymbolFreq(st)).toFixed(2)}%)`,
      );
    }
    if (changedSymbols.length === 0) {
      console.log("reached fixed point, or close enough");
      break;
    }
  }
  lastStats = stats;
  spec = {
    ...spec,
    allStates: stats.states.sortedByFreqAsc,
    allSymbols: stats.symbols.sortedByFreqDesc,
  };
  nInnerSteps *= 2;
}
