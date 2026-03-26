import { useMemo } from "react";
import { type TuringMachineSnapshot } from "./types";

type TapeViewProps = {
  tm: TuringMachineSnapshot;
};

export function TapeView({ tm }: TapeViewProps) {
  // Build tape display — pad with blanks so head is always visible
  const displayTape = useMemo(
    () => tm.tape.join(""),
    [tm.tape],
  );
  const pointerLine =
    " ".repeat(tm.pos) + `^ ${tm.state}`;

  return (
    <pre className="tm-tape">
      <code>
        {displayTape} ...
      </code>
      {"\n"}
      <code>
        {pointerLine}
      </code>
    </pre>
  );
}
