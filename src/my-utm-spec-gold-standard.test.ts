/**
 * DO NOT MODIFY THIS FILE.
 */

import { describe, expect, it } from "vitest";
import {
  copySnapshot,
  getStatus,
  makeInitSnapshot,
  run,
  step,
  type TuringMachineSnapshot,
} from "./types";
import { myUtmSpec } from "./my-utm-spec";
import {
  acceptImmediatelySpec,
  checkPalindromeSpec,
  doubleXSpec,
  flipBitsSpec,
  rejectImmediatelySpec,
} from "./toy-machines";
import { makeArrayTapeOverlay, runUntilInnerStep } from "./util";
import { must } from "./test-util";

function listAllSnapshots<State extends string, Symbol extends string>(
  tm: TuringMachineSnapshot<State, Symbol>,
): Array<TuringMachineSnapshot<State, Symbol>> {
  const res = [];
  while (getStatus(tm) === "running") {
    res.push(copySnapshot(tm));
    step(tm);
  }
  res.push(tm);
  return res;
}

function expectTmsEqual<State extends string, Symbol extends string>(
  a: TuringMachineSnapshot<State, Symbol>,
  b: TuringMachineSnapshot<State, Symbol>,
): void {
  expect(a.spec).toEqual(b.spec);
  expect(a.state).toEqual(b.state);
  expect(a.pos).toEqual(b.pos);

  const blank = a.spec.blank;
  let i = 0;
  while (true) {
    const bSym = b.tape.get(i);
    const aSym = a.tape.get(i);
    if (aSym === undefined && bSym === undefined) {
      break;
    }
    expect(aSym ?? blank).toEqual(bSym ?? blank);
    i++;
  }
}

const variousSnapshots = [
  makeInitSnapshot(acceptImmediatelySpec, makeArrayTapeOverlay([])),
  makeInitSnapshot(rejectImmediatelySpec, makeArrayTapeOverlay([])),
  ...listAllSnapshots(
    makeInitSnapshot(flipBitsSpec, makeArrayTapeOverlay(["0", "1"])),
  ),
  ...listAllSnapshots(
    makeInitSnapshot(doubleXSpec, makeArrayTapeOverlay(["$", "X", "X"])),
  ),
  ...listAllSnapshots(
    makeInitSnapshot(checkPalindromeSpec, makeArrayTapeOverlay(["a", "a"])),
  ),
  ...listAllSnapshots(
    makeInitSnapshot(checkPalindromeSpec, makeArrayTapeOverlay(["b", "b"])),
  ),
  ...listAllSnapshots(
    makeInitSnapshot(checkPalindromeSpec, makeArrayTapeOverlay(["a", "b"])),
  ),
];

describe("myUtmSpec gold standard tests", () => {
  describe("decode", () => {
    it.each(variousSnapshots)("inverts encode", (tm) => {
      const roundtrip = must(myUtmSpec.encode(tm).decode());
      expectTmsEqual(roundtrip, tm);
    });

    it("can encode/decode itself", () => {
      const simulated = myUtmSpec.encode(
        makeInitSnapshot(flipBitsSpec, makeArrayTapeOverlay(["0"])),
      );
      const simulator = myUtmSpec.encode(simulated);
      const decoded = simulator.decode();
      expectTmsEqual(must(decoded), simulated);
    });
  });

  describe("rules", { timeout: 20_000 }, () => {
    it.each(variousSnapshots)(
      "decodes to (original snapshot / undefined) for a while, then stepped snapshot",
      (tm) => {
        const utm = myUtmSpec.encode(tm);
        runUntilInnerStep(utm);
        expectTmsEqual(must(utm.decode()), step(tm));
      },
    );

    it("terminates with the same status as the simulated machine", () => {
      expect(
        getStatus(
          run(
            makeInitSnapshot(acceptImmediatelySpec, makeArrayTapeOverlay([])),
          ),
        ),
      ).toBe("accept");
      expect(
        getStatus(
          run(
            makeInitSnapshot(rejectImmediatelySpec, makeArrayTapeOverlay([])),
          ),
        ),
      ).toBe("reject");
    });

    it("terminates with the correct decoded tape", () => {
      const tm = makeInitSnapshot(
        flipBitsSpec,
        makeArrayTapeOverlay(["0", "1", "0", "1", "1"]),
      );
      const utm = run(myUtmSpec.encode(tm));

      run(tm);
      run(utm);

      const decoded = utm.decode();
      expectTmsEqual(must(decoded), tm);
    });
  });

  describe("recursion", () => {
    it("can simulate itself", { timeout: 600_000 }, () => {
      const simulator = myUtmSpec.encode(
        myUtmSpec.encode(
          myUtmSpec.encode(
            makeInitSnapshot(flipBitsSpec, makeArrayTapeOverlay(["0"])),
          ),
        ),
      );
      const doubleSimulator = myUtmSpec.encode(simulator);

      let decoded;
      for (let i = 0; i < 1e9; i++) {
        expect(getStatus(step(doubleSimulator))).toBe("running");
        decoded = doubleSimulator.decode();
        if (decoded && decoded.pos !== simulator.pos) {
          break;
        }
      }
      const target = step(simulator);
      expectTmsEqual(must(decoded), target);
    });
  });
});
