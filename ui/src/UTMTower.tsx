import { useCallback, useState } from "react";
import { infiniteUtmTapeBackground } from "./infinite-utm";
import { LogSlider } from "./LogSlider";
import {
  MyUtmSnapshot,
  myUtmSpec,
  type MyUtmState,
  type MyUtmSymbol,
} from "./my-utm-spec";
import { TapeView } from "./TapeView";
import { makeSimpleTapeOverlay, type TuringMachineSnapshot } from "./types";
import { useTuringMachine } from "./useTuringMachine";
import { must } from "./util";

type RecursiveUtmSnapshot = MyUtmSnapshot<MyUtmState, MyUtmSymbol>;

function makeRecursiveUtmSnapshot(
  tm?: TuringMachineSnapshot<MyUtmState, MyUtmSymbol>,
): RecursiveUtmSnapshot {
  return new MyUtmSnapshot({
    simSpec: myUtmSpec,
    ...(tm ?? {
      pos: 0,
      spec: myUtmSpec,
      state: myUtmSpec.initial,
      tape: makeSimpleTapeOverlay(infiniteUtmTapeBackground),
    }),
  });
}

export function UTMTower() {
  const [highestTickedFloor, setHighestTickedFloor] = useState(0);
  const [upperFloors, setUpperFloors] = useState<RecursiveUtmSnapshot[]>([]);
  const recomputeUpperFloors = useCallback(
    (base: TuringMachineSnapshot<MyUtmState, MyUtmSymbol>) => {
      const newUpperFloors = [
        makeRecursiveUtmSnapshot(must(makeRecursiveUtmSnapshot(base).decode())),
      ];
      for (let i = 0; i < highestTickedFloor; i++) {
        newUpperFloors.push(
          must(makeRecursiveUtmSnapshot(newUpperFloors[i].decode())),
        );
      }
      for (let i = 0; i < upperFloors.length; i++) {
        if (
          upperFloors[i].state !== "init" &&
          newUpperFloors[i].state === "init"
        ) {
          setHighestTickedFloor(i);
        }
      }
      setUpperFloors(newUpperFloors);
    },
    [highestTickedFloor, upperFloors],
  );
  const {
    snapshot,
    status,
    playPause,
    doStep,
    reset: resetUtm,
  } = useTuringMachine(makeRecursiveUtmSnapshot(), {
    onStateChange(_oldState, cur) {
      if (cur.state !== "init") return;
      recomputeUpperFloors(cur);
    },
  });

  const halted = status !== "running";

  return (
    <div className="tm-viewer">
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
        <button onClick={resetUtm}>Reset</button>
        <LogSlider
          label="FPS"
          value={playPause.fps}
          onChange={playPause.setFps}
          min={1}
          max={100000000}
        />
      </div>

      <TapeView tm={snapshot} radius={40} />
      {upperFloors.map((tm, i) => (
        <TapeView key={i} tm={tm ?? makeRecursiveUtmSnapshot()} radius={40} />
      ))}
      <TapeView tm={makeRecursiveUtmSnapshot()} radius={40} />
      <TapeView tm={makeRecursiveUtmSnapshot()} radius={40} />
      <TapeView tm={makeRecursiveUtmSnapshot()} radius={40} />
      <TapeView tm={makeRecursiveUtmSnapshot()} radius={40} />
      <TapeView tm={makeRecursiveUtmSnapshot()} radius={40} />
      <TapeView tm={makeRecursiveUtmSnapshot()} radius={40} />
      <TapeView tm={makeRecursiveUtmSnapshot()} radius={40} />
      <TapeView tm={makeRecursiveUtmSnapshot()} radius={40} />
      <TapeView tm={makeRecursiveUtmSnapshot()} radius={40} />
      <TapeView tm={makeRecursiveUtmSnapshot()} radius={40} />
      <TapeView tm={makeRecursiveUtmSnapshot()} radius={40} />
    </div>
  );
}
