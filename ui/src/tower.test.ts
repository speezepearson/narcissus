import { describe, expect, it } from "vitest";
import {
  buildTower,
  updateTower,
  decodeUtmTape,
  type TowerLevel,
  type UtmMeta,
} from "./tower";

// ── Helpers ──

// 2 states (A, B), 2 symbols (0, 1) → 1 state bit, 1 symbol bit
const META: UtmMeta = { utmStates: ["A", "B"], utmSymbolChars: "01" };

// Rule: in state 0 reading 0, write 0 stay in state 0 move R
const RULE = ".0|0|0|0|R";

/** Build a valid L0 tape string with the given encoded state and tape cells. */
function makeL0Tape(opts: {
  stateBits: string;
  tapeCells: string;
  rules?: string;
  accepting?: string;
  blank?: string;
}): string {
  const rules = opts.rules ?? RULE;
  const accepting = opts.accepting ?? "0";
  const blank = opts.blank ?? "0";
  return `$#${rules}#${accepting}#${opts.stateBits}#${blank}#${opts.tapeCells}`;
}

function makeLevel(
  state: string,
  headPos: number,
  tape: string,
): TowerLevel {
  return { state, headPos, tape, tapeLen: tape.length };
}

// ── Tests ──

describe("decodeUtmTape sanity", () => {
  it("decodes a tape with state A", () => {
    const tape = makeL0Tape({ stateBits: "0", tapeCells: "^0,1,0" });
    const decoded = decodeUtmTape(tape, META.utmStates, META.utmSymbolChars);
    expect(decoded).toEqual({ state: "A", headPos: 0, tape: "010" });
  });

  it("decodes a tape with state B", () => {
    const tape = makeL0Tape({ stateBits: "1", tapeCells: "^0,1,0" });
    const decoded = decodeUtmTape(tape, META.utmStates, META.utmSymbolChars);
    expect(decoded).toEqual({ state: "B", headPos: 0, tape: "010" });
  });

  it("decodes head position from ^ marker", () => {
    const tape = makeL0Tape({ stateBits: "0", tapeCells: "0,^1,0" });
    const decoded = decodeUtmTape(tape, META.utmStates, META.utmSymbolChars);
    expect(decoded).toEqual({ state: "A", headPos: 1, tape: "010" });
  });
});

describe("buildTower (gold standard)", () => {
  it("builds a 2-level tower from a decodable L0 tape", () => {
    const l0Tape = makeL0Tape({ stateBits: "0", tapeCells: "^0,1,0" });
    const l0 = makeLevel("UTM_STATE_1", 5, l0Tape);
    const tower = buildTower(l0, META);

    expect(tower).toHaveLength(2);
    expect(tower[0]).toEqual(l0);
    expect(tower[1]).toEqual({
      state: "A",
      headPos: 0,
      tape: "010",
      tapeLen: 3,
    });
  });

  it("builds a 1-level tower when L0 tape is not decodable", () => {
    const l0 = makeLevel("Q0", 0, "not_a_utm_tape");
    const tower = buildTower(l0, META);
    expect(tower).toHaveLength(1);
    expect(tower[0]).toEqual(l0);
  });
});

describe("updateTower equivalence with buildTower", () => {
  it("matches buildTower when L0 state changes", () => {
    const tape1 = makeL0Tape({ stateBits: "0", tapeCells: "^0,1,0" });
    const l0v1 = makeLevel("S1", 5, tape1);
    const tower = buildTower(l0v1, META);

    const tape2 = makeL0Tape({ stateBits: "1", tapeCells: "^1,0,1" });
    const l0v2 = makeLevel("S2", 6, tape2);

    const expected = buildTower(l0v2, META);

    updateTower(l0v2, tower, META);
    expect(tower).toEqual(expected);
  });

  it("matches buildTower when L0 state changes but decoded L1 state is the same", () => {
    // Both tapes encode state "A" (stateBits "0"), but L0 state differs
    const tape1 = makeL0Tape({ stateBits: "0", tapeCells: "^0,1,0" });
    const l0v1 = makeLevel("S1", 5, tape1);
    const tower = buildTower(l0v1, META);

    const tape2 = makeL0Tape({ stateBits: "0", tapeCells: "^1,0,1" });
    const l0v2 = makeLevel("S2", 6, tape2);

    const expected = buildTower(l0v2, META);

    updateTower(l0v2, tower, META);
    expect(tower).toEqual(expected);
  });

  it("matches buildTower on first call (empty tower)", () => {
    const tape = makeL0Tape({ stateBits: "0", tapeCells: "^0,1,0" });
    const l0 = makeLevel("S1", 5, tape);

    const expected = buildTower(l0, META);

    const tower: TowerLevel[] = [];
    updateTower(l0, tower, META);
    expect(tower).toEqual(expected);
  });

  it("matches buildTower when tower shrinks (L0 tape becomes non-decodable)", () => {
    const tape1 = makeL0Tape({ stateBits: "0", tapeCells: "^0,1,0" });
    const l0v1 = makeLevel("S1", 5, tape1);
    const tower = buildTower(l0v1, META);
    expect(tower).toHaveLength(2);

    const l0v2 = makeLevel("S2", 0, "garbage");
    const expected = buildTower(l0v2, META);

    updateTower(l0v2, tower, META);
    expect(tower).toEqual(expected);
  });
});

describe("updateTower skips decoding when L0 state unchanged", () => {
  it("preserves L1+ when L0 state does not change", () => {
    const tape1 = makeL0Tape({ stateBits: "0", tapeCells: "^0,1,0" });
    const l0v1 = makeLevel("S1", 5, tape1);
    const tower = buildTower(l0v1, META);
    const origL1 = { ...tower[1] };

    // Same L0 state "S1", different headPos and tape content
    const tape2 = makeL0Tape({ stateBits: "1", tapeCells: "^1,1,1" });
    const l0v2 = makeLevel("S1", 7, tape2);

    updateTower(l0v2, tower, META);

    // L0 was updated
    expect(tower[0]).toEqual(l0v2);
    // L1 was NOT re-decoded (preserved from before)
    expect(tower[1]).toEqual(origL1);
  });
});

describe("updateTower detects L1 state change", () => {
  it("re-decodes and updates L1 state from A to B", () => {
    const tape1 = makeL0Tape({ stateBits: "0", tapeCells: "^0,1,0" });
    const l0v1 = makeLevel("S1", 5, tape1);
    const tower = buildTower(l0v1, META);
    expect(tower[1].state).toBe("A");

    // Change encoded state from "0" (A) to "1" (B)
    const tape2 = makeL0Tape({ stateBits: "1", tapeCells: "^0,1,0" });
    const l0v2 = makeLevel("S2", 6, tape2);

    updateTower(l0v2, tower, META);
    expect(tower[1].state).toBe("B");
  });

  it("re-decodes and updates L1 state from B to A", () => {
    const tape1 = makeL0Tape({ stateBits: "1", tapeCells: "^1,0" });
    const l0v1 = makeLevel("S1", 3, tape1);
    const tower = buildTower(l0v1, META);
    expect(tower[1].state).toBe("B");

    const tape2 = makeL0Tape({ stateBits: "0", tapeCells: "^1,0" });
    const l0v2 = makeLevel("S2", 4, tape2);

    updateTower(l0v2, tower, META);
    expect(tower[1].state).toBe("A");
  });
});
