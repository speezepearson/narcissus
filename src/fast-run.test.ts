import { describe, expect, it } from "vitest";
import {
  compile,
  compileSnapshot,
  fastRun,
  fastStep,
  decompileSnapshot,
} from "./fast-run";
import {
  flipBitsSpec,
  checkPalindromeSpec,
  doubleXSpec,
  acceptImmediatelySpec,
  rejectImmediatelySpec,
} from "./toy-machines";
import { getStatus, makeInitSnapshot, run, step } from "./types";
import { makeArrayTapeOverlay } from "./util";
import type { TuringMachineSnapshot } from "./types";

/** Compare two snapshots' observable state (state, pos, tape contents). */
function expectSnapshotsMatch<State extends string, Symbol extends string>(
  actual: TuringMachineSnapshot<State, Symbol>,
  expected: TuringMachineSnapshot<State, Symbol>,
): void {
  expect(actual.state).toBe(expected.state);
  expect(actual.pos).toBe(expected.pos);
  const blank = expected.spec.blank;
  let i = 0;
  while (true) {
    const a = actual.tape.get(i);
    const b = expected.tape.get(i);
    if (a === undefined && b === undefined) break;
    expect(a ?? blank).toBe(b ?? blank);
    i++;
  }
}

describe("fast-run", () => {
  describe("compile", () => {
    it("creates a compiled machine with correct dimensions", () => {
      const compiled = compile(flipBitsSpec);
      expect(compiled.numStates).toBe(1);
      expect(compiled.numSymbols).toBe(3);
    });

    it("marks accepting states", () => {
      const compiled = compile(flipBitsSpec);
      // "init" is state 0, and it's accepting
      expect(compiled.accepting[0]).toBe(1);
    });

    it("marks non-accepting states", () => {
      const compiled = compile(checkPalindromeSpec);
      // "start" is state 0, not accepting
      expect(compiled.accepting[0]).toBe(0);
      // "accept" is state 1, accepting
      expect(compiled.accepting[1]).toBe(1);
    });
  });

  describe("fastRun matches regular run", () => {
    async function compareFastAndRegular<
      State extends string,
      Symbol extends string,
    >(
      spec: import("./types").TuringMachineSpec<State, Symbol>,
      tapeContents: Symbol[],
    ) {
      // Run with regular engine
      const snap1 = makeInitSnapshot(
        spec,
        makeArrayTapeOverlay([...tapeContents]),
      );
      await run(snap1);
      const expectedStatus = getStatus(snap1);

      // Run with fast engine
      const snap2 = makeInitSnapshot(
        spec,
        makeArrayTapeOverlay([...tapeContents]),
      );
      const compiled = compile(spec);
      const cSnap = compileSnapshot(snap2, compiled);
      const fastResult = fastRun(cSnap);
      const result = decompileSnapshot(cSnap, spec);

      expect(fastResult.halted).toBe(true);
      if (!fastResult.halted) throw new Error("unreachable");
      expect(fastResult.status).toBe(expectedStatus);
      expectSnapshotsMatch(result, snap1);
    }

    it("flipBitsSpec", async () => {
      await compareFastAndRegular(flipBitsSpec, ["0", "1", "0", "1", "1"]);
    });

    it("flipBitsSpec empty tape", async () => {
      await compareFastAndRegular(flipBitsSpec, []);
    });

    it("acceptImmediatelySpec", async () => {
      await compareFastAndRegular(acceptImmediatelySpec, []);
    });

    it("rejectImmediatelySpec", async () => {
      await compareFastAndRegular(rejectImmediatelySpec, []);
    });

    it("checkPalindromeSpec - aa (accept)", async () => {
      await compareFastAndRegular(checkPalindromeSpec, ["a", "a"]);
    });

    it("checkPalindromeSpec - ab (reject)", async () => {
      await compareFastAndRegular(checkPalindromeSpec, ["a", "b"]);
    });

    it("checkPalindromeSpec - aba (accept)", async () => {
      await compareFastAndRegular(checkPalindromeSpec, ["a", "b", "a"]);
    });

    it("doubleXSpec", async () => {
      await compareFastAndRegular(doubleXSpec, ["$", "X", "X"]);
    });

    it("doubleXSpec single X", async () => {
      await compareFastAndRegular(doubleXSpec, ["$", "X"]);
    });
  });

  describe("step-by-step equivalence", () => {
    it("each fast step matches regular step", () => {
      const spec = flipBitsSpec;
      const tape: ("0" | "1" | "_")[] = ["0", "1", "0"];

      const regular = makeInitSnapshot(spec, makeArrayTapeOverlay([...tape]));
      const compiled = compile(spec);
      const fast = compileSnapshot(
        makeInitSnapshot(spec, makeArrayTapeOverlay([...tape])),
        compiled,
      );

      while (getStatus(regular) === "running") {
        step(regular);
        fastStep(fast);
        const decompiled = decompileSnapshot(fast, spec);
        expectSnapshotsMatch(decompiled, regular);
      }
    });
  });

  describe("gas limit", () => {
    it("returns halted: false when gas is exceeded", () => {
      const compiled = compile(checkPalindromeSpec);
      const snap = compileSnapshot(
        makeInitSnapshot(
          checkPalindromeSpec,
          makeArrayTapeOverlay(["a", "b", "a"]),
        ),
        compiled,
      );
      const result = fastRun(snap, { gas: 1 });
      expect(result.halted).toBe(false);
      expect(result.steps).toBe(1);
    });
  });
});
