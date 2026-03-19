/**
 * DO NOT MODIFY THIS FILE.
 */

import { describe, expect, it } from "vitest";
import {
  getStatus,
  makeInitSnapshot,
  run,
  step,
  type TuringMachineSnapshot,
} from "./types";
import { makeInitUtmSnapshot, myUtmSpec } from "./my-utm-spec";
import {
  acceptImmediatelySpec,
  checkPalindromeSpec,
  flipBitsSpec,
  rejectImmediatelySpec,
} from "./toy-machines";
import { tmsEqual } from "./util";
import { expectTmsEqual, must } from "./test-util";

function listAllSnapshots<TM extends TuringMachineSnapshot<string, string>>(
  tm: TM,
): Array<TM> {
  const res = [];
  while (getStatus(tm) === "running") {
    res.push(structuredClone(tm));
    step(tm);
  }
  res.push(tm);
  return res;
}

const variousSnapshots = [
  makeInitSnapshot(acceptImmediatelySpec, []),
  makeInitSnapshot(rejectImmediatelySpec, []),
  ...listAllSnapshots(
    makeInitSnapshot(flipBitsSpec, ["0", "1"]),
  ),
  ...listAllSnapshots(
    makeInitSnapshot(checkPalindromeSpec, ["a", "a"]),
  ),
  ...listAllSnapshots(
    makeInitSnapshot(checkPalindromeSpec, ["b", "b"]),
  ),
  ...listAllSnapshots(
    makeInitSnapshot(checkPalindromeSpec, ["a", "b"]),
  ),
];

describe("myUtmSpec gold standard tests", () => {
  describe("decode", () => {
    it.each(variousSnapshots)("inverts encode", (tm) => {
      const encoded = myUtmSpec.encode(tm);
      const decoded = myUtmSpec.decode(tm.spec, makeInitSnapshot(myUtmSpec, encoded));
      expectTmsEqual(must(decoded), tm);
    });

    it('can encode/decode itself', () => {
      const simulated = makeInitUtmSnapshot(makeInitSnapshot(flipBitsSpec, ["0"]));
      const simulator = makeInitUtmSnapshot(simulated);
      const decoded = myUtmSpec.decode(myUtmSpec, simulator);
      expectTmsEqual(must(decoded), simulated);
    })
  });

  describe("rules", () => {
    it.each(variousSnapshots)(
      "decodes to (original snapshot / undefined) for a while, then stepped snapshot",
      (tm) => {
        const utm = makeInitUtmSnapshot(tm);

        let snap = myUtmSpec.decode(tm.spec, utm);

        while (snap === undefined || tmsEqual(snap, tm)) {
          if (getStatus(utm) !== 'running') break;
          step(utm);
          snap = myUtmSpec.decode(tm.spec, utm);
        }

        const snap1 = myUtmSpec.decode(tm.spec, utm);
        expectTmsEqual(must(snap1), step(tm));
      },
    );

    it("terminates with the same status as the simulated machine", () => {
      expect(getStatus(run(makeInitSnapshot(acceptImmediatelySpec, [])))).toBe(
        "accept",
      );
      expect(getStatus(run(makeInitSnapshot(rejectImmediatelySpec, [])))).toBe(
        "reject",
      );
    });

    it("terminates with the correct decoded tape", () => {
      const tm = makeInitSnapshot(flipBitsSpec, ["0", "1", "0", "1", "1"]);
      const utm = run(makeInitUtmSnapshot(tm));

      run(tm);
      run(utm);

      const decoded = myUtmSpec.decode(tm.spec, utm);
      expectTmsEqual(must(decoded), tm);
    });
  });

  describe("recursion", () => {
    it("can simulate itself", () => {
      const simulator = makeInitUtmSnapshot(makeInitSnapshot(flipBitsSpec, ["0"]));
      const doubleSimulator = makeInitUtmSnapshot(simulator);

      let decoded;
      for (let i=0; i<1e7; i++) {
        expect(getStatus(step(doubleSimulator))).toBe('running');
        decoded = myUtmSpec.decode(myUtmSpec, doubleSimulator);
        if (decoded && (decoded.pos !== simulator.pos)) {
          break;
        }
      }
      const target = step(simulator);
      expectTmsEqual(must(decoded), target);
    });
  });
});
