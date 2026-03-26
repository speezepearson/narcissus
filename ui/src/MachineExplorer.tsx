import { useMemo, useState } from "react";
import { TuringMachineViewer } from "./TuringMachineViewer";
import { makeInitSnapshot, makeSimpleTapeOverlay } from "./types";
import { machineSpecs } from "./parseSpec";

export function MachineExplorer() {
  const [selectedIdx, setSelectedIdx] = useState(0);
  const [tapeInput, setTapeInput] = useState("");

  const selected = machineSpecs[selectedIdx];

  // Non-blank display chars the user can type
  const inputSymbolChars = useMemo(() => {
    return Object.entries(selected.symbolChars).filter(
      ([sym]) => sym !== selected.blank,
    );
  }, [selected]);

  const validSymbols = useMemo(() => {
    return new Set(selected.spec.allSymbols);
  }, [selected]);

  const invalidChars = useMemo(() => {
    const invalid: string[] = [];
    for (const ch of tapeInput) {
      if (!validSymbols.has(ch) && !invalid.includes(ch)) {
        invalid.push(ch);
      }
    }
    return invalid;
  }, [tapeInput, validSymbols]);

  const snapshot = useMemo(() => {
    if (invalidChars.length > 0) return null;
    const { spec } = selected;

    const tapeSymbols = [...tapeInput];

    const overlay = makeSimpleTapeOverlay<string>((i) =>
      i >= 0 && i < tapeSymbols.length ? tapeSymbols[i] : undefined,
    );

    return makeInitSnapshot(spec, overlay);
  }, [selected, tapeInput, invalidChars]);

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

      <p style={{ margin: "0 0 8px 0", fontSize: "0.9em" }}>
        Tape alphabet:{" "}
        {inputSymbolChars.map(([sym, ch]) => (
          <code key={sym} style={{ marginRight: "8px" }}>
            {ch}
          </code>
        ))}
        <code style={{ marginRight: "8px", opacity: 0.5 }}>
          {selected.symbolChars[selected.blank]} (blank)
        </code>
      </p>

      <label className="tm-tape-input">
        Tape:{" "}
        <input
          type="text"
          value={tapeInput}
          onChange={(e) => setTapeInput(e.target.value)}
          placeholder={`Type using: ${inputSymbolChars.map(([, ch]) => ch).join("")}`}
          spellCheck={false}
        />
      </label>

      {invalidChars.length > 0 && (
        <p style={{ color: "red", margin: "8px 0" }}>
          Invalid character{invalidChars.length > 1 ? "s" : ""}:{" "}
          {invalidChars.map((ch) => (
            <code key={ch} style={{ marginRight: "4px" }}>
              {ch}
            </code>
          ))}
          — allowed:{" "}
          {selected.spec.allSymbols.map((s) => (
            <code key={s} style={{ marginRight: "4px" }}>
              {s}
            </code>
          ))}
        </p>
      )}
      {snapshot && (
        <TuringMachineViewer
          key={`${selectedIdx}-${tapeInput}`}
          init={snapshot}
        />
      )}
    </div>
  );
}
