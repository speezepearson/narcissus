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
import { must } from "./util";

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
      makeInitSnapshot(flipBitsSpec.spec, welcomeModalExample.bitFlipperInput),
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
    () => must(decodeFromUtm(flipBitsSpec.spec, utm1Snapshot)),
    [utm1Snapshot],
  );

  const [decodedFromL1, setDecodedFromL1] =
    useState<TuringMachineSnapshot | null>(null);

  useEffect(() => {
    setDecodedFromL1(initialDecodedL1);
  }, [initialDecodedL1]);

  const onUtm1StateChange = useCallback(
    (_oldState: State, cur: TuringMachineSnapshot) => {
      const decoded = decodeFromUtm(flipBitsSpec.spec, cur);
      if (decoded) {
        setDecodedFromL1(decoded);
      }
    },
    [],
  );

  // Decoded L2 -> L1 UTM guest, then L1 -> flip-bits guest
  const initialDecodedL2 = useMemo(() => {
    const l1 = must(decodeFromUtm(utmSpec.spec, utm2Snapshot));
    const l0 = must(decodeFromUtm(flipBitsSpec.spec, l1));
    return { l1, l0 };
  }, [utm2Snapshot]);

  const [decodedFromL2, setDecodedFromL2] = useState<{
    l1: TuringMachineSnapshot;
    l0: TuringMachineSnapshot;
  }>(initialDecodedL2);

  useEffect(() => {
    setDecodedFromL2(initialDecodedL2);
  }, [initialDecodedL2]);

  const onUtm2StateChange = useCallback(
    (_oldState: State, cur: TuringMachineSnapshot) => {
      const l1 = decodeFromUtm(utmSpec.spec, cur);
      if (l1) setDecodedFromL2((l) => ({ ...l, l1 }));
      const l2 = l1 && decodeFromUtm(flipBitsSpec.spec, l1);
      if (l2) setDecodedFromL2((l) => ({ ...l, l2 }));
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
          is, right?), which takes a tape of 0/1s and flips them all:
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
          You may recall that a Turing machine is "universal" if it's capable of
          simulating arbitrary other Turing machines. (Precisely: if there
          exists some encoding scheme mapping (other TM, tape for other TM) to
          (tape for the UTM) such that the UTM "accepts" if-and-only-if the
          simulated TM would accept.)
        </p>
        <p style={{ marginBottom: "16px", lineHeight: "1.6" }}>
          Here's a universal Turing machine simulating the same bit-flipping
          machine we were looking at before:
        </p>

        <div style={{ margin: "0 10% 16px" }}>
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
              It encodes all the simulated machine's states/symbols into binary
              strings.
            </li>
            <li>
              It has a description of all the simulated machine's state
              transitions: <code>.0|10|0|01|R;</code> means "if you're in state
              0, and you see symbol 10, then stay in state 0, and write symbol
              01, and move right."
            </li>
            <li>
              Also delimited by <code>#</code>s, there's: the list of accepting
              states (here, just state <code>1</code>); the machine's "blank"
              symbol that should be used to fill in the right-hand side of the
              tape (here, symbol <code>00</code>); the machine's current state
              (it starts in state <code>0</code>);
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
          comparing one bit at a time. When if finds a matching rule, it copies
          the rule's new state into the state register (one bit at a time, as
          always), and the new symbol into the cell the simulated head is
          pointed at.
        </p>

        <p style={{ marginBottom: "16px", lineHeight: "1.6" }}>
          I find it pleasantly mesmerizing to watch.
        </p>

        <hr style={{ margin: "3em 0" }} />

        <p style={{ marginBottom: "16px", lineHeight: "1.6" }}>
          Anyway, here's <i>another</i> UTM simulating <i>that</i> one:
        </p>

        <div style={{ margin: "0 10% 16px" }}>
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

          <details>
            <summary>Nitty-gritty details for the curious</summary>

            <p style={{ marginBottom: "16px", lineHeight: "1.6" }}>
              If you look closely at this UTM's looooong rules section, you'll
              notice not all the rules have the same format.{" "}
              <code>.10100001|0010|10100001|0000|R;</code>
              should look familiar, the same format as the bit-flipper rules;
              but <code>.01011101,0,1000,1011,1101|R;</code> is new.
            </p>

            <p style={{ marginBottom: "16px", lineHeight: "1.6" }}>
              This is because -- and I cannot believe I am writing this -- I did
              some performance optimization on the UTM.
              <ol>
                <li>
                  <code>.STATE,SYM,SYM,SYM|DIR;</code> means "if you're in state
                  STATE, and you see any of these SYMbols, don't change state,
                  don't overwrite the symbol, just move DIR."
                </li>
                <li>
                  ...and also, those <code>SYM</code>s are actually{" "}
                  <i>prefixes</i> -- a SYM of <code>0</code> means "any symbol
                  whose binary representation starting with 0."
                </li>
              </ol>
              Together, those tricks greatly reduce the number of
              bit-comparisons the UTM needs to do in order to find the
              applicable rule, when the simulated TM is in a state where it's
              scanning through the tape looking for a particular symbol. (Which,
              recall, is how the UTM spends most of its time.)
            </p>
          </details>
        </div>

        <p style={{ marginBottom: "16px", lineHeight: "1.6" }}>
          Patience is a virtue.
        </p>

        <hr style={{ margin: "3em 0" }} />

        <p style={{ marginBottom: "16px", lineHeight: "1.6" }}>
          And, you know, there's no reason this ever needs to <i>stop</i>. We
          could construct an (infinitely long, lazily initialized) tape that
          describes a UTM simulating a UTM simulating a UTM simulating a UTM
          simulating...
        </p>

        <p style={{ marginBottom: "16px", lineHeight: "1.6" }}>
          That simulation is screaming along right now on some cloud machine.
          The fruits of its labor are being streamed to you live!
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
