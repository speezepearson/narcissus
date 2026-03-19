import { useMemo, useState } from "react";
import { TuringMachineViewer } from "./TuringMachineViewer";
import { UTMViewer } from "./UTMViewer";
import { checkPalindromeSpec, write1sForeverSpec } from "./toy-machines";
import { myUtmSpec } from "./my-utm-spec";
import "./App.css";

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
      <UTMViewer
        key={tapeInput + "-utm"}
        utmSpec={myUtmSpec}
        simSpec={checkPalindromeSpec}
        initialSimTape={initialTape}
      />
    </div>
  );
}

export default App;
