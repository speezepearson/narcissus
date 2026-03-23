import { useMemo } from "react";
import { type TuringMachineSnapshot } from "./types";

type TapeViewProps<State extends string, Symbol extends string> = {
  tm: TuringMachineSnapshot<State, Symbol>;
  radius: number;
};

export function TapeView<State extends string, Symbol extends string>({
  tm,
  radius,
}: TapeViewProps<State, Symbol>) {
  // Build tape display — pad with blanks so head is always visible
  const displayTape = useMemo(
    () =>
      Array.from({ length: 2 * radius + 1 }, (_, i) => {
        const ind = tm.pos + i - radius;
        if (ind < 0) return " ";
        return tm.tape.get(ind) ?? tm.spec.blank;
      }).join(""),
    [tm, radius],
  );
  const moreLeft = tm.pos > radius;

  const pointerLine =
    " ".repeat(radius) + `^ (state=${tm.state}, pos=${tm.pos})`;

  return (
    <pre className="tm-tape">
      <code>
        {moreLeft ? "... " : <>&nbsp;&nbsp;&nbsp;&nbsp;</>}
        {displayTape} ...
      </code>
      {"\n"}
      <code>
        <>&nbsp;&nbsp;&nbsp;&nbsp;</>
        {pointerLine}
      </code>
    </pre>
  );
}
