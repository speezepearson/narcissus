import { useMemo, useState } from "react";
import "./App.css";
import { TuringMachineViewer } from "./TuringMachineViewer";
import { UTMViewer } from "./UTMViewer";
import { MyUtmSnapshot } from "./my-utm-spec";
import {
  acceptImmediatelySpec,
  checkPalindromeSpec,
  doubleXSpec,
  flipBitsSpec,
  write1sForeverSpec,
} from "./toy-machines";
import { makeInitSnapshot } from "./types";
import { makeArrayTapeOverlay } from "./util";

export function SamplerPlatter() {
  const acceptImmediatelySnapshot = useMemo(
    () => makeInitSnapshot(acceptImmediatelySpec, makeArrayTapeOverlay([])),
    [],
  );

  const initialWrite1sForeverSnapshot = useMemo(
    () => makeInitSnapshot(write1sForeverSpec, makeArrayTapeOverlay([])),
    [],
  );

  const [palindromeInput, setPalindromeInput] = useState("abba");
  const initialPalindromeSnapshot = useMemo(
    () =>
      makeInitSnapshot(
        checkPalindromeSpec,
        makeArrayTapeOverlay(
          palindromeInput
            .split("")
            .filter((c): c is "a" | "b" => c === "a" || c === "b"),
        ),
      ),
    [palindromeInput],
  );

  const [flipBitsInput, setFlipBitsInput] = useState("011");
  const initialFlipBitsSnapshot = useMemo(
    () =>
      makeInitSnapshot(
        flipBitsSpec,
        makeArrayTapeOverlay(
          flipBitsInput
            .split("")
            .filter((c): c is "0" | "1" => c === "0" || c === "1"),
        ),
      ),
    [flipBitsInput],
  );

  const [doubleXCount, setDoubleXCount] = useState(5);
  const initialDoubleXSnapshot = useMemo(
    () =>
      makeInitSnapshot(
        doubleXSpec,
        makeArrayTapeOverlay([
          "$",
          ...Array.from({ length: doubleXCount }, () => "X"),
        ]),
      ),
    [doubleXCount],
  );

  const [utmSimChoice, setUtmSimChoice] = useState<
    "doubleX" | "flipBits" | "acceptImmediately" | "write1sForever"
  >("doubleX");
  const [utmKey, initUtmSnapshot] = useMemo(() => {
    switch (utmSimChoice) {
      case "doubleX":
        return [
          `doubleX-${doubleXCount}`,
          MyUtmSnapshot.fromSimSnapshot(initialDoubleXSnapshot),
        ];
      case "flipBits":
        return [
          `flipBits-${flipBitsInput}`,
          MyUtmSnapshot.fromSimSnapshot(initialFlipBitsSnapshot),
        ];
      case "acceptImmediately":
        return [
          `acceptImmediately`,
          MyUtmSnapshot.fromSimSnapshot(acceptImmediatelySnapshot),
        ];
      case "write1sForever":
        return [
          `write1sForever`,
          MyUtmSnapshot.fromSimSnapshot(initialWrite1sForeverSnapshot),
        ];
    }
  }, [
    acceptImmediatelySnapshot,
    doubleXCount,
    flipBitsInput,
    initialDoubleXSnapshot,
    initialFlipBitsSnapshot,
    initialWrite1sForeverSnapshot,
    utmSimChoice,
  ]);

  return (
    <div style={{ padding: "24px" }}>
      <h2 style={{ marginTop: "32px" }}>Accept Immediately</h2>
      <TuringMachineViewer init={acceptImmediatelySnapshot} />

      <h2 style={{ marginTop: "32px" }}>Write 1s Forever</h2>
      <TuringMachineViewer init={initialWrite1sForeverSnapshot} />

      <h2 style={{ marginTop: "32px" }}>Flip Bits</h2>
      <label className="tm-tape-input">
        Tape:
        <input
          type="text"
          value={flipBitsInput}
          onChange={(e) => setFlipBitsInput(e.target.value)}
          placeholder="e.g. 011"
          spellCheck={false}
        />
      </label>
      <TuringMachineViewer init={initialFlipBitsSnapshot} />
      <h2>Palindrome Checker</h2>
      <label className="tm-tape-input">
        Tape:
        <input
          type="text"
          value={palindromeInput}
          onChange={(e) => setPalindromeInput(e.target.value)}
          placeholder="e.g. abba"
          spellCheck={false}
        />
      </label>
      <TuringMachineViewer
        key={palindromeInput}
        init={initialPalindromeSnapshot}
      />

      <h2 style={{ marginTop: "32px" }}>Double X</h2>
      <label className="tm-tape-input">
        {" "}
        Number of X's:
        <input
          type="number"
          min={0}
          max={100}
          step={1}
          value={doubleXCount}
          onChange={(e) => setDoubleXCount(Number(e.target.value))}
        />
      </label>
      <TuringMachineViewer key={doubleXCount} init={initialDoubleXSnapshot} />

      <h2 style={{ marginTop: "32px" }}>UTM Simulation</h2>
      <label>
        Simulate:
        <select
          value={utmSimChoice}
          onChange={(e) =>
            setUtmSimChoice(
              e.target.value as
                | "doubleX"
                | "flipBits"
                | "acceptImmediately"
                | "write1sForever",
            )
          }
        >
          <option value="doubleX">Double X</option>
          <option value="flipBits">Flip Bits</option>
          <option value="acceptImmediately">Accept Immediately</option>
          <option value="write1sForever">Write 1s Forever</option>
        </select>
      </label>
      <UTMViewer key={utmKey} init={initUtmSnapshot} />
    </div>
  );
}
