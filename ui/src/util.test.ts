import { describe, expect, it } from "vitest";
import { tapesEqual } from "./util";
import fc from "fast-check";

describe("tapesEqual", () => {
  it("should return true iff the tapes are equal modulo trailing blanks", () =>
    fc.assert(
      fc.property(
        fc.array(fc.oneof(fc.constant("a"), fc.constant("b")), {
          maxLength: 5,
        }),
        fc.array(fc.oneof(fc.constant("a"), fc.constant("b")), {
          maxLength: 5,
        }),
        fc.integer({ min: 0, max: 3 }),
        fc.integer({ min: 0, max: 3 }),
        (aPrefix, bPrefix, nABlanks, nBBlanks) => {
          const samePrefix =
            aPrefix.length === bPrefix.length &&
            aPrefix.every((c, i) => c === bPrefix[i]);
          const a = [...aPrefix, ...Array(nABlanks).fill("_")];
          const b = [...bPrefix, ...Array(nBBlanks).fill("_")];
          expect(tapesEqual(a, b, "_"), `a=${a}, b=${b}`).toBe(samePrefix);
        },
      ),
    ));
});
