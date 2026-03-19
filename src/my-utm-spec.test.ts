import { describe, it } from "vitest";
import { myUtmSpec } from "./my-utm-spec";
import {
  makeInitSnapshot,
  step,
  getStatus,
} from "./types";
import {
  checkPalindromeSpec,
} from "./toy-machines";
import { isDeepStrictEqual } from "node:util";
import { expectTmsEqual, must } from "./test-util";
import { tmsEqual } from "./util";

describe("palindrome debug", () => {
  it("check encoded tape size", () => {
    const tm = makeInitSnapshot(checkPalindromeSpec, ["a", "b", "b", "a"]);
    const encoded = myUtmSpec.encode(tm);
    console.log(`Encoded tape length: ${encoded.length}`);

    // Find # positions
    const hashes: number[] = [];
    for (let i = 0; i < encoded.length; i++) {
      if (encoded[i] === "#") hashes.push(i);
    }
    console.log(`Section boundaries (#): ${hashes.join(", ")}`);
    console.log(`RULES section: ${hashes[0]+1} to ${hashes[1]-1} (${hashes[1] - hashes[0] - 1} chars)`);
    console.log(`ACC section: ${hashes[1]+1} to ${hashes[2]-1}`);
    console.log(`STATE section: ${hashes[2]+1} to ${hashes[3]-1}`);

    // Count rules (number of . or * prefixes)
    let ruleCount = 0;
    for (const c of encoded) {
      if (c === ".") ruleCount++;
    }
    console.log(`Number of rules: ${ruleCount}`);
  });

  it("first step with high limit", () => {
    const tm = makeInitSnapshot(checkPalindromeSpec, ["a", "b", "b", "a"]);
    const utm = makeInitSnapshot(myUtmSpec, myUtmSpec.encode(tm));

    const snap0 = must(myUtmSpec.decode(tm.spec, utm));
    expectTmsEqual(snap0, tm);

    let steps = 0;
    const maxSteps = 5_000_000;

    let decoded = myUtmSpec.decode(tm.spec, utm);
    while (
      (decoded === undefined || tmsEqual(snap0, decoded)) &&
      getStatus(utm) === "running"
    ) {
      step(utm);
      decoded = myUtmSpec.decode(tm.spec, utm);
      steps++;
      if (steps > maxSteps) {
        console.log(`TIMEOUT phase 1 after ${maxSteps} steps, UTM state=${utm.state}`);
        break;
      }
    }

    if (steps <= maxSteps) {
      console.log(`Phase 1 done in ${steps} steps, UTM state=${utm.state}`);
    }

    if (steps <= maxSteps) {
      const snap1 = myUtmSpec.decode(tm.spec, utm);
      const stepped = step(tm);
      console.log(`Phase 2 done in ${steps} total steps`);
      console.log(`UTM status: ${getStatus(utm)}`);
      console.log(`snap1: ${JSON.stringify({state: snap1?.state, pos: snap1?.pos})}`);
      console.log(`step(tm): ${JSON.stringify({state: stepped.state, pos: stepped.pos})}`);
      expectTmsEqual(must(snap1), stepped);
    }
  }, 30000);
});
