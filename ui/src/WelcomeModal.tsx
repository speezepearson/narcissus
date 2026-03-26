import { useMemo, useState } from "react";
import { TuringMachineViewer } from "./TuringMachineViewer";
import { makeInitSnapshot, Symbol } from "./types";
import { machineSpecs } from "./parseSpec";

const STORAGE_KEY = "welcomeModalDismissed";

export function WelcomeModal() {
  const [visible, setVisible] = useState(
    true // () => !localStorage.getItem(STORAGE_KEY),
  );

  const [palindromeInput, setPalindromeInput] = useState("racecar");

  const snapshot = useMemo(() => {
    const parsed = machineSpecs.find((s) => s.name === "Check Palindrome");
    if (!parsed) throw new Error("Check Palindrome spec not found");
    const tapeSymbols = [...palindromeInput];
    return makeInitSnapshot(parsed.spec, tapeSymbols.map(c => Symbol.parse(c)));
  }, [palindromeInput]);


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

        <p style={{ textAlign: 'left', marginBottom: "16px", lineHeight: "1.6" }}>
          Here's a Turing machine:
        </p>

        <TuringMachineViewer key={palindromeInput} init={snapshot} />

        <p style={{ textAlign: 'left', marginBottom: "16px", lineHeight: "1.6" }}>
          It's a very simple mechanism: the machine has some list of "states" it can be in,
          and it's pointed at a "tape" full of cells, each cell containing one of some fixed set of symbols
          (in this case, "blank" or a letter).
          At each time step, the machine looks at the cell it's pointed at, and depending on that symbol and the state it's in,
          changes to a new state, writes a new symbol, and moves left or right.
        </p>

        <p style={{ textAlign: 'left', marginBottom: "16px", lineHeight: "1.6" }}>
          This 
        </p>

        <p style={{ textAlign: 'left', marginBottom: "16px", lineHeight: "1.6" }}>
          You can run it on input of your choice:
          <input
            type="text"
            value={palindromeInput}
            onChange={(e) => setPalindromeInput(e.target.value)}
            style={{ marginLeft: "8px", width: "100%" }}
          />
        </p>

        <button
          onClick={dismiss}
          style={{
            marginTop: "16px",
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
          Close
        </button>
      </div>
    </div>
  );
}
