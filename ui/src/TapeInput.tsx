import { useMemo } from "react";
import {
  makeInitSnapshot,
  Symbol,
  type TuringMachineSnapshot,
  type TuringMachineSpec,
} from "./types";
import type { ParsedSpec } from "./parseSpec";

type TapeInputProps = {
  parsed: ParsedSpec;
  value: string;
  onChange: (value: string) => void;
};

export type TapeInputResult = {
  snapshot: TuringMachineSnapshot | null;
  invalidChars: string[];
};

export function useTapeInput(
  spec: TuringMachineSpec,
  value: string,
): TapeInputResult {
  const validSymbols = useMemo(() => new Set(spec.allSymbols), [spec]);

  const invalidChars = useMemo(() => {
    const invalid: string[] = [];
    for (const ch of value) {
      if (!validSymbols.has(ch as Symbol) && !invalid.includes(ch)) {
        invalid.push(ch);
      }
    }
    return invalid;
  }, [value, validSymbols]);

  const snapshot = useMemo(() => {
    if (invalidChars.length > 0) return null;
    const tape = [...value].map((c) => Symbol.parse(c));
    return makeInitSnapshot(spec, tape);
  }, [spec, value, invalidChars]);

  return { snapshot, invalidChars };
}

export function TapeInput({ parsed, value, onChange }: TapeInputProps) {
  const { invalidChars } = useTapeInput(parsed.spec, value);

  const inputSymbolChars = useMemo(() => {
    return Object.entries(parsed.symbolChars).filter(
      ([sym]) => sym !== parsed.blank,
    );
  }, [parsed]);

  return (
    <div>
      <label className="tm-tape-input">
        Input:{" "}
        <input
          type="text"
          value={value}
          onChange={(e) => onChange(e.target.value)}
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
          {parsed.spec.allSymbols.map((s) => (
            <code key={s} style={{ marginRight: "4px" }}>
              {s}
            </code>
          ))}
        </p>
      )}
    </div>
  );
}
