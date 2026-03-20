import { useCallback, useEffect, useRef, useState } from "react";
import { LogSlider } from "./LogSlider";
import { MyUtmSnapshot, myUtmSpec } from "./my-utm-spec";
import { TapeView } from "./TapeView";
import {
  type TuringMachineSnapshot,
  copySnapshot,
  getStatus,
  step,
} from "./types";
import { usePlayPause } from "./usePlayPause";

type MyUTMViewerProps<SimState extends string, SimSymbol extends string> = {
  initialSim: TuringMachineSnapshot<SimState, SimSymbol>;
  optimizationHints?: Array<[SimState, SimSymbol]>;
};

export function MyUTMViewer<SimState extends string, SimSymbol extends string>({
  initialSim,
  optimizationHints,
}: MyUTMViewerProps<SimState, SimSymbol>) {
  const makeInitial = useCallback(() => {
    const utmSnapshot = myUtmSpec.encode(initialSim, {
      optimizationHints,
    }) as MyUtmSnapshot<SimState, SimSymbol>;
    if (!(utmSnapshot instanceof MyUtmSnapshot)) {
      throw new Error("utmSnapshot is not a MyUtmSnapshot???");
    }
    const decoded = utmSnapshot.decode({ sparse: false });
    return { utmSnapshot, decoded };
  }, [initialSim, optimizationHints]);

  const [utmSnapshot, setUtmSnapshot] = useState(
    () => makeInitial().utmSnapshot,
  );
  const [utmStatus, setUtmStatus] = useState<"accept" | "reject" | "running">(
    "running",
  );
  const [lastDecoded, setLastDecoded] = useState<TuringMachineSnapshot<
    SimState,
    SimSymbol
  > | null>(() => makeInitial().decoded ?? null);
  const [visibleRadius, setVisibleRadius] = useState(25);
  const [stepCount, setStepCount] = useState(0);

  const utmRef = useRef(utmSnapshot);
  const statusRef = useRef(utmStatus);
  const lastDecodedRef = useRef(lastDecoded);
  const stepCountRef = useRef(0);

  useEffect(() => {
    utmRef.current = utmSnapshot;
  }, [utmSnapshot]);
  useEffect(() => {
    statusRef.current = utmStatus;
  }, [utmStatus]);
  useEffect(() => {
    lastDecodedRef.current = lastDecoded;
  }, [lastDecoded]);
  useEffect(() => {
    stepCountRef.current = stepCount;
  }, [stepCount]);

  const MAX_HISTORY = 20;
  const historyRef = useRef<
    {
      snap: MyUtmSnapshot<SimState, SimSymbol>;
      decoded: TuringMachineSnapshot<SimState, SimSymbol> | null;
      stepCount: number;
    }[]
  >([]);
  const [canRewind, setCanRewind] = useState(false);

  const pushHistory = useCallback(() => {
    const h = historyRef.current;
    h.push({
      snap: new MyUtmSnapshot(utmRef.current),
      decoded: lastDecodedRef.current
        ? copySnapshot(lastDecodedRef.current)
        : null,
      stepCount: stepCountRef.current,
    });
    if (h.length > MAX_HISTORY) h.shift();
    setCanRewind(true);
  }, []);

  const publishUtm = useCallback(
    (snap: MyUtmSnapshot<SimState, SimSymbol>, steps: number) => {
      const st = getStatus(snap);
      utmRef.current = snap;
      statusRef.current = st;
      setUtmSnapshot(snap);
      setUtmStatus(st);
      setStepCount((c) => c + steps);
      const decoded = snap.decode({ sparse: false });
      if (decoded) {
        lastDecodedRef.current = decoded;
        setLastDecoded(decoded);
      }
    },
    [],
  );

  // Single step
  const doStep = useCallback(() => {
    if (statusRef.current !== "running") return;
    pushHistory();
    const next = utmRef.current;
    step(next);
    publishUtm(next, 1);
  }, [pushHistory, publishUtm]);

  // Step until UTM state changes
  const doStepState = useCallback(() => {
    if (statusRef.current !== "running") return;
    pushHistory();
    const snap = new MyUtmSnapshot(utmRef.current);
    const startState = snap.state;

    let steps = 0;
    while (getStatus(snap) === "running") {
      step(snap);
      steps++;
      if (snap.state !== startState) break;
    }

    publishUtm(snap, steps);
  }, [pushHistory, publishUtm]);

  const reset = useCallback(() => {
    const { utmSnapshot: snap, decoded } = makeInitial();
    utmRef.current = snap;
    statusRef.current = "running";
    lastDecodedRef.current = decoded ?? null;
    setUtmSnapshot(snap);
    setUtmStatus("running");
    setLastDecoded(decoded ?? null);
    setStepCount(0);
    historyRef.current = [];
    setCanRewind(false);
  }, [makeInitial]);

  const rewind = useCallback(() => {
    const h = historyRef.current;
    const entry = h.pop();
    if (!entry) return;
    utmRef.current = entry.snap;
    statusRef.current = "running";
    lastDecodedRef.current = entry.decoded;
    setUtmSnapshot(entry.snap);
    setUtmStatus("running");
    setLastDecoded(entry.decoded);
    setStepCount(entry.stepCount);
    setCanRewind(h.length > 0);
  }, []);

  const onSteps = useCallback(
    (count: number) => {
      if (statusRef.current !== "running") return false;
      pushHistory();
      const snap = new MyUtmSnapshot(utmRef.current);
      for (let i = 0; i < count; i++) {
        stepCountRef.current++;
        if (getStatus(step(snap)) !== "running") break;
      }
      const st = getStatus(snap);
      utmRef.current = snap;
      statusRef.current = st;
      setUtmSnapshot(snap);
      setUtmStatus(st);
      setStepCount(stepCountRef.current);
      const decoded = snap.decode({ sparse: false });
      if (decoded) {
        lastDecodedRef.current = decoded;
        setLastDecoded(decoded);
      }
      return st === "running";
    },
    [pushHistory],
  );

  const { playing, toggle, fps, setFps } = usePlayPause({ onSteps });

  const halted = utmStatus !== "running";

  return (
    <div className="tm-viewer">
      <div className="utm-status-line">
        UTM step: {stepCount} | UTM state: {utmSnapshot.state} | head:{" "}
        {utmSnapshot.pos}
      </div>

      <TapeView tm={utmSnapshot} radius={visibleRadius} />

      {lastDecoded && <TapeView tm={lastDecoded} radius={visibleRadius} />}

      {halted && (
        <div className={`tm-result tm-result-${utmStatus}`}>
          UTM {utmStatus.toUpperCase()}
        </div>
      )}

      <div className="tm-controls">
        <button onClick={doStep} disabled={halted}>
          Step
        </button>
        <button onClick={doStepState} disabled={halted}>
          Step State
        </button>
        <button onClick={toggle} disabled={halted}>
          {playing ? "Pause" : "Play"}
        </button>
        <button onClick={rewind} disabled={!canRewind}>
          Rewind
        </button>
        <button onClick={reset}>Reset</button>
        <LogSlider
          label="FPS"
          value={fps}
          onChange={setFps}
          min={1}
          max={10000000}
        />
        <LogSlider
          label="Radius"
          value={visibleRadius}
          onChange={setVisibleRadius}
          min={1}
          max={1000}
        />
      </div>
    </div>
  );
}
