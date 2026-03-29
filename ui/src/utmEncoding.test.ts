import { describe, expect, it } from "vitest";
import { getStatus, makeInitSnapshot, run, Symbol } from "./types";
import { machineSpecs, rustExport } from "./parseSpec";
import { decodeFromUtm } from "./utmEncoding";

function getSpec(name: string) {
  const spec = machineSpecs.find((s) => s.name === name);
  if (!spec) throw new Error(`${name} spec not found`);
  return spec;
}

const flipBits = getSpec("Flip Bits");
const utm = getSpec("Universal Turing Machine");

function syms(s: string): Symbol[] {
  return [...s].map((c) => Symbol.parse(c));
}

describe("UTM encoding roundtrip: flip bits on 10110", () => {
  it("direct execution produces correct result", async () => {
    const snapshot = makeInitSnapshot(flipBits.spec, syms("10110"));
    await run(snapshot);
    expect(getStatus(snapshot)).toBe("accept");
    // Flip bits: 1->0, 0->1, 1->0, 1->0, 0->1
    expect(snapshot.tape.slice(0, 5).join("")).toBe("01001");
  });
});

describe("welcomeModalExample tapes decode correctly", () => {
  it("double-decoding doubleUtmInput recovers bitFlipperInput", () => {
    const { welcomeModalExample } = rustExport;

    // L2 tape -> decode with utmSpec guest -> L1 UTM snapshot
    const l1 = decodeFromUtm(
      utm.spec,
      welcomeModalExample.doubleUtmInput,
    );

    // L1 tape -> decode with flipBitsSpec guest -> flip-bits snapshot
    const l0 = decodeFromUtm(flipBits.spec, l1.tape);

    expect(l0.tape).toEqual(welcomeModalExample.bitFlipperInput);
    expect(l0.pos).toBe(0);
    expect(l0.state).toBe(flipBits.spec.initial);
  });
});
