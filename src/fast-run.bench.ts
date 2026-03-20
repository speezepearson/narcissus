import { bench, describe } from "vitest";
import { makeInitSnapshot, step, getStatus } from "./types";
import { acceptImmediatelySpec } from "./toy-machines";
import { makeArrayTapeOverlay } from "./util";
import { myUtmSpec } from "./my-utm-spec";
import { compile, compileSnapshot, fastStep, fastRun } from "./fast-run";

const snapshot = myUtmSpec.encode(
  myUtmSpec.encode(
    makeInitSnapshot(acceptImmediatelySpec, makeArrayTapeOverlay([])),
  ),
);

const STEPS = 1_000_000;

describe(`${STEPS.toLocaleString()} steps on UTM-of-UTM(acceptImmediately)`, () => {
  bench("step (original)", () => {
    const snap = {
      spec: snapshot.spec,
      state: snapshot.state,
      tape: snapshot.tape.clone(),
      pos: snapshot.pos,
    };
    for (let i = 0; i < STEPS; i++) {
      if (getStatus(snap) !== "running") break;
      step(snap);
    }
  });

  bench("fastStep (one at a time)", () => {
    const machine = compile(snapshot.spec);
    const compiled = compileSnapshot(snap(), machine);
    for (let i = 0; i < STEPS; i++) {
      if (!fastStep(compiled)) break;
    }
  });

  bench("fastRun (batch)", () => {
    const machine = compile(snapshot.spec);
    const compiled = compileSnapshot(snap(), machine);
    fastRun(compiled, { gas: STEPS });
  });
});

// Helper to avoid paying clone cost in the benchmark loop setup for fast variants
function snap() {
  return {
    spec: snapshot.spec,
    state: snapshot.state,
    tape: snapshot.tape.clone(),
    pos: snapshot.pos,
  };
}
