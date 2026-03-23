import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { isDeepStrictEqual } from "node:util";
import {
  MyUtmSnapshot,
  type MyUtmState,
  type MyUtmSymbol,
} from "../src/my-utm-spec";
import myUtmOptimizationHints from "../src/my-utm-spec-transition-optimization-hints";
import { flipBitsSpec } from "../src/toy-machines";
import {
  type StateIdx,
  type SymbolIdx,
  type TapeIdx,
  type TapeOverlay,
  copySnapshot,
  getStatus,
  makeInitSnapshot,
  step,
} from "../src/types";
import { makeArrayTapeOverlay, runUntilInnerStep } from "../src/util";

/** Materialize a TapeOverlay into a concrete array (reads until get() returns undefined). */
function materializeTape<S extends string>(tape: TapeOverlay<S>): S[] {
  const result: S[] = [];
  for (let i = 0; ; i++) {
    const v = tape.get(i);
    if (v === undefined) break;
    result.push(v);
  }
  return result;
}

const SAVEPOINT_FILE = "verify-nesting.savepoint.json";

function padN(n: number, width: number) {
  return n.toString().padStart(width, " ");
}

type Savepoint = {
  innerSteps: number;
  sim: {
    state: MyUtmState;
    tape: MyUtmSymbol[];
    pos: TapeIdx;
  };
};

let sim: MyUtmSnapshot<
  (typeof flipBitsSpec)["allStates"][StateIdx],
  (typeof flipBitsSpec)["allSymbols"][SymbolIdx]
>;
let real: MyUtmSnapshot<MyUtmState, MyUtmSymbol>;
let innerSteps: number;

const loadArg = process.argv.includes("--load");
if (loadArg && existsSync(SAVEPOINT_FILE)) {
  const data: Savepoint = JSON.parse(readFileSync(SAVEPOINT_FILE, "utf-8"));
  sim = new MyUtmSnapshot({
    ...data.sim,
    tape: makeArrayTapeOverlay(data.sim.tape),
    simSpec: flipBitsSpec,
  });
  real = MyUtmSnapshot.fromSimSnapshot(sim);
  innerSteps = data.innerSteps;
  console.log(`Loaded savepoint: innerSteps=${innerSteps}`);
} else {
  sim = MyUtmSnapshot.fromSimSnapshot(
    makeInitSnapshot(flipBitsSpec, makeArrayTapeOverlay(["0"])),
  );
  real = MyUtmSnapshot.fromSimSnapshot(sim, {
    optimizationHints: myUtmOptimizationHints,
  });
  innerSteps = 0;
}

const expectedInnerSteps = (() => {
  const utm = copySnapshot(sim);
  let steps = 0;
  while (getStatus(step(utm)) === "running") {
    steps++;
  }
  return steps;
})();

let lastInnerTickT = Date.now();
while (true) {
  const innerStepResult = runUntilInnerStep(real);
  if (innerStepResult.type === "halt") {
    break;
  }
  const { decoded } = innerStepResult;
  innerSteps++;
  step(sim);
  const simWindow = [-3, -2, -1, 0, 1, 2, 3].map(
    (i) => sim.tape.get(sim.pos + i) ?? sim.spec.blank,
  );
  const decodedWindow = [-3, -2, -1, 0, 1, 2, 3].map(
    (i) => decoded.tape.get(decoded.pos + i) ?? decoded.spec.blank,
  );
  if (
    !isDeepStrictEqual(
      [sim.state, sim.pos, simWindow],
      [decoded.state, decoded.pos, decodedWindow],
    )
  ) {
    console.error("sim and decoded are not equal!");
    console.error(`  sim.state = ${sim.state}`);
    console.error(`  decoded.state   = ${decoded.state}`);
    console.error(`  sim.pos   = ${sim.pos}`);
    console.error(`  decoded.pos     = ${decoded.pos}`);
    console.error(
      `  sim.tape.window = ${[-3, -2, -1, 0, 1, 2, 3].map((i) => sim.tape.get(sim.pos + i)).join("")}`,
    );
    console.error(
      `  decoded.tape.window   = ${[-3, -2, -1, 0, 1, 2, 3].map((i) => decoded.tape.get(decoded.pos + i)).join("")}`,
    );
    throw new Error("sim and decoded are not equal");
  }
  const now = Date.now();
  console.log(
    `ticked the simulated machine! (dur=${padN(Math.round(now - lastInnerTickT), 4)}ms)`,
  );
  console.log(
    `${padN(innerSteps, 6)}/${padN(expectedInnerSteps, 6)} : ${materializeTape(decoded.tape).join("")} => ${materializeTape(new MyUtmSnapshot({ ...decoded, simSpec: flipBitsSpec }).tape).join("")}`,
  );
  console.log(
    Array(6 + 1 + 6 + 3 + decoded.pos)
      .fill(" ")
      .join("") + `^ (state=${decoded.state})`,
  );
  lastInnerTickT = now;
  if (innerSteps > expectedInnerSteps) {
    throw new Error("should have halted by now");
  }
  const savepoint: Savepoint = {
    innerSteps,
    sim: {
      state: sim.state,
      tape: materializeTape(sim.tape),
      pos: sim.pos,
    },
  };
  writeFileSync(SAVEPOINT_FILE, JSON.stringify(savepoint));
}

console.log(`done`);
