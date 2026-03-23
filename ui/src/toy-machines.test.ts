import { describe, expect, it } from "vitest";
import {
  getStatus,
  makeInitSnapshot,
  run,
  step,
  type TapeOverlay,
} from "./types";
import {
  acceptImmediatelySpec,
  checkPalindromeSpec,
  doubleXSpec,
  flipBitsSpec,
  rejectImmediatelySpec,
  type DoubleXSymbol,
} from "./toy-machines";
import { makeArrayTapeOverlay } from "./util";

describe("acceptImmediatelySpec", () => {
  it("terminates with accept", () => {
    expect(
      getStatus(
        step(makeInitSnapshot(acceptImmediatelySpec, makeArrayTapeOverlay([]))),
      ),
    ).toBe("accept");
  });
});

describe("rejectImmediatelySpec", () => {
  it("terminates with reject", () => {
    expect(
      getStatus(
        step(makeInitSnapshot(rejectImmediatelySpec, makeArrayTapeOverlay([]))),
      ),
    ).toBe("reject");
  });
});

describe("flipBitsSpec", () => {
  it("flips bits", async () => {
    const tm = await run(
      makeInitSnapshot(
        flipBitsSpec,
        makeArrayTapeOverlay(["0", "1", "0", "1", "1"]),
      ),
    );
    expect(getStatus(tm)).toBe("accept");
    ["1", "0", "1", "0", "0"].forEach((sym, i) => {
      expect(tm.tape.get(i)).toBe(sym);
    });
    expect(tm.tape.get(5)).toSatisfy((sym) => sym === "_" || sym === undefined);
  });
});

describe("doubleXSpec", () => {
  function makeXTape(n: number): TapeOverlay<DoubleXSymbol> {
    return makeArrayTapeOverlay([
      "$",
      ...Array.from({ length: n }, (): "X" => "X"),
    ]);
  }

  function expectedTape(n: number): ("$" | "X" | "_")[] {
    if (n === 0) return ["$", "_"];
    return ["$", ...Array.from({ length: 2 * n }, (): "X" => "X"), "_"];
  }

  it.each([0, 1, 2, 3])("doubles %i X's", async (n) => {
    const tm = await run(makeInitSnapshot(doubleXSpec, makeXTape(n)));
    expect(getStatus(tm)).toBe("accept");
    [...expectedTape(n), undefined].forEach((sym, i) => {
      expect(tm.tape.get(i)).toBe(sym);
    });
  });
});

describe("checkPalindromeSpec", () => {
  it("accepts an even-length palindrome", async () => {
    const tm = await run(
      makeInitSnapshot(
        checkPalindromeSpec,
        makeArrayTapeOverlay(["a", "b", "b", "a"]),
      ),
    );
    expect(getStatus(tm)).toBe("accept");
  });
  it("accepts an odd-length palindrome", async () => {
    const tm = await run(
      makeInitSnapshot(
        checkPalindromeSpec,
        makeArrayTapeOverlay(["a", "b", "b", "b", "a"]),
      ),
    );
    expect(getStatus(tm)).toBe("accept");
  });
  it("rejects a non-palindrome", async () => {
    const tm = await run(
      makeInitSnapshot(
        checkPalindromeSpec,
        makeArrayTapeOverlay(["a", "b", "b", "a", "b"]),
      ),
    );
    expect(getStatus(tm)).toBe("reject");
  });
});
