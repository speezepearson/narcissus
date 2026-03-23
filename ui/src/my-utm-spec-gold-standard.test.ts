/**
 * DO NOT MODIFY THIS FILE.
 */

import { describe, expect, it } from "vitest";
import { myUtmSpec } from "./my-utm-spec";
import { must } from "./test-util";
import {
  acceptImmediatelySpec,
  checkPalindromeSpec,
  doubleXSpec,
  flipBitsSpec,
  rejectImmediatelySpec,
} from "./toy-machines";
import {
  copySnapshot,
  getStatus,
  makeInitSnapshot,
  run,
  step,
  type TuringMachineSnapshot,
} from "./types";
import { makeArrayTapeOverlay, runUntilInnerStep } from "./util";

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
  {
    name: "acceptImmediatelySpec",
    tm: makeInitSnapshot(acceptImmediatelySpec, makeArrayTapeOverlay([])),
  },
  {
    name: "rejectImmediatelySpec",
    tm: makeInitSnapshot(rejectImmediatelySpec, makeArrayTapeOverlay([])),
  },
  ...listAllSnapshots(
    makeInitSnapshot(flipBitsSpec, makeArrayTapeOverlay(["0", "1"])),
  ).map((tm, i) => ({ name: `flipBitsSpec-${i}`, tm })),
  ...listAllSnapshots(
    makeInitSnapshot(doubleXSpec, makeArrayTapeOverlay(["$", "X", "X"])),
  ).map((tm, i) => ({ name: `doubleXSpec-${i}`, tm })),
  ...listAllSnapshots(
    makeInitSnapshot(checkPalindromeSpec, makeArrayTapeOverlay(["a", "a"])),
  ).map((tm, i) => ({ name: `checkPalindromeSpec-a-a-${i}`, tm })),
  ...listAllSnapshots(
    makeInitSnapshot(checkPalindromeSpec, makeArrayTapeOverlay(["b", "b"])),
  ).map((tm, i) => ({ name: `checkPalindromeSpec-b-b-${i}`, tm })),
  ...listAllSnapshots(
    makeInitSnapshot(checkPalindromeSpec, makeArrayTapeOverlay(["a", "b"])),
  ).map((tm, i) => ({ name: `checkPalindromeSpec-a-b-${i}`, tm })),
];

describe("myUtmSpec gold standard tests", () => {
  describe("decode", () => {
    it.each(variousSnapshots)("inverts encode - $name", ({ tm }) => {
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

  describe.concurrent("rules", { timeout: 20_000 }, () => {
    it.concurrent.each(variousSnapshots)(
      "decodes to (original snapshot / undefined) for a while, then stepped snapshot - $name",
      ({ tm }) => {
        const utm = myUtmSpec.encode(tm);
        if (runUntilInnerStep(utm).type === "halt") return;
        expectTmsEqual(must(utm.decode()), step(tm));
      },
    );

    it.concurrent(
      "terminates with the same status as the simulated machine",
      async () => {
        expect(
          getStatus(
            await run(
              makeInitSnapshot(acceptImmediatelySpec, makeArrayTapeOverlay([])),
            ),
          ),
        ).toBe("accept");
        expect(
          getStatus(
            await run(
              makeInitSnapshot(rejectImmediatelySpec, makeArrayTapeOverlay([])),
            ),
          ),
        ).toBe("reject");
      },
    );

    it.concurrent("terminates with the correct decoded tape", async () => {
      const tm = makeInitSnapshot(
        flipBitsSpec,
        makeArrayTapeOverlay(["0", "1", "0", "1", "1"]),
      );
      const utm = await run(myUtmSpec.encode(tm));

      await run(tm);
      await run(utm);

      const decoded = utm.decode();
      expectTmsEqual(must(decoded), tm);
    });
  });
});
