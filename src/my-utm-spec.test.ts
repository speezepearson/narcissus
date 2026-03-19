import { describe, it } from "vitest";
import { myUtmSpec } from "./my-utm-spec";
import {
  makeInitSnapshot,
} from "./types";
import {
  checkPalindromeSpec,
} from "./toy-machines";

describe("palindrome debug", () => {
  it("check encoded tape size", () => {
    const tm = makeInitSnapshot(checkPalindromeSpec, ["a", "b", "b", "a"]);
    const encoded = myUtmSpec.encode(tm);
    console.log(`Encoded tape length: ${encoded.tape.length}`);

    // Find # positions
    const hashes: number[] = [];
    for (let i = 0; i < encoded.tape.length; i++) {
      if (encoded.tape[i] === "#") hashes.push(i);
    }
    console.log(`Section boundaries (#): ${hashes.join(", ")}`);
    console.log(`RULES section: ${hashes[0]+1} to ${hashes[1]-1} (${hashes[1] - hashes[0] - 1} chars)`);
    console.log(`ACC section: ${hashes[1]+1} to ${hashes[2]-1}`);
    console.log(`STATE section: ${hashes[2]+1} to ${hashes[3]-1}`);

    // Count rules (number of . or * prefixes)
    let ruleCount = 0;
    for (const c of encoded.tape) {
      if (c === ".") ruleCount++;
    }
    console.log(`Number of rules: ${ruleCount}`);
  });
});
