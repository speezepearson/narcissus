import { describe, expect, it } from "vitest";
import {
  getStatus,
  makeInitSnapshot,
  run,
  Symbol,
} from "./types";
import { machineSpecs } from "./parseSpec";
import { decodeFromUtm, encodeForUtm } from "./utmEncoding";

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

  it("UTM simulation matches direct execution", async () => {
    // Run flip-bits directly
    const direct = makeInitSnapshot(flipBits.spec, syms("10110"));
    await run(direct);

    // Encode flip-bits + "10110" for the UTM
    const guestInit = makeInitSnapshot(flipBits.spec, syms("10110"));
    const utmTape = encodeForUtm(flipBits.spec, guestInit);

    // Run the UTM
    const utmSnapshot = makeInitSnapshot(utm.spec, utmTape);
    await run(utmSnapshot, { gas: 1e8 });
    expect(getStatus(utmSnapshot)).toBe("accept");

    // Decode the UTM's tape back into a flip-bits snapshot
    const decoded = decodeFromUtm(flipBits.spec, utmSnapshot.tape);

    // The decoded guest tape should match direct execution
    const directTape = direct.tape.slice(0, 5).join("");
    const decodedTape = decoded.tape.slice(0, 5).join("");
    expect(decodedTape).toBe(directTape);
    expect(decoded.state).toBe(direct.state);
  });
});
