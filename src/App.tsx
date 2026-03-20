import { useMemo, useState } from "react";
import "./App.css";
import { TuringMachineViewer } from "./TuringMachineViewer";
import { MyUTMViewer } from "./UTMViewer";
import { infiniteUtmTapeBackground } from "./infinite-utm";
import { myUtmSpec } from "./my-utm-spec";
import myUtmOptimizationHints from "./my-utm-spec-transition-optimization-hints";
import {
  checkPalindromeSpec,
  doubleXSpec,
  write1sForeverSpec,
} from "./toy-machines";
import { makeInitSnapshot, makeSimpleTapeOverlay } from "./types";
import { makeArrayTapeOverlay } from "./util";

function App() {
  const [tapeInput, setTapeInput] = useState("abba");

  const initialTape = useMemo(
    () =>
      makeArrayTapeOverlay(
        tapeInput
          .split("")
          .filter((c): c is "a" | "b" => c === "a" || c === "b"),
      ),
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
      <TuringMachineViewer
        spec={write1sForeverSpec}
        initialTape={makeArrayTapeOverlay([])}
      />

      <h2 style={{ marginTop: "32px" }}>Double X</h2>
      <TuringMachineViewer
        spec={doubleXSpec}
        initialTape={makeArrayTapeOverlay([
          "$",
          ...Array.from({ length: 100 }, () => "X"),
        ])}
      />

      <h2 style={{ marginTop: "32px" }}>UTM Simulation</h2>
      <MyUTMViewer
        key={tapeInput + "-utm"}
        initialSim={makeInitSnapshot(
          myUtmSpec,
          makeSimpleTapeOverlay(infiniteUtmTapeBackground),
        )}
        optimizationHints={myUtmOptimizationHints}
      />
    </div>
  );
}

export default App;
