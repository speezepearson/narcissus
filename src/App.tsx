import { useMemo, useState } from "react";
import { TuringMachineViewer } from "./TuringMachineViewer";
import { MyUTMViewer } from "./UTMViewer";
import {
  checkPalindromeSpec,
  flipBitsSpec,
  write1sForeverSpec,
} from "./toy-machines";
import "./App.css";
import { makeInitSnapshot } from "./types";
import { myUtmSpec } from "./my-utm-spec";
import { makeArrayTapeOverlay } from "./util";

function App() {
  const [tapeInput, setTapeInput] = useState("abba");

  const initialTape = useMemo(
    () =>
      tapeInput.split("").filter((c): c is "a" | "b" => c === "a" || c === "b"),
    [tapeInput],
  );

  return (
    <div style={{ padding: "24px" }}>
      <h2>Palindrome Checker</h2>
      <label className="tm-tape-input">
        Tape:
        <input
          type="text"
          value={tapeInput}
          onChange={(e) => setTapeInput(e.target.value)}
          placeholder="e.g. abba"
          spellCheck={false}
        />
      </label>
      <TuringMachineViewer
        key={tapeInput}
        spec={checkPalindromeSpec}
        initialTape={initialTape}
      />

      <h2 style={{ marginTop: "32px" }}>Write 1s Forever</h2>
      <TuringMachineViewer spec={write1sForeverSpec} initialTape={[]} />

      <h2 style={{ marginTop: "32px" }}>UTM Simulation</h2>
      <MyUTMViewer
        key={tapeInput + "-utm"}
        initialSim={myUtmSpec.encode(
          makeInitSnapshot(flipBitsSpec, makeArrayTapeOverlay(["0", "1"])),
        )}
      />
    </div>
  );
}

export default App;
