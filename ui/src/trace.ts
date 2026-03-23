import { myUtmSpec } from "./my-utm-spec";
import { flipBitsSpec } from "./toy-machines";
import {
  getStatus,
  makeInitSnapshot,
  step,
  type TuringMachineSnapshot,
} from "./types";
import { makeArrayTapeOverlay } from "./util";

const simSnap = step(
  makeInitSnapshot(flipBitsSpec, makeArrayTapeOverlay(["0", "1"])),
);
const utm = myUtmSpec.encode(simSnap);

// Read the full UTM tape as a string
function readTape(
  snap: TuringMachineSnapshot<string, string>,
  maxLen = 200,
): string {
  const chars: string[] = [];
  for (let i = 0; i < maxLen; i++) {
    const c = snap.tape.get(i);
    if (c === undefined) break;
    chars.push(c);
  }
  return chars.join("");
}

console.log("Simulated machine:");
console.log(`  state=${simSnap.state} pos=${simSnap.pos}`);
console.log(`  tape: ${readTape(simSnap)}`);
console.log();

console.log("Initial UTM tape:");
console.log(`  ${readTape(utm)}`);
console.log(`  UTM state=${utm.state} pos=${utm.pos}`);

const decoded0 = utm.decode();
console.log(
  `  decode() => ${decoded0 ? `state=${decoded0.state} pos=${decoded0.pos}` : "undefined"}`,
);
console.log();

const MAX_STEPS = 5000;
for (let i = 1; i <= MAX_STEPS; i++) {
  const prevState = utm.state;
  step(utm);
  const st = getStatus(utm);

  // Try to decode
  const decoded = utm.decode();

  // Print every step where something interesting happens:
  // state changed, decoded returned a value, or machine halted
  const stateChanged = utm.state !== prevState;
  if (stateChanged || decoded || st !== "running") {
    const tapeStr = readTape(utm);
    // Highlight the head position
    const head = utm.pos;
    const before = tapeStr.slice(0, head);
    const at = tapeStr[head] ?? "_";
    const after = tapeStr.slice(head + 1);

    console.log(`step ${i}: UTM state=${utm.state} pos=${head}  status=${st}`);
    console.log(`  tape: ${before}[${at}]${after}`);
    if (decoded) {
      console.log(`  DECODED: state=${decoded.state} pos=${decoded.pos}`);
    }
    if (st !== "running") {
      console.log(`\nHalted after ${i} steps: ${st}`);
      break;
    }
  }

  if (i === MAX_STEPS) {
    console.log(`\nStopped after ${MAX_STEPS} steps (still running)`);
  }
}
