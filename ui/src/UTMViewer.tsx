import { useCallback, useState } from "react";
import { LogSlider } from "./LogSlider";
import {
  MyUtmSnapshot,
  type MyUtmState,
  type MyUtmSymbol,
} from "./my-utm-spec";
import { TapeView } from "./TapeView";
import { type TuringMachineSnapshot } from "./types";
import { useTuringMachine } from "./useTuringMachine";
import { must } from "./util";

type UTMViewerProps<SimState extends string, SimSymbol extends string> = {
  init: MyUtmSnapshot<SimState, SimSymbol>;
};

export function UTMViewer<SimState extends string, SimSymbol extends string>({
  init,
}: UTMViewerProps<SimState, SimSymbol>) {
  const [lastDecoded, setLastDecoded] = useState<
    TuringMachineSnapshot<SimState, SimSymbol>
  >(() => must(init.decode({ sparse: false })));

  const onStateChange = useCallback(
    (
      _oldState: MyUtmState,
      cur: TuringMachineSnapshot<MyUtmState, MyUtmSymbol>,
    ) => {
      if (cur.state !== "init") return;
      const utm = new MyUtmSnapshot({ simSpec: init.simSpec, ...cur });
      setLastDecoded(must(utm.decode()));
    },
    [init],
  );

  const { snapshot, status, playPause, doStep, reset } = useTuringMachine(
    init,
    { onStateChange },
  );

  const halted = status !== "running";

  return (
    <div className="tm-viewer">
      <TapeView tm={snapshot} radius={40} />
      <TapeView tm={lastDecoded} radius={40} />

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
