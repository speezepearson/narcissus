import { useCallback, useEffect, useMemo, useState } from "react";
import { TuringMachineViewer } from "./TuringMachineViewer";
import {
  makeInitSnapshot,
  type State,
  type TuringMachineSnapshot,
} from "./types";
import { machineSpecs, rustExport } from "./parseSpec";
import { decodeFromUtm } from "./utmEncoding";
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
    () => !localStorage.getItem(STORAGE_KEY),
  );

  const { welcomeModalExample } = rustExport;

  const snapshot = useMemo(
    () =>
      makeInitSnapshot(
        flipBitsSpec.spec,
        welcomeModalExample.bitFlipperInput,
      ),
    [welcomeModalExample],
  );

  // L1: UTM simulating flip-bits
  const utm1Snapshot = useMemo(
    () => makeInitSnapshot(utmSpec.spec, welcomeModalExample.utmInput),
    [welcomeModalExample],
  );

  // L2: UTM simulating L1
  const utm2Snapshot = useMemo(
    () => makeInitSnapshot(utmSpec.spec, welcomeModalExample.doubleUtmInput),
    [welcomeModalExample],
  );

  // Decoded L1 -> flip-bits guest
  const initialDecodedL1 = useMemo(
    () => decodeFromUtm(flipBitsSpec.spec, utm1Snapshot.tape),
    [utm1Snapshot],
  );

  const [decodedFromL1, setDecodedFromL1] =
    useState<TuringMachineSnapshot | null>(null);

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

  const dismiss = () => {
    localStorage.setItem(STORAGE_KEY, "1");
    setVisible(false);
  };

  if (!visible) {
    return (
      <button
        onClick={() => setVisible(true)}
        aria-label="Show welcome info"
        style={{
          position: "fixed",
          bottom: "20px",
          right: "20px",
          zIndex: 999,
          width: "36px",
          height: "36px",
          borderRadius: "50%",
          border: "1px solid var(--border)",
          background: "var(--bg)",
          color: "var(--fg)",
          fontSize: "18px",
          fontFamily: "var(--mono)",
          cursor: "pointer",
          boxShadow: "var(--shadow)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        ?
      </button>
    );
  }

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
        textAlign: "left",
      }}
      onClick={dismiss}
    >
      <div
        style={{
          background: "var(--bg)",
          border: "1px solid var(--border)",
          borderRadius: "12px",
          padding: "32px",
          maxWidth: "70%",
          width: "90%",
          boxShadow: "var(--shadow)",
          alignSelf: "flex-start",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <h2 style={{ marginTop: 0, textAlign: "center" }}>
          Welcome to the Self-Simulating Tower!
        </h2>

        <p style={{ marginBottom: "16px", lineHeight: "1.6" }}>
          Here's a simple Turing machine (you know what a{" "}
          <a href="https://en.wikipedia.org/wiki/Turing_machine">
            Turing machine
          </a>{" "}
          is, right?), which flips all the bits on its tape:
        </p>

        <div style={{ margin: "0 10%" }}>
          <TuringMachineViewer
            init={snapshot}
            initialFps={5}
            stateDescriptions={flipBitsSpec.stateDescriptions}
          />
        </div>

        <hr style={{ margin: "3em 0" }} />

        <p style={{ marginBottom: "16px", lineHeight: "1.6" }}>
          Here's a universal Turing machine simulating the same flip-bits
          machine on the same input:
        </p>

        <div style={{ margin: "0 10%" }}>
          <TuringMachineViewer
            init={utm1Snapshot}
            onStateChange={onUtm1StateChange}
            initialFps={30}
            stateDescriptions={utmSpec.stateDescriptions}
          />

          {decodedFromL1 && (
            <>
              <p
                style={{
                  textAlign: "left",
                  marginBottom: "8px",
                  marginTop: "16px",
                  lineHeight: "1.6",
                }}
              >
                Decoded from the UTM's tape:
              </p>
              <TapeView
                tm={decodedFromL1}
                stateDescriptions={flipBitsSpec.stateDescriptions}
              />
            </>
          )}
        </div>

        <p style={{ marginBottom: "16px", lineHeight: "1.6" }}>
          You can kinda see how it works:
          <ul>
            <li>
              It has a description of all the simulated machine's state
              transitions: <code>.0|10|0|01|R;</code> means "if you're in state
              0, and you see symbol 10, then stay in state 0, and write symbol
              01, and move right."
            </li>
            <li>
              After the next <code>#</code>s, there's: the list of accepting
              states (here, just state 1); the machine's current state (starting
              in state 0); the machine's "blank" symbol that should be used to
              fill in the right-hand side of the tape;
            </li>
            <li>
              ...and, finally, the simulated machine's encoded tape, stretching
              off to infinity. Each cell is delimited with a comma (or, for the
              cell the simulated machine's head is pointing at, <code>^</code>).
            </li>
          </ul>
          The UTM, simulating the machine, spends almost all of its time going
          through the rule list trying to find which one matches the simulated
          machine's current [state+symbol], going back and forth between [the
          rule it's currently checking] and [the simulated state/head sections],
          comparing one bit at a time.
        </p>

        <p style={{ marginBottom: "16px", lineHeight: "1.6" }}>
          I find it pleasantly mesmerizing to watch.
        </p>

        <hr style={{ margin: "3em 0" }} />

        <p style={{ marginBottom: "16px", lineHeight: "1.6" }}>
          Anyway, here's a UTM simulating that one:
        </p>

        <div style={{ margin: "0 10%" }}>
          <div style={{ fontSize: "0.3em" }}>
            <TuringMachineViewer
              init={utm2Snapshot}
              onStateChange={onUtm2StateChange}
              initialFps={10000000}
              stateDescriptions={utmSpec.stateDescriptions}
            />
          </div>

          {decodedFromL2 && (
            <>
              <p
                style={{
                  textAlign: "left",
                  marginBottom: "8px",
                  marginTop: "16px",
                  lineHeight: "1.6",
                }}
              >
                Decoded (middle UTM):
              </p>
              <TapeView
                tm={decodedFromL2.l1}
                stateDescriptions={utmSpec.stateDescriptions}
              />
              <p
                style={{
                  textAlign: "left",
                  marginBottom: "8px",
                  marginTop: "8px",
                  lineHeight: "1.6",
                }}
              >
                Decoded (bit-flipper):
              </p>
              <TapeView
                tm={decodedFromL2.l0}
                stateDescriptions={flipBitsSpec.stateDescriptions}
              />
            </>
          )}
        </div>

        <p style={{ marginBottom: "16px", lineHeight: "1.6" }}>
          Patience is a virtue.
        </p>

        <hr style={{ margin: "3em 0" }} />

        <p style={{ marginBottom: "16px", lineHeight: "1.6" }}>
          And, you know, there's no reason this ever needs to <i>stop</i>. We
          could construct an (infinitely long, lazily initialized) tape that
          describes a UTM simulating a UTM simulating a UTM simulating itself
          simulating...
        </p>

        <p style={{ marginBottom: "16px", lineHeight: "1.6" }}>
          That simulation is screaming along right now on some cloud machine.
          The fruits of its labor are being streamed to you now!
        </p>

        <div style={{ textAlign: "center" }}>
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
            Show me the fruits!
          </button>
        </div>
      </div>
    </div>
  );
}
