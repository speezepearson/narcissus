import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  copySnapshot,
  getStatus,
  makeInitSnapshot,
  step,
  type TapeOverlay,
  type TuringMachineSnapshot,
  type TuringMachineSpec,
} from "./types";

function useTuringMachine<State extends string, Symbol extends string>(
  spec: TuringMachineSpec<State, Symbol>,
  initialTape: TapeOverlay<Symbol>,
) {
  const [snapshot, setSnapshot] = useState(() => {
    const s = makeInitSnapshot(spec, initialTape);
    return s;
  });
  const [status, setStatus] = useState<"accept" | "reject" | "running">(
    "running",
  );
  const [playing, setPlaying] = useState(false);
  const [logFps, setLogFps] = useState(Math.log10(5));
  const fps = Math.round(10 ** logFps);

  const snapshotRef = useRef(snapshot);
  const statusRef = useRef(status);

  useEffect(() => {
    snapshotRef.current = snapshot;
  }, [snapshot]);

  useEffect(() => {
    statusRef.current = status;
  }, [status]);

  const stepOnce = useCallback((snap: TuringMachineSnapshot<State, Symbol>) => {
    const st = getStatus(step(snap));
    return st;
  }, []);

  const doStep = useCallback(() => {
    if (statusRef.current !== "running") return;
    const next = copySnapshot(snapshotRef.current);
    const st = stepOnce(next);
    snapshotRef.current = next;
    statusRef.current = st;
    setSnapshot(next);
    setStatus(st);
    if (st !== "running") {
      setPlaying(false);
    }
  }, [stepOnce]);

  const reset = useCallback(() => {
    const s = makeInitSnapshot(spec, initialTape.clone());
    setSnapshot(s);
    setStatus("running");
    setPlaying(false);
  }, [spec, initialTape]);

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
    const interval = setInterval(() => {
      if (statusRef.current !== "running") return;
      accumRef.current += fpsRef.current / MAX_RENDER_FPS;
      const stepsThisFrame = Math.floor(accumRef.current);
      accumRef.current -= stepsThisFrame;
      if (stepsThisFrame === 0) return;

      const snap = copySnapshot(snapshotRef.current);
      let st: "accept" | "reject" | "running" = "running";
      for (let i = 0; i < stepsThisFrame; i++) {
        st = stepOnce(snap);
        if (st !== "running") break;
      }
      snapshotRef.current = snap;
      statusRef.current = st;
      setSnapshot(snap);
      setStatus(st);
      if (st !== "running") {
        setPlaying(false);
      }
    }, 1000 / MAX_RENDER_FPS);
    return () => clearInterval(interval);
  }, [playing, stepOnce]);

  return {
    snapshot,
    status,
    playing,
    setPlaying,
    fps,
    logFps,
    setLogFps,
    doStep,
    reset,
  };
}

type TuringMachineViewerProps<State extends string, Symbol extends string> = {
  spec: TuringMachineSpec<State, Symbol>;
  initialTape: TapeOverlay<Symbol>;
};

export function TuringMachineViewer<
  State extends string,
  Symbol extends string,
>({ spec, initialTape }: TuringMachineViewerProps<State, Symbol>) {
  const {
    snapshot,
    status,
    playing,
    setPlaying,
    fps,
    logFps,
    setLogFps,
    doStep,
    reset,
  } = useTuringMachine(spec, initialTape);

  const halted = status !== "running";

  // Build tape display — pad with blanks so head is always visible
  const charOffset = 15;
  const displayTape = useMemo(
    () =>
      Array.from({ length: 2 * charOffset + 1 }, (_, i) => {
        const ind = snapshot.pos + i - charOffset;
        if (ind < 0) return " ";
        return snapshot.tape.get(ind) ?? spec.blank;
      }).join(""),
    [snapshot, spec],
  );
  const moreLeft = snapshot.pos > charOffset;

  const pointerLine =
    " ".repeat(charOffset) + `^ (state=${snapshot.state}, pos=${snapshot.pos})`;

  return (
    <div className="tm-viewer">
      <pre className="tm-tape">
        <code>
          {moreLeft ? "... " : <>&nbsp;&nbsp;&nbsp;&nbsp;</>}
          {displayTape} ...
        </code>
        {"\n"}
        <code>
          <>&nbsp;&nbsp;&nbsp;&nbsp;</>
          {pointerLine}
        </code>
      </pre>

      {halted && (
        <div className={`tm-result tm-result-${status}`}>
          {status.toUpperCase()}
        </div>
      )}

      <div className="tm-controls">
        <button onClick={doStep} disabled={halted}>
          Step
        </button>
        <button onClick={() => setPlaying((p) => !p)} disabled={halted}>
          {playing ? "Pause" : "Play"}
        </button>
        <button onClick={reset}>Reset</button>
        <label className="tm-fps">
          FPS:
          <input
            type="range"
            min={0}
            max={6}
            step={0.1}
            value={logFps}
            onChange={(e) => setLogFps(Number(e.target.value))}
          />
          <span>{fps}</span>
        </label>
      </div>
    </div>
  );
}
