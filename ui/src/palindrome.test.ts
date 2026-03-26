import { describe, expect, it } from "vitest";
import { getStatus, makeInitSnapshot, run, Symbol } from "./types";
import { machineSpecs } from "./parseSpec";

const palindromeSpec = machineSpecs.find((s) => s.name === "Check Palindrome")!;
if (!palindromeSpec) throw new Error("Check Palindrome spec not found");

function makeSnapshot(input: string) {
  const tape = [...input].map((c) => Symbol.parse(c));
  return makeInitSnapshot(palindromeSpec.spec, tape);
}

describe("Check Palindrome", () => {
  it("accepts abcba", async () => {
    const snapshot = makeSnapshot("abcba");
    await run(snapshot);
    expect(getStatus(snapshot)).toBe("accept");
  });

  it("rejects abcca", async () => {
    const snapshot = makeSnapshot("abcca");
    await run(snapshot);
    expect(getStatus(snapshot)).toBe("reject");
  });
});
