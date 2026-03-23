import { useEffect, useMemo, useState } from "react";
import { TuringMachineViewer } from "./TuringMachineViewer";
import {
  type Dir,
  type TuringMachineSpec,
  makeInitSnapshot,
  makeSimpleTapeOverlay,
} from "./types";

type JsonSpec = {
  name: string;
  description: string;
  allStates: string[];
  allSymbols: string[];
  initial: string;
  acceptingStates: string[];
  blank: string;
  rules: Record<string, Record<string, [string, string, string]>>;
  symbolChars: Record<string, string>;
};

/**
 * Remap symbols from Rust enum names (e.g. "Zero") to display chars (e.g. "0")
 * so TapeView naturally renders single-char symbols.
 */
function parseSpec(json: JsonSpec): {
  spec: TuringMachineSpec<string, string>;
  symbolChars: Record<string, string>;
} {
  const sc = json.symbolChars; // rustName -> displayChar

  const rules = new Map<string, Map<string, [string, string, Dir]>>();
  for (const [state, symbolMap] of Object.entries(json.rules)) {
    const inner = new Map<string, [string, string, Dir]>();
    for (const [symbol, [ns, nsym, dir]] of Object.entries(symbolMap)) {
      inner.set(sc[symbol], [ns, sc[nsym], dir as Dir]);
    }
    rules.set(state, inner);
  }

  return {
    spec: {
      allStates: json.allStates,
      allSymbols: json.allSymbols.map((s) => sc[s]),
      initial: json.initial,
      acceptingStates: new Set(json.acceptingStates),
      blank: sc[json.blank],
      rules,
    },
    symbolChars: json.symbolChars,
  };
}

export function MachineExplorer() {
  const [specs, setSpecs] = useState<JsonSpec[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selectedIdx, setSelectedIdx] = useState(0);
  const [tapeInput, setTapeInput] = useState("");

  useEffect(() => {
    fetch("machine-specs.json")
      .then((r) => {
        if (!r.ok) throw new Error(`HTTP ${r.status}`);
        return r.json();
      })
      .then((data: JsonSpec[]) => setSpecs(data))
      .catch((e: unknown) => setError(String(e)));
  }, []);

  const selected = specs?.[selectedIdx] ?? null;

  const parsed = useMemo(() => {
    if (!selected) return null;
    return parseSpec(selected);
  }, [selected]);

  // Non-blank display chars the user can type
  const inputSymbolChars = useMemo(() => {
    if (!selected) return [];
    return Object.entries(selected.symbolChars).filter(
      ([sym]) => sym !== selected.blank,
    );
  }, [selected]);

  const validSymbols = useMemo(() => {
    if (!parsed) return new Set<string>();
    return new Set(parsed.spec.allSymbols);
  }, [parsed]);

  const snapshot = useMemo(() => {
    if (!parsed) return null;
    const { spec } = parsed;

    // Each char in the input that is a valid display-char symbol gets added to the tape
    const tapeSymbols: string[] = [];
    for (const ch of tapeInput) {
      if (validSymbols.has(ch)) {
        tapeSymbols.push(ch);
      }
    }

    const overlay = makeSimpleTapeOverlay<string>(
      (i) => (i >= 0 && i < tapeSymbols.length ? tapeSymbols[i] : undefined),
    );

    return makeInitSnapshot(spec, overlay);
  }, [parsed, tapeInput, validSymbols]);

  if (error) return <div style={{ color: "red" }}>Error loading specs: {error}</div>;
  if (!specs) return <div>Loading machine specs...</div>;
  if (!selected || !parsed || !snapshot) return null;

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
            {specs.map((s, i) => (
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

      <TuringMachineViewer key={`${selectedIdx}-${tapeInput}`} init={snapshot} />
    </div>
  );
}
