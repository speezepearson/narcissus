import { useState } from "react";
import { TuringMachineViewer } from "./TuringMachineViewer";
import { machineSpecs } from "./parseSpec";
import { TapeInput } from "./TapeInput";
import { useTapeInput } from "./useTapeInput";

export function MachineExplorer() {
  const [selectedIdx, setSelectedIdx] = useState(0);
  const [tapeInput, setTapeInput] = useState("");

  const selected = machineSpecs[selectedIdx];

  const { snapshot } = useTapeInput(selected.spec, tapeInput);

  return (
    <div style={{ padding: "24px" }}>
      <h2>Machine Explorer</h2>

      <div style={{ marginBottom: "16px" }}>
        <label>
          Machine:{" "}
          <select
            value={selectedIdx}
            onChange={(e) => {
              setSelectedIdx(Number(e.target.value));
              setTapeInput("");
            }}
          >
            {machineSpecs.map((s, i) => (
              <option key={i} value={i}>
                {s.name}
              </option>
            ))}
          </select>
        </label>
      </div>

      <p style={{ fontStyle: "italic", margin: "0 0 12px 0" }}>
        {selected.description}
      </p>

      <TapeInput parsed={selected} value={tapeInput} onChange={setTapeInput} />
      {snapshot && (
        <TuringMachineViewer
          key={`${selectedIdx}-${tapeInput}`}
          init={snapshot}
          stateDescriptions={selected.stateDescriptions}
          graph={selected.graph}
        />
      )}
    </div>
  );
}
