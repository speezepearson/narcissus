import { useMemo, useState } from "react";
import "./App.css";
import { TuringMachineViewer } from "./TuringMachineViewer";
import { UTMViewer } from "./UTMViewer";
import { MyUtmSnapshot } from "./my-utm-spec";
import {
  checkPalindromeSpec,
  doubleXSpec,
  flipBitsSpec,
  write1sForeverSpec,
} from "./toy-machines";
import { makeInitSnapshot } from "./types";
import { makeArrayTapeOverlay } from "./util";

function App() {
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

  const initialWrite1sForeverSnapshot = useMemo(
    () => makeInitSnapshot(write1sForeverSpec, makeArrayTapeOverlay([])),
    [],
  );

  const initUtmSnapshot = useMemo(
    () =>
      MyUtmSnapshot.fromSimSnapshot(
        makeInitSnapshot(flipBitsSpec, makeArrayTapeOverlay(["0", "1"])),
      ),
    [],
  );

  return (
    <div style={{ padding: "24px" }}>
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

      <h2 style={{ marginTop: "32px" }}>Write 1s Forever</h2>
      <TuringMachineViewer init={initialWrite1sForeverSnapshot} />

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
      <UTMViewer init={initUtmSnapshot} />
    </div>
  );
}

export default App;
