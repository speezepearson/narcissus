import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  compile,
  compileSnapshot,
  decompileSnapshot,
  fastRun,
  type CompiledSnapshot,
} from "./fast-run";
import { TapeView } from "./TapeView";
import {
  makeInitSnapshot,
  type TapeOverlay,
  type TuringMachineSpec,
} from "./types";

function useTuringMachine<State extends string, Symbol extends string>(
  spec: TuringMachineSpec<State, Symbol>,
  initialTape: TapeOverlay<Symbol>,
) {
  const machine = useMemo(() => compile(spec), [spec]);

  const makeInitCompiled = useCallback(
    () => compileSnapshot(makeInitSnapshot(spec, initialTape), machine),
    [spec, initialTape, machine],
  );

  const compiledRef = useRef<CompiledSnapshot>(makeInitCompiled());
  const statusRef = useRef<"accept" | "reject" | "running">("running");

  const [snapshot, setSnapshot] = useState(() =>
    makeInitSnapshot(spec, initialTape),
  );
  const [status, setStatus] = useState<"accept" | "reject" | "running">(
    "running",
  );
  const [playing, setPlaying] = useState(false);
  const [logFps, setLogFps] = useState(Math.log10(5));
  const fps = Math.round(10 ** logFps);

  const publish = useCallback(
    (st: "accept" | "reject" | "running") => {
      setSnapshot(decompileSnapshot(compiledRef.current, spec));
      setStatus(st);
      statusRef.current = st;
      if (st !== "running") setPlaying(false);
    },
    [spec],
  );

  const doStep = useCallback(() => {
    if (statusRef.current !== "running") return;
    const result = fastRun(compiledRef.current, { gas: 1 });
    publish(result.halted ? result.status : "running");
  }, [publish]);

  const reset = useCallback(() => {
    compiledRef.current = compileSnapshot(
      makeInitSnapshot(spec, initialTape.clone()),
      machine,
    );
    publish("running");
  }, [spec, initialTape, machine, publish]);

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

      const result = fastRun(compiledRef.current, { gas: stepsThisFrame });
      publish(result.halted ? result.status : "running");
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
