import { useMemo, useState } from "react";
import { TuringMachineViewer } from "./TuringMachineViewer";
import { makeInitSnapshot, makeSimpleTapeOverlay } from "./types";
import { machineSpecs } from "./parseSpec";

const STORAGE_KEY = "welcomeModalDismissed";
const PALINDROME_INPUT = "ABCBA";

export function WelcomeModal() {
  const [visible, setVisible] = useState(
    () => !localStorage.getItem(STORAGE_KEY),
  );

  const snapshot = useMemo(() => {
    const parsed = machineSpecs.find((s) => s.name === "Check Palindrome");
    if (!parsed) return null;
    const tapeSymbols = [...PALINDROME_INPUT];
    const overlay = makeSimpleTapeOverlay<string>((i) =>
      i >= 0 && i < tapeSymbols.length ? tapeSymbols[i] : undefined,
    );
    return makeInitSnapshot(parsed.spec, overlay);
  }, []);

  if (!visible) return null;

  const dismiss = () => {
    localStorage.setItem(STORAGE_KEY, "1");
    setVisible(false);
  };

  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        background: "rgba(0, 0, 0, 0.5)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 1000,
      }}
      onClick={dismiss}
    >
      <div
        style={{
          background: "var(--bg)",
          border: "1px solid var(--border)",
          borderRadius: "12px",
          padding: "32px",
          maxWidth: "480px",
          width: "90%",
          boxShadow: "var(--shadow)",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <h2 style={{ marginTop: 0 }}>Welcome to the Self-Simulating Tower!</h2>

        <p style={{ marginBottom: "16px", lineHeight: "1.6" }}>
          Here is a Turing machine:
        </p>

        {snapshot && <TuringMachineViewer init={snapshot} />}

        <button
          onClick={dismiss}
          style={{
            fontFamily: "var(--mono)",
            fontSize: "14px",
            padding: "8px 20px",
            borderRadius: "6px",
            border: "1px solid var(--border)",
            background: "var(--accent)",
            color: "#fff",
            cursor: "pointer",
          }}
        >
          Got it
        </button>
      </div>
    </div>
  );
}
