import { describe, expect, it } from "vitest";
import { makeInitSnapshot } from "./types";
import { machineSpecs } from "./parseSpec";
import { encodeForUtm } from "./utmEncoding";

const flipBits = machineSpecs.find((s) => s.name === "Flip Bits")!;

describe("UTM encoding cross-check with Rust", () => {
  it("encoding of flip-bits on empty tape matches Rust output (modulo rule order)", () => {
    // From `cargo run --bin export_flip_bits_encoding`:
    const rustOutput = "$1#00#.0|01|0|10|R;.0|00|1|00|L;.0|10|0|01|R#0#00#^00";

    const snapshot = makeInitSnapshot(flipBits.spec, [flipBits.spec.blank]);
    const encoded = encodeForUtm(flipBits.spec, snapshot);
    const tsOutput = encoded.join("");

    // Split into sections by #
    const rustSections = rustOutput.split("#");
    const tsSections = tsOutput.split("#");
    expect(tsSections.length).toBe(rustSections.length);

    // Rules section (index 2) may differ in order; compare as sets
    const rustRules = new Set(rustSections[2].split(";"));
    const tsRules = new Set(tsSections[2].split(";"));
    expect(tsRules).toEqual(rustRules);

    // All other sections must match exactly
    for (let i = 0; i < rustSections.length; i++) {
      if (i === 2) continue; // skip rules (checked above)
      expect(tsSections[i], `section ${i}`).toBe(rustSections[i]);
    }
  });
});
