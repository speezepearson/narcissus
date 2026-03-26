import { useCallback, useEffect, useMemo, useState } from "react";
import { TuringMachineViewer } from "./TuringMachineViewer";
import { makeInitSnapshot, type State, type TuringMachineSnapshot } from "./types";
import { machineSpecs } from "./parseSpec";
import { TapeInput, useTapeInput } from "./TapeInput";
import { decodeFromUtm, encodeForUtm } from "./utmEncoding";
import { TapeView } from "./TapeView";

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

  // L1: UTM simulating flip-bits
  const utm1Snapshot = useMemo(() => {
    if (!snapshot) return null;
    const utmTape = encodeForUtm(flipBitsSpec.spec, snapshot);
    return makeInitSnapshot(utmSpec.spec, utmTape);
  }, [snapshot]);

  // L2: UTM simulating L1
  const utm2Snapshot = useMemo(() => {
    if (!utm1Snapshot) return null;
    const utmTape = encodeForUtm(utmSpec.spec, utm1Snapshot);
    return makeInitSnapshot(utmSpec.spec, utmTape);
  }, [utm1Snapshot]);

  // Decoded L1 -> flip-bits guest
  const initialDecodedL1 = useMemo(() => {
    if (!utm1Snapshot) return null;
    return decodeFromUtm(flipBitsSpec.spec, utm1Snapshot.tape);
  }, [utm1Snapshot]);

  const [decodedFromL1, setDecodedFromL1] = useState<TuringMachineSnapshot | null>(null);

  useEffect(() => {
    setDecodedFromL1(initialDecodedL1);
  }, [initialDecodedL1]);

  const onUtm1StateChange = useCallback(
    (_oldState: State, cur: TuringMachineSnapshot) => {
      if (cur.state === "Init") {
        try {
          setDecodedFromL1(decodeFromUtm(flipBitsSpec.spec, cur.tape));
        } catch {
          // SWALLOW_EXCEPTION: UTM may be mid-operation with an undecodable tape during Init transitions
        }
      }
    },
    [],
  );

  // Decoded L2 -> L1 UTM guest, then L1 -> flip-bits guest
  const initialDecodedL2 = useMemo(() => {
    if (!utm2Snapshot) return null;
    const l1 = decodeFromUtm(utmSpec.spec, utm2Snapshot.tape);
    const l0 = decodeFromUtm(flipBitsSpec.spec, l1.tape);
    return { l1, l0 };
  }, [utm2Snapshot]);

  const [decodedFromL2, setDecodedFromL2] = useState<{
    l1: TuringMachineSnapshot;
    l0: TuringMachineSnapshot;
  } | null>(null);

  useEffect(() => {
    setDecodedFromL2(initialDecodedL2);
  }, [initialDecodedL2]);

  const onUtm2StateChange = useCallback(
    (_oldState: State, cur: TuringMachineSnapshot) => {
      if (cur.state === "Init") {
        try {
          const l1 = decodeFromUtm(utmSpec.spec, cur.tape);
          const l0 = decodeFromUtm(flipBitsSpec.spec, l1.tape);
          setDecodedFromL2({ l1, l0 });
        } catch {
          // SWALLOW_EXCEPTION: UTM may be mid-operation with an undecodable tape during Init transitions
        }
      }
    },
    [],
  );

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
        zIndex: 1000,
        overflowY: "auto",
        display: "flex",
        justifyContent: "center",
        padding: "32px 0",
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
          alignSelf: "flex-start",
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

        <hr style={{ margin: "3em 0" }} />

        <p style={{ textAlign: 'left', marginBottom: "16px", lineHeight: "1.6" }}>
          Here's a universal Turing machine simulating the same flip-bits machine on the same input:
        </p>

        {utm1Snapshot && <TuringMachineViewer key={`utm1-${flipBitsInput}`} init={utm1Snapshot} onStateChange={onUtm1StateChange} />}

        {decodedFromL1 && (
          <>
            <p style={{ textAlign: 'left', marginBottom: "8px", marginTop: "16px", lineHeight: "1.6" }}>
              Decoded from the UTM's tape:
            </p>
            <TapeView tm={decodedFromL1} />
          </>
        )}

        <hr style={{ margin: "3em 0" }} />

        <p style={{ textAlign: 'left', marginBottom: "16px", lineHeight: "1.6" }}>
          Here's a UTM simulating that one:
        </p>

        {utm2Snapshot && <TuringMachineViewer key={`utm2-${flipBitsInput}`} init={utm2Snapshot} onStateChange={onUtm2StateChange} />}

        {decodedFromL2 && (
          <>
            <p style={{ textAlign: 'left', marginBottom: "8px", marginTop: "16px", lineHeight: "1.6" }}>
              Decoded (middle UTM):
            </p>
            <TapeView tm={decodedFromL2.l1} />
            <p style={{ textAlign: 'left', marginBottom: "8px", marginTop: "8px", lineHeight: "1.6" }}>
              Decoded (bit-flipper):
            </p>
            <TapeView tm={decodedFromL2.l0} />
          </>
        )}

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
