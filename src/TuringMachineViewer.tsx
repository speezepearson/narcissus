import { useCallback, useEffect, useRef, useState } from "react";
import { LogSlider } from "./LogSlider";
import { TapeView } from "./TapeView";
import {
  getStatus,
  makeInitSnapshot,
  step,
  type TapeOverlay,
  type TuringMachineSnapshot,
  type TuringMachineSpec,
} from "./types";
import { usePlayPause } from "./usePlayPause";

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

  const onSteps = useCallback(
    (count: number) => {
      if (statusRef.current !== "running") return false;
      const snap = snapRef.current;
      for (let i = 0; i < count; i++) {
        step(snap);
        if (getStatus(snap) !== "running") break;
      }
      publish(snap);
      return getStatus(snap) === "running";
    },
    [publish],
  );

  const playPause = usePlayPause({ onSteps });

  return {
    snapshot,
    status,
    doStep,
    reset,
    ...playPause,
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
  const { snapshot, status, playing, toggle, fps, setFps, doStep, reset } =
    useTuringMachine(spec, initialTape);

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
        <button onClick={toggle} disabled={halted}>
          {playing ? "Pause" : "Play"}
        </button>
        <button onClick={reset}>Reset</button>
        <LogSlider
          label="FPS"
          value={fps}
          onChange={setFps}
          min={1}
          max={10000000}
        />
      </div>
    </div>
  );
}
