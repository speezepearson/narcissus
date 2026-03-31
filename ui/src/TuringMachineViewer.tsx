import { useState } from "react";
import { LogSlider } from "./LogSlider";
import { TapeView } from "./TapeView";
import { TMStateGraph } from "./TMStateGraph";
import type { GraphSpec } from "./parseSpec";
import { type State, type TuringMachineSnapshot } from "./types";
import { useTuringMachine } from "./useTuringMachine";

type TuringMachineViewerProps = {
  init: TuringMachineSnapshot;
  onStateChange?: (oldState: State, cur: TuringMachineSnapshot) => void;
  initialFps?: number;
  stateDescriptions?: Record<string, string>;
  graph?: GraphSpec;
};

export function TuringMachineViewer({
  init,
  onStateChange,
  initialFps,
  stateDescriptions,
  graph,
}: TuringMachineViewerProps) {
  const { snapshot, status, playPause, doStep, reset } = useTuringMachine(
    init,
    { onStateChange, initialFps },
  );

  const halted = status !== "running";
  const [showGraph, setShowGraph] = useState(false);

  // Current symbol under head (for edge highlighting)
  const currentSymbol = snapshot.tape[snapshot.pos] ?? snapshot.spec.blank;

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
        {graph && (
          <button onClick={() => setShowGraph((v) => !v)}>
            {showGraph ? "Hide Graph" : "Show Graph"}
          </button>
        )}
        <LogSlider
          label="FPS"
          value={playPause.fps}
          onChange={playPause.setFps}
          min={1}
          max={10000000}
        />
      </div>

      <TapeView tm={snapshot} stateDescriptions={stateDescriptions} />

      {showGraph && graph && (
        <TMStateGraph
          graph={graph}
          currentState={snapshot.state}
          currentSymbol={currentSymbol}
        />
      )}
    </div>
  );
}
