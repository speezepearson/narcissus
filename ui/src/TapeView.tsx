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
    <div
      style={{
        background: "var(--code-bg)",
        padding: "8px 12px",
        borderRadius: "6px",
        transition: "height 0.3s ease, min-height 0.3s ease",
        overflow: "hidden",
      }}
    >
      <div
        style={{
          fontSize: "0.8em",
          opacity: 0.7,
          marginBottom: "2px",
          wordBreak: "break-all",
        }}
      >
        {tm.state}
      </div>
      <div
        style={{
          fontFamily: "var(--mono)",
          lineHeight: "1.3",
          overflowWrap: "break-word",
        }}
        dangerouslySetInnerHTML={{ __html: colorizedHtml + " ..." }}
      />
    </div>
  );
}
