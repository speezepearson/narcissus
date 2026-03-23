import { LogSlider } from "./LogSlider";
import { TapeView } from "./TapeView";
import { type TuringMachineSnapshot } from "./types";
import { useTuringMachine } from "./useTuringMachine";

type TuringMachineViewerProps<State extends string, Symbol extends string> = {
  init: TuringMachineSnapshot<State, Symbol>;
};

export function TuringMachineViewer<
  State extends string,
  Symbol extends string,
>({ init }: TuringMachineViewerProps<State, Symbol>) {
  const { snapshot, status, playPause, doStep, reset } = useTuringMachine(init);

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
    </div>
  );
}
