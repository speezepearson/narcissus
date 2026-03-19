import { describe, expect, it } from "vitest";
import { getStatus, makeInitSnapshot, run, step } from "./types";
import {
  acceptImmediatelySpec,
  checkPalindromeSpec,
  flipBitsSpec,
  rejectImmediatelySpec,
} from "./toy-machines";

describe("acceptImmediatelySpec", () => {
  it("terminates with accept", () => {
    expect(getStatus(step(makeInitSnapshot(acceptImmediatelySpec, [])))).toBe(
      "accept",
    );
  });
});

describe("rejectImmediatelySpec", () => {
  it("terminates with reject", () => {
    expect(getStatus(step(makeInitSnapshot(rejectImmediatelySpec, [])))).toBe(
      "reject",
    );
  });
});

describe("flipBitsSpec", () => {
  it("flips bits", () => {
    const tm = run(makeInitSnapshot(flipBitsSpec, ["0", "1", "0", "1", "1"]));
    expect(getStatus(tm)).toBe("accept");
    expect(tm.tape).toEqual(["1", "0", "1", "0", "0", "_"]);
  });
});

describe("checkPalindromeSpec", () => {
  it("accepts an even-length palindrome", () => {
    const tm = run(makeInitSnapshot(checkPalindromeSpec, ["a", "b", "b", "a"]));
    expect(getStatus(tm)).toBe("accept");
  });
  it("accepts an odd-length palindrome", () => {
    const tm = run(
      makeInitSnapshot(checkPalindromeSpec, ["a", "b", "b", "b", "a"]),
    );
    expect(getStatus(tm)).toBe("accept");
  });
  it("rejects a non-palindrome", () => {
    const tm = run(
      makeInitSnapshot(checkPalindromeSpec, ["a", "b", "b", "a", "b"]),
    );
    expect(getStatus(tm)).toBe("reject");
  });
});
