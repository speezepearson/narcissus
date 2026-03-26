import { useMemo, useState } from "react";
import { TuringMachineViewer } from "./TuringMachineViewer";
import { makeInitSnapshot } from "./types";
import { machineSpecs } from "./parseSpec";
import { TapeInput, useTapeInput } from "./TapeInput";
import { encodeForUtm } from "./utmEncoding";

const STORAGE_KEY = "welcomeModalDismissed";

function getSpec(name: string) {
  const spec = machineSpecs.find((s) => s.name === name);
  if (!spec) throw new Error(`${name} spec not found`);
  return spec;
}
const flipBitsSpec = getSpec("Flip Bits");
const utmSpec = getSpec("Universal Turing Machine");

export function WelcomeModal() {
  const [visible, setVisible] = useState(
    true // () => !localStorage.getItem(STORAGE_KEY),
  );

  const [flipBitsInput, setFlipBitsInput] = useState("010101");

  const { snapshot } = useTapeInput(flipBitsSpec.spec, flipBitsInput);

  const utmSnapshot = useMemo(() => {
    if (!snapshot) return null;
    const utmTape = encodeForUtm(flipBitsSpec.spec, snapshot);
    return makeInitSnapshot(utmSpec.spec, utmTape);
  }, [snapshot]);


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
          maxWidth: "40em",
          width: "90%",
          boxShadow: "var(--shadow)",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <h2 style={{ marginTop: 0 }}>Welcome to the Self-Simulating Tower!</h2>

        <p style={{ textAlign: 'left', marginBottom: "16px", lineHeight: "1.6" }}>
          Here's a simple Turing machine (you know what a <a href="https://en.wikipedia.org/wiki/Turing_machine">Turing machine</a> is, right?),
          which flips all the bits on its tape:
        </p>

        {snapshot && <TuringMachineViewer key={flipBitsInput} init={snapshot} />}

        <TapeInput parsed={flipBitsSpec} value={flipBitsInput} onChange={setFlipBitsInput} />

        <p style={{ textAlign: 'left', marginBottom: "16px", lineHeight: "1.6" }}>
          And here's a Universal Turing Machine simulating the same flip-bits machine on the same input:
        </p>

        {utmSnapshot && <TuringMachineViewer key={`utm-${flipBitsInput}`} init={utmSnapshot} />}

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
