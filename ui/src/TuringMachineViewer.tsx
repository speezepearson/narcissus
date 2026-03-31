import { useMemo, useState } from "react";
import { LogSlider } from "./LogSlider";
import { TapeView } from "./TapeView";
import { TMStateGraph } from "./TMStateGraph";
import { buildGraph, type ClusterConfig } from "./buildGraph";
import { type State, type TuringMachineSnapshot } from "./types";
import { useTuringMachine } from "./useTuringMachine";

type TuringMachineViewerProps = {
  init: TuringMachineSnapshot;
  onStateChange?: (oldState: State, cur: TuringMachineSnapshot) => void;
  initialFps?: number;
  stateDescriptions?: Record<string, string>;
  clusterConfig?: ClusterConfig;
};

export function TuringMachineViewer({
  init,
  onStateChange,
  initialFps,
  stateDescriptions,
  clusterConfig,
}: TuringMachineViewerProps) {
  const { snapshot, status, playPause, doStep, reset } = useTuringMachine(
    init,
    { onStateChange, initialFps },
  );

  const halted = status !== "running";
  const [showGraph, setShowGraph] = useState(false);

  // Current symbol under head (for edge highlighting)
  const currentSymbol = snapshot.tape[snapshot.pos] ?? snapshot.spec.blank;

  // Build graph from spec (memoized on spec identity)
  const graph = useMemo(
    () => buildGraph(snapshot.spec, clusterConfig),
    [snapshot.spec, clusterConfig],
  );

  return (
    <div className="tm-viewer">
      <div className="tm-controls">
        <button onClick={doStep} disabled={halted}>
          Step
        </button>
        <button onClick={playPause.toggle} disabled={halted}>
          {playPause.playing ? "Pause" : "Play"}
        </button>
        <button onClick={reset}>Reset</button>
        <button onClick={() => setShowGraph((v) => !v)}>
          {showGraph ? "Hide Graph" : "Show Graph"}
        </button>
        <LogSlider
          label="FPS"
          value={playPause.fps}
          onChange={playPause.setFps}
          min={1}
          max={10000000}
        />
      </div>

      <TapeView tm={snapshot} stateDescriptions={stateDescriptions} />

      {showGraph && (
        <TMStateGraph
          graph={graph}
          currentState={snapshot.state}
          currentSymbol={currentSymbol}
        />
      )}
    </div>
  );
}
