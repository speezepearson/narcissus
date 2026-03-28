import { describe, expect, it } from "vitest";
import { makeInitSnapshot } from "./types";
import { machineSpecs } from "./parseSpec";
import { encodeForUtm } from "./utmEncoding";

const flipBits = machineSpecs.find((s) => s.name === "Flip Bits")!;

describe("UTM encoding cross-check with Rust", () => {
  it("encoding of flip-bits on empty tape matches Rust output (modulo rule order)", () => {
    // From `cargo run --bin export_flip_bits_encoding`:
    // New layout: # ACC # BLANK # RULES $ STATE # TAPE
    const rustOutput = "#1#00#.0|01|0|10|R;.0|00|1|00|L;.0|10|0|01|R$0#^00";

    const snapshot = makeInitSnapshot(flipBits.spec, [flipBits.spec.blank]);
    const encoded = encodeForUtm(flipBits.spec, snapshot);
    const tsOutput = encoded.join("");

    // Split by $ to separate RULES from STATE
    const rustParts = rustOutput.split("$");
    const tsParts = tsOutput.split("$");
    expect(tsParts.length).toBe(rustParts.length);

    // Left of $: # ACC # BLANK # RULES
    const rustLeftSections = rustParts[0].split("#");
    const tsLeftSections = tsParts[0].split("#");
    expect(tsLeftSections.length).toBe(rustLeftSections.length);

    // Rules section (last # section before $) may differ in order
    const rulesIdx = rustLeftSections.length - 1;
    const rustRules = new Set(rustLeftSections[rulesIdx].split(";"));
    const tsRules = new Set(tsLeftSections[rulesIdx].split(";"));
    expect(tsRules).toEqual(rustRules);

    // All other left sections must match exactly
    for (let i = 0; i < rustLeftSections.length; i++) {
      if (i === rulesIdx) continue;
      expect(tsLeftSections[i], `left section ${i}`).toBe(rustLeftSections[i]);
    }

    // Right of $: STATE # TAPE
    expect(tsParts[1], "right of $").toBe(rustParts[1]);
  });
});
