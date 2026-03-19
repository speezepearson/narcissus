import { useCallback, useEffect, useRef, useState } from "react";
import {
  type TuringMachineSpec,
  type TuringMachineSnapshot,
  copySnapshot,
  makeInitSnapshot,
  step,
  getStatus,
} from "./types";

function padTape<State extends string, Symbol extends string>(
  snapshot: TuringMachineSnapshot<State, Symbol>,
  blank: Symbol,
) {
  while (snapshot.tape.length <= snapshot.pos) {
    snapshot.tape.push(blank);
  }
}

function useTuringMachine<State extends string, Symbol extends string>(
  spec: TuringMachineSpec<State, Symbol>,
  initialTape: readonly Symbol[],
) {
  const [snapshot, setSnapshot] = useState(() => {
    const s = makeInitSnapshot(spec, initialTape);
    padTape(s, spec.blank);
    return s;
  });
  const [status, setStatus] = useState<"accept" | "reject" | "running">("running");
  const [playing, setPlaying] = useState(false);
  const [fps, setFps] = useState(5);

  const snapshotRef = useRef(snapshot);
  const statusRef = useRef(status);

  useEffect(() => {
    snapshotRef.current = snapshot;
  }, [snapshot]);

  useEffect(() => {
    statusRef.current = status;
  }, [status]);

  const doStep = useCallback(() => {
    if (statusRef.current !== "running") return;
    const next = copySnapshot(snapshotRef.current);
    padTape(next, spec.blank);
    const status = getStatus(step(next));
    padTape(next, spec.blank);
    snapshotRef.current = next;
    statusRef.current = status;
    setSnapshot(next);
    setStatus(status);
    if (status !== "running") {
      setPlaying(false);
    }
  }, [spec.blank]);

  const reset = useCallback(() => {
    const s = makeInitSnapshot(spec, initialTape);
    padTape(s, spec.blank);
    setSnapshot(s);
    setStatus("running");
    setPlaying(false);
  }, [spec, initialTape]);

  useEffect(() => {
    if (!playing) return;
    const interval = setInterval(doStep, 1000 / fps);
    return () => clearInterval(interval);
  }, [playing, fps, doStep]);

  return { snapshot, status, playing, setPlaying, fps, setFps, doStep, reset };
}

type TuringMachineViewerProps<State extends string, Symbol extends string> = {
  spec: TuringMachineSpec<State, Symbol>;
  initialTape: readonly Symbol[];
};

export function TuringMachineViewer<
  State extends string,
  Symbol extends string,
>({ spec, initialTape }: TuringMachineViewerProps<State, Symbol>) {
  const { snapshot, status, playing, setPlaying, fps, setFps, doStep, reset } =
    useTuringMachine(spec, initialTape);

  const halted = status !== "running";

  // Build tape display — pad with blanks so head is always visible
  const tape = snapshot.tape;
  const displayCells =
    tape.length > snapshot.pos
      ? tape
      : [
          ...tape,
          ...Array.from<string>({
            length: snapshot.pos - tape.length + 1,
          }).fill(snapshot.spec.blank),
        ];
  const displayTape = displayCells.join("");

  // Compute character offset of the head position
  const charOffset = displayCells.slice(0, snapshot.pos).join("").length;
  const pointerLine = " ".repeat(charOffset) + `^ (state=${snapshot.state})`;

  return (
    <div className="tm-viewer">
      <pre className="tm-tape">
        <code>{displayTape}</code>
        {"\n"}
        <code>{pointerLine}</code>
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
            min={1}
            max={60}
            value={fps}
            onChange={(e) => setFps(Number(e.target.value))}
          />
          <span>{fps}</span>
        </label>
      </div>
    </div>
  );
}
