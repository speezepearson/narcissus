import { describe, expect, it } from "vitest";
import { infiniteUtmTapeBackground } from "./infinite-utm";
import { MyUtmSnapshot, myUtmSpec, type MyUtmSymbol } from "./my-utm-spec";
import { must } from "./test-util";
import { makeSimpleTapeOverlay, type TapeOverlay } from "./types";
import { tapeIndexOf } from "./util";

describe("infiniteUtmTapeBackground", () => {
  it("decodes to itself", () => {
    function decode(tape: TapeOverlay<MyUtmSymbol>): TapeOverlay<MyUtmSymbol> {
      const utm = new MyUtmSnapshot({
        simSpec: myUtmSpec,
        pos: 0,
        state: myUtmSpec.initial,
        tape,
      });
      return must(utm.decode()).tape;
    }

    const tape = makeSimpleTapeOverlay(infiniteUtmTapeBackground);
    const dec1 = decode(tape);
    const dec2 = decode(dec1);
    const headerLen = must(tapeIndexOf(tape, "^"));
    for (let i = 0; i < 2 * headerLen; i++) {
      expect(dec1.get(i), `${i}`).toBe(tape.get(i));
      expect(dec2.get(i)).toBe(dec1.get(i));
    }
  });
});
