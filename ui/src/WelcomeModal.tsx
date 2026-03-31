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
      const l0 = l1 && decodeFromUtm(flipBitsSpec.spec, l1);
      if (l0) setDecodedFromL2((l) => ({ ...l, l0 }));
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
        <h1 style={{ margin: "0 0 1em", textAlign: "center" }}>
          A brief explanation
        </h1>

        <p style={{ marginBottom: "16px", lineHeight: "1.6" }}>
          Here's a simple{" "}
          <a href="https://en.wikipedia.org/wiki/Turing_machine">
            Turing machine
          </a>{" "}
          , which takes a tape of 0/1s, flips them all, and then halts:
        </p>

        <div style={{ margin: "0 10%" }}>
          <TuringMachineViewer
            init={snapshot}
            initialFps={5}
            stateDescriptions={flipBitsSpec.stateDescriptions}
            graph={flipBitsSpec.graph}
          />
          <p className="aside">
            (The red cell is where the machine's head is.)
          </p>
        </div>

        <hr style={{ margin: "3em 0" }} />

        <p style={{ marginBottom: "16px", lineHeight: "1.6" }}>
          You may recall that a Turing machine is "universal" if it's capable of
          simulating arbitrary other Turing machines. (Precisely: if there
          exists some encoding scheme mapping (other TM, tape for other
          TM)→(tape for the UTM) such that the UTM "accepts" if-and-only-if the
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
            graph={utmSpec.graph}
          />
          <p className="aside">
            (The green squares have no mechanical significance; they just call
            out points of interest.)
          </p>

          <details className="aside">
            <summary>Nitty-gritty details for the curious</summary>

            <ul>
              <li>
                We encode all the simulated machine's states/symbols into binary
                strings.
              </li>
              <li>
                The UTM's tape layout is:{" "}
                <code>$ # ACCEPTSTATES # BLANK # RULES # STATE # TAPE</code>
                <ul>
                  <li>
                    <code>ACCEPTSTATES</code> is a list of the simulated
                    machine's "accepting" states. When the UTM discovers that
                    the simulated machine has halted, it checks whether the
                    simulated machine's current state is in the list of
                    accepting states, and accepts/rejects based on that.
                  </li>
                  <li>
                    <code>BLANK</code> is the simulated machine's "blank"
                    symbol. Classically, a TM's tape is infinite and covered in
                    some blank symbol. When the simulated machine's head moves
                    farther off the end of the simulated tape, the UTM needs to
                    know what pattern to fill in the simulated cell with.
                  </li>
                  <li>
                    <code>RULES</code> is a list of the simulated machine's
                    state transitions. Each one is of the form{" "}
                    <code>.STATE|SYMBOL|NEWSTATE|NEWSYMBOL|DIR;</code>.
                  </li>
                  <li>
                    <code>STATE</code> is the simulated machine's current state.
                  </li>
                  <li>
                    <code>TAPE</code> is the simulated machine's tape. Each cell
                    begins with a <code>,</code>, or <code>^</code> to indicate
                    the cell the simulated machine's head is pointing at.
                  </li>
                </ul>
              </li>
              <li>
                The UTM's core loop looks like:
                <ul>
                  <li>
                    For each rule, starting from the right-hand end of the RULES
                    section:
                  </li>
                  <li>
                    Mark it with a <code>*</code> to make it easy to get back
                    to.
                  </li>
                  <li>
                    Compare the rule's <code>STATE</code> to the simulated
                    machine's <code>STATE</code>. (Comparisons happen one bit at
                    a time: translate 0/1 to X/Y, one bit at a time. On any
                    mismatch, clean up the X/Y transformation and abort.) On a
                    mismatch, this rule doesn't apply; move on to the next one.
                  </li>
                  <li>
                    Compare the rule's <code>SYMBOL</code> to the contents of
                    the TAPE cell marked <code>^</code> (where the simulated
                    head is). On a mismatch, this rule doesn't apply; move on to
                    the next one.
                  </li>
                  <li>
                    Copy the rule's <code>NEWSTATE</code> to the simulated
                    machine's <code>STATE</code>. (Copies happen one bit at a
                    time, much like comparisons.)
                  </li>
                  <li>
                    Copy the rule's <code>NEWSYMBOL</code> to the contents of
                    the TAPE cell marked <code>^</code> (where the simulated
                    head is).
                  </li>
                  <li>
                    Move the simulated head to the previous/next comma, as
                    prescribed by the rule's DIR.
                  </li>
                </ul>
              </li>
            </ul>
          </details>

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
          You can kinda see how it works: it represents the simulated machine's
          state/symbols as binary; it tracks the simulated machine's state (the{" "}
          <code>#0#</code>); and it tracks the simulated machine's head (the{" "}
          <code>^</code>). It has a description of all the simulated machine's
          transition rules e.g. <code>.0|10|0|01|R;</code>, and it goes through
          them one by one to see which is applicable to the current
          state/symbol, then copies over the applied rule's new state/symbol and
          moves the machine's simulated head (marked <code>^</code>).
        </p>

        <p style={{ marginBottom: "16px", lineHeight: "1.6" }}>
          I find it pleasantly mesmerizing to watch.
        </p>

        <hr style={{ margin: "3em 0" }} />

        <p style={{ marginBottom: "16px", lineHeight: "1.6" }}>
          And a UTM is, of course, itself a Turing machine, so it can simulate
          itself too:
        </p>

        <div style={{ margin: "0 10% 16px" }}>
          <div style={{ fontSize: "0.3em" }}>
            <TuringMachineViewer
              init={utm2Snapshot}
              onStateChange={onUtm2StateChange}
              initialFps={10000000}
              stateDescriptions={utmSpec.stateDescriptions}
              graph={utmSpec.graph}
            />
          </div>

          <details className="aside">
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
