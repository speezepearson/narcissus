import { describe, expect, it } from "vitest";
import { tapesEqual } from "./util";
import fc from 'fast-check';
import { isDeepStrictEqual } from "node:util";

describe('tapesEqual', () => {
  it('should return true iff the tapes are equal modulo trailing blanks', () => fc.assert(fc.property(
    fc.array(fc.oneof(fc.constant('a'), fc.constant('b')), {maxLength: 5}),
    fc.array(fc.oneof(fc.constant('a'), fc.constant('b')), {maxLength: 5}),
    fc.integer({min: 0, max: 3}),
    fc.integer({min: 0, max: 3}),
    (aPrefix, bPrefix, nABlanks, nBBlanks) => {
      const samePrefix = isDeepStrictEqual(aPrefix, bPrefix);
      const a = [...aPrefix, ...Array(nABlanks).fill('_')];
      const b = [...bPrefix, ...Array(nBBlanks).fill('_')];
      expect(tapesEqual(a, b, '_'), `a=${a}, b=${b}`).toBe(samePrefix);
  })));
});