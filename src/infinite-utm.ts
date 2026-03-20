import {
  MyUtmSnapshot,
  myUtmSpec,
  numBits,
  toBinary,
  type MyUtmState,
  type MyUtmSymbol,
} from "./my-utm-spec";
import myUtmOptimizationHints from "./my-utm-spec-transition-optimization-hints";
import { makeInitSnapshot, type TapeOverlay } from "./types";
import { indexOf, makeArrayTapeOverlay, must, tapeIndexOf } from "./util";

export class InfiniteUtm extends MyUtmSnapshot<MyUtmState, MyUtmSymbol> {
  constructor({
    pos = 0,
    state = myUtmSpec.initial,
    tape = makeInfiniteUtmTape(),
  }: {
    pos?: number;
    state?: MyUtmState;
    tape?: TapeOverlay<MyUtmSymbol>;
  } = {}) {
    super({
      simSpec: myUtmSpec,
      pos,
      state,
      tape,
    });
  }

  override decode(optimizationHints?: {
    sparse?: boolean;
  }): InfiniteUtm | undefined {
    const plain = super.decode(optimizationHints);
    if (plain === undefined) return undefined;
    return new InfiniteUtm({
      pos: plain.pos,
      state: plain.state,
      tape: plain.tape, // TODO: do we need to manipulate this?
    });
  }
}

const header = (() => {
  const baseUtm = myUtmSpec.encode(
    makeInitSnapshot(myUtmSpec, makeArrayTapeOverlay([])),
    { optimizationHints: myUtmOptimizationHints },
  );
  const headerEnd = must(tapeIndexOf(baseUtm.tape, "^"));
  return Array.from({ length: headerEnd }, (_, i) => must(baseUtm.tape.get(i)));
})();
const nSymBits = numBits(myUtmSpec.allStates.length);
const cellSize = 1 + nSymBits;

function getBackgroundAt(idx: number): MyUtmSymbol {
  if (idx < header.length) return header[idx];
  const cellIdx = Math.floor((idx - header.length) / cellSize);
  const within = (idx - header.length) % cellSize;
  if (within === 0) {
    return cellIdx === 0 ? "^" : ",";
  }
  const x = toBinary(
    must(indexOf(myUtmSpec.allSymbols, getBackgroundAt(cellIdx))),
    nSymBits,
  );
  return x[within - 1];
}

function makeInfiniteUtmTape(
  overrides?: Map<number, MyUtmSymbol>,
): TapeOverlay<MyUtmSymbol> {
  overrides ??= new Map<number, MyUtmSymbol>();
  return {
    get(idx: number): MyUtmSymbol | undefined {
      return overrides.get(idx) ?? getBackgroundAt(idx);
    },
    set(idx: number, sym: MyUtmSymbol): void {
      overrides.set(idx, sym);
    },
    clone(): TapeOverlay<MyUtmSymbol> {
      return makeInfiniteUtmTape(new Map(overrides));
    },
  };
}

const t = makeInfiniteUtmTape();
console.log(Array.from({ length: 1e5 }, (_, i) => t.get(i)).join(""));
