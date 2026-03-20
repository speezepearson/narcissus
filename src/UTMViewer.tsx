import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { compile, compileSnapshot, fastStep, writeBack } from "./fast-run";
import { MyUtmSnapshot, myUtmSpec } from "./my-utm-spec";
import { TapeView } from "./TapeView";
import { makeBreaker } from "./test-util";
import {
  type TuringMachineSnapshot,
  copySnapshot,
  getStatus,
  step,
} from "./types";

type MyUTMViewerProps<SimState extends string, SimSymbol extends string> = {
  initialSim: TuringMachineSnapshot<SimState, SimSymbol>;
  optimizationHints?: Array<[SimState, SimSymbol]>;
};

export function MyUTMViewer<SimState extends string, SimSymbol extends string>({
  initialSim,
  optimizationHints,
}: MyUTMViewerProps<SimState, SimSymbol>) {
  const machine = useMemo(() => compile(myUtmSpec), []);

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
  const [playing, setPlaying] = useState(false);
  const [logFps, setLogFps] = useState(Math.log10(5));
  const fps = Math.round(10 ** logFps);
  const [logRadius, setLogRadius] = useState(1.4);
  const visibleRadius = Math.round(10 ** logRadius);
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

  // Single step — overhead of compile/writeBack not worth it for 1 step
  const doStep = useCallback(() => {
    if (statusRef.current !== "running") return;
    pushHistory();
    const next = utmRef.current;
    const st = getStatus(step(next));

    utmRef.current = next;
    statusRef.current = st;
    setUtmSnapshot(next);
    setUtmStatus(st);
    setStepCount((c) => c + 1);

    const decoded = next.decode({ sparse: false });
    if (decoded) {
      lastDecodedRef.current = decoded;
      setLastDecoded(decoded);
    }

    if (st !== "running") {
      setPlaying(false);
    }
  }, [pushHistory]);

  // Step until UTM state changes — can be many steps, use fast path
  const doStepState = useCallback(() => {
    if (statusRef.current !== "running") return;
    pushHistory();
    const snap = new MyUtmSnapshot(utmRef.current);
    const compiled = compileSnapshot(snap, machine);
    const startState = compiled.state;

    let steps = 0;
    while (true) {
      if (!fastStep(compiled)) break;
      steps++;
      if (compiled.state !== startState) break;
    }

    writeBack(compiled, snap);
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

    if (st !== "running") {
      setPlaying(false);
    }
  }, [machine, pushHistory]);

  const reset = useCallback(() => {
    const { utmSnapshot: snap, decoded } = makeInitial();
    utmRef.current = snap;
    statusRef.current = "running";
    lastDecodedRef.current = decoded ?? null;
    setUtmSnapshot(snap);
    setUtmStatus("running");
    setLastDecoded(decoded ?? null);
    setPlaying(false);
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
    setPlaying(false);
    setCanRewind(h.length > 0);
  }, []);

  const fpsRef = useRef(fps);
  useEffect(() => {
    fpsRef.current = fps;
  }, [fps]);
  const accumRef = useRef(0);

  useEffect(() => {
    if (!playing) {
      accumRef.current = 0;
      return;
    }
    const MAX_RENDER_FPS = 30;
    const interval = setInterval(async () => {
      if (statusRef.current !== "running") return;
      accumRef.current += fpsRef.current / MAX_RENDER_FPS;
      const stepsThisFrame = Math.floor(accumRef.current);
      accumRef.current -= stepsThisFrame;
      if (stepsThisFrame === 0) return;
      pushHistory();
      const snap = new MyUtmSnapshot(utmRef.current);
      // const compiled = compileSnapshot(snap, machine);
      const breaker = makeBreaker();
      for (let i = 0; i < stepsThisFrame; i++) {
        stepCountRef.current++;
        if (getStatus(step(snap)) !== "running") break;
        await breaker();
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
      if (st !== "running") {
        setPlaying(false);
      }
    }, 1000 / MAX_RENDER_FPS);
    return () => clearInterval(interval);
  }, [playing, machine, pushHistory]);

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
        <button onClick={() => setPlaying((p) => !p)} disabled={halted}>
          {playing ? "Pause" : "Play"}
        </button>
        <button onClick={rewind} disabled={!canRewind}>
          Rewind
        </button>
        <button onClick={reset}>Reset</button>
        <label className="tm-fps">
          FPS:
          <input
            type="range"
            min={0}
            max={7}
            step={0.1}
            value={logFps}
            onChange={(e) => setLogFps(Number(e.target.value))}
          />
          <span>{fps}</span>
        </label>
        <label className="tm-fps">
          Radius:
          <input
            type="range"
            min={0}
            max={3}
            step={0.01}
            value={logRadius}
            onChange={(e) => setLogRadius(Number(e.target.value))}
          />
          <span>{visibleRadius}</span>
        </label>
      </div>
    </div>
  );
}
