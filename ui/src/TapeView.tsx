import { useMemo } from "react";
import { type TuringMachineSnapshot } from "./types";
import { colorizeTape } from "./colorizeTape";

type TapeViewProps = {
  tm: TuringMachineSnapshot;
};

export function TapeView({ tm }: TapeViewProps) {
  const colorizedHtml = useMemo(
    () => colorizeTape(tm.tape as string[], tm.pos),
    [tm.tape, tm.pos],
  );

  return (
    <div className="tm-tape">
      <div style={{ fontSize: "0.8em", opacity: 0.7, marginBottom: "2px", wordBreak: "break-all" }}>{tm.state}</div>
      <div  style={{ wordBreak: "break-all" }} dangerouslySetInnerHTML={{ __html: colorizedHtml + " ..." }} />
    </div>
  );
}
