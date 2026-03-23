import { myUtmSpec, numBits, toBinary, type MyUtmSymbol } from "./my-utm-spec";
import myUtmOptimizationHints from "./my-utm-spec-transition-optimization-hints";
import { makeInitSnapshot, type TapeIdx } from "./types";
import {
  makeArrayTapeOverlay,
  must,
  mustSymbolIndex,
  tapeIndexOf,
} from "./util";

const header = (() => {
  const baseUtm = myUtmSpec.encode(
    makeInitSnapshot(myUtmSpec, makeArrayTapeOverlay([])),
    { optimizationHints: myUtmOptimizationHints },
  );
  const headerEnd = must(tapeIndexOf(baseUtm.tape, "^"));
  return Array.from({ length: headerEnd }, (_, i) => must(baseUtm.tape.get(i)));
})();
const nSymBits = numBits(myUtmSpec.allSymbols.length);
const cellSize = 1 + nSymBits;

export function infiniteUtmTapeBackground(idx: TapeIdx): MyUtmSymbol {
  if (idx < header.length) return header[idx];
  const cellIdx = Math.floor((idx - header.length) / cellSize);
  const within = (idx - header.length) % cellSize;
  if (within === 0) {
    return cellIdx === 0 ? "^" : ",";
  }
  const x = toBinary(
    mustSymbolIndex(myUtmSpec.allSymbols, infiniteUtmTapeBackground(cellIdx)),
    nSymBits,
  );
  return x[within - 1];
}
