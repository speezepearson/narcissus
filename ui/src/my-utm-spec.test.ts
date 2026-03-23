import { describe, expect, it } from "vitest";
import { myUtmSpec } from "./my-utm-spec";
import { must } from "./test-util";
import { flipBitsSpec } from "./toy-machines";
import { makeInitSnapshot } from "./types";
import { makeArrayTapeOverlay } from "./util";

describe("decode isolation", () => {
  it("decoded machine's set() does not write back to the UTM tape", () => {
    const sim = makeInitSnapshot(
      flipBitsSpec,
      makeArrayTapeOverlay(["0", "1"]),
    );
    const utm = myUtmSpec.encode(sim);
    const decoded = must(utm.decode());

    // Snapshot ALL UTM tape cells that we can see
    const utmBefore: string[] = [];
    for (let i = 0; ; i++) {
      const v = utm.tape.get(i);
      if (v === undefined) break;
      utmBefore.push(v);
    }

    // Write to the decoded tape
    decoded.tape.set(0, "1");
    expect(decoded.tape.get(0)).toBe("1");

    // Every UTM tape cell should be unchanged
    for (let i = 0; i < utmBefore.length; i++) {
      expect(utm.tape.get(i)).toBe(utmBefore[i]);
    }
  });

  it("decoded machine's tape does not change if the UTM continues running", () => {
    const sim = makeInitSnapshot(
      flipBitsSpec,
      makeArrayTapeOverlay(["0", "1"]),
    );
    const utm = myUtmSpec.encode(sim);
    const decoded = must(utm.decode());

    // Snapshot all decoded tape cells
    const decodedBefore: (string | undefined)[] = [];
    for (let i = 0; ; i++) {
      const v = decoded.tape.get(i);
      decodedBefore.push(v);
      if (v === undefined) break;
    }

    // Directly mutate the UTM tape at a position that encodes sim cell 0
    // (find the first cell in the tape section by scanning for a # boundary)
    let tapeSecStart = 0;
    let hashCount = 0;
    for (let i = 0; ; i++) {
      if (utm.tape.get(i) === "#") hashCount++;
      if (hashCount === 5) {
        tapeSecStart = i + 1;
        break;
      }
    }
    // Flip a bit in the first encoded cell
    const bitPos = tapeSecStart + 1; // first bit after the marker
    const before = utm.tape.get(bitPos)!;
    utm.tape.set(bitPos, before === "0" ? "1" : "0");

    // Decoded tape should still reflect the state at time of decoding
    for (let i = 0; i < decodedBefore.length; i++) {
      expect(decoded.tape.get(i)).toBe(decodedBefore[i]);
    }
  });
});
