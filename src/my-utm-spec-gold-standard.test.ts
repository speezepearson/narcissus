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
import { myUtmSpec } from "./my-utm-spec";
import {
  acceptImmediatelySpec,
  checkPalindromeSpec,
  flipBitsSpec,
  rejectImmediatelySpec,
} from "./toy-machines";
import { isDeepStrictEqual } from "node:util";

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
    makeInitSnapshot(flipBitsSpec, ["0", "1", "0", "1", "1"]),
  ),
  ...listAllSnapshots(
    makeInitSnapshot(checkPalindromeSpec, ["a", "b", "b", "a"]),
  ),
  ...listAllSnapshots(
    makeInitSnapshot(checkPalindromeSpec, ["a", "b", "c", "b", "a"]),
  ),
  ...listAllSnapshots(
    makeInitSnapshot(checkPalindromeSpec, ["a", "b", "c", "d", "b", "a"]),
  ),
];

describe("myUtmSpec gold standard tests", () => {
  describe("decode", () => {
    it.each(variousSnapshots)("inverts encode", (tm) => {
      const encoded = myUtmSpec.encode(tm);
      const decoded = myUtmSpec.decode(tm.spec, encoded);
      expect(decoded).toEqual(tm);
    });
  });

  describe("rules", () => {
    it.each(variousSnapshots)(
      "decodes to original snapshot, then undefined, then stepped snapshot",
      (tm) => {
        const utm = makeInitSnapshot(myUtmSpec, myUtmSpec.encode(tm));

        const snap0 = myUtmSpec.decode(tm.spec, utm.tape);
        expect(snap0).toEqual(tm);

        while (isDeepStrictEqual(snap0, myUtmSpec.decode(tm.spec, utm.tape))) {
          expect(step(utm)).toBe("continue");
        }
        while (myUtmSpec.decode(tm.spec, utm.tape) === undefined) {
          expect(step(utm)).toBe("continue");
        }

        const snap1 = myUtmSpec.decode(tm.spec, utm.tape);
        expect(snap1).not.toEqual(snap0);
        expect(snap1).toEqual(step(tm));
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
      const utm = run(makeInitSnapshot(myUtmSpec, myUtmSpec.encode(tm)));

      run(tm);
      run(utm);

      expect(myUtmSpec.decode(tm.spec, utm.tape)).toEqual(tm);
    });
  });

  describe("recursion", () => {
    it("can simulate itself", () => {
      const baseTm = makeInitSnapshot(checkPalindromeSpec, ["a", "b", "a"]);
      const utm = makeInitSnapshot(
        myUtmSpec,
        myUtmSpec.encode(makeInitSnapshot(myUtmSpec, myUtmSpec.encode(baseTm))),
      );

      run(baseTm);
      run(utm);

      expect(getStatus(utm)).toBe(getStatus(baseTm));
      const dec1 = myUtmSpec.decode(myUtmSpec, utm.tape);
      expect(dec1).not.toBeUndefined();
      const dec2 = myUtmSpec.decode(baseTm.spec, dec1!.tape);
      expect(dec2).toEqual(baseTm);
    });
  });
});
