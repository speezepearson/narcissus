import { LogSlider } from "./LogSlider";
import { TapeView } from "./TapeView";
import { type State, type TuringMachineSnapshot } from "./types";
import { useTuringMachine } from "./useTuringMachine";

type TuringMachineViewerProps = {
  init: TuringMachineSnapshot;
  onStateChange?: (oldState: State, cur: TuringMachineSnapshot) => void;
  initialFps?: number;
};

export function TuringMachineViewer({
  init,
  onStateChange,
  initialFps,
}: TuringMachineViewerProps) {
  const { snapshot, status, playPause, doStep, reset } = useTuringMachine(
    init,
    { onStateChange, initialFps },
  );

  const halted = status !== "running";

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
        <LogSlider
          label="FPS"
          value={playPause.fps}
          onChange={playPause.setFps}
          min={1}
          max={10000000}
        />
      </div>

      <TapeView tm={snapshot} />

      {halted && (
        <div className={`tm-result tm-result-${status}`}>
          {status.toUpperCase()}
        </div>
      )}
    </div>
  );
}
