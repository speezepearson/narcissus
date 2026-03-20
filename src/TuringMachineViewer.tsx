import { useCallback, useEffect, useRef, useState } from "react";
import { TapeView } from "./TapeView";
import {
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
  const [snapshot, setSnapshot] = useState(() =>
    makeInitSnapshot(spec, initialTape),
  );
  const [status, setStatus] = useState<"accept" | "reject" | "running">(
    "running",
  );
  const [playing, setPlaying] = useState(false);
  const [logFps, setLogFps] = useState(Math.log10(5));
  const fps = Math.round(10 ** logFps);

  const snapRef = useRef(snapshot);
  const statusRef = useRef(status);

  useEffect(() => {
    snapRef.current = snapshot;
  }, [snapshot]);
  useEffect(() => {
    statusRef.current = status;
  }, [status]);

  const publish = useCallback((snap: TuringMachineSnapshot<State, Symbol>) => {
    const st = getStatus(snap);
    snapRef.current = snap;
    statusRef.current = st;
    setSnapshot({ ...snap });
    setStatus(st);
    if (st !== "running") setPlaying(false);
  }, []);

  const doStep = useCallback(() => {
    if (statusRef.current !== "running") return;
    step(snapRef.current);
    publish(snapRef.current);
  }, [publish]);

  const reset = useCallback(() => {
    const snap = makeInitSnapshot(spec, initialTape.clone());
    publish(snap);
  }, [spec, initialTape, publish]);

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

      const snap = snapRef.current;
      for (let i = 0; i < stepsThisFrame; i++) {
        step(snap);
        if (getStatus(snap) !== "running") break;
      }
      publish(snap);
    }, 1000 / MAX_RENDER_FPS);
    return () => clearInterval(interval);
  }, [playing, publish]);

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

  return (
    <div className="tm-viewer">
      <TapeView tm={snapshot} radius={40} />

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
            max={7}
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
