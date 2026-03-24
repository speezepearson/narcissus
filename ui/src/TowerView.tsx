import { useEffect, useRef, useState } from "react";

const GREEN_SYMS = new Set(["*", "X", "Y", "^", ">"]);

interface TowerDeltaLevel {
  head_pos: number;
  state: string;
  tape_len: number;
  overwritten: [number, string][];
}

interface TowerDelta {
  steps: number;
  guest_steps: number;
  steps_per_sec: number;
  tower: TowerDeltaLevel[];
}

function reconstructTape(
  unblemished: string,
  overwritten: [number, string][],
  end: number,
): string {
  const chars = unblemished.slice(0, end).split("");
  // Pad with '_' if end > unblemished length
  while (chars.length < end) {
    chars.push("_");
  }
  for (const [idx, ch] of overwritten) {
    if (idx < end) {
      chars[idx] = ch;
    }
  }
  return chars.join("");
}

function colorizeTape(tape: string, headPos: number): string {
  let out = "";
  for (let i = 0; i < tape.length; i++) {
    const ch = tape[i];
    const escaped =
      ch === "&" ? "&amp;" : ch === "<" ? "&lt;" : ch === ">" ? "&gt;" : ch;
    if (i === headPos) {
      out += `<span style="background:#f87171">${escaped}</span>`;
    } else if (GREEN_SYMS.has(ch)) {
      out += `<span style="background:#4ade80">${escaped}</span>`;
    } else {
      out += escaped;
    }
  }
  return out;
}

export function TowerView() {
  const [unblemished, setUnblemished] = useState<string | null>(null);
  const [data, setData] = useState<TowerDelta | null>(null);
  const maxHeadPos = useRef<number[]>([]);

  // Fetch unblemished tape once on mount
  useEffect(() => {
    fetch("/api/tape")
      .then((res) => res.text())
      .then(setUnblemished)
      .catch(() => {
        // SWALLOW_EXCEPTION: server may not be ready yet; page reload will retry
      });
  }, []);

  // SSE stream for tower deltas
  useEffect(() => {
    const es = new EventSource("/api/tower");
    es.onmessage = (event) => {
      const json: TowerDelta = JSON.parse(event.data);
      // Track max head_pos per level
      while (maxHeadPos.current.length < json.tower.length) {
        maxHeadPos.current.push(0);
      }
      for (let i = 0; i < json.tower.length; i++) {
        const hp = json.tower[i].head_pos;
        if (hp > maxHeadPos.current[i]) {
          maxHeadPos.current[i] = hp;
        }
      }
      setData(json);
    };
    return () => es.close();
  }, []);

  if (!unblemished || !data) {
    return <div style={{ padding: "16px" }}>Loading...</div>;
  }

  return (
    <div style={{ textAlign: "left", padding: "16px" }}>
      <h2 style={{ marginBottom: "8px" }}>
        Tower &mdash; {data.steps.toLocaleString()} steps
        {data.steps_per_sec > 0 && (
          <span
            style={{
              fontWeight: "normal",
              fontSize: "14px",
              marginLeft: "12px",
            }}
          >
            ({data.steps_per_sec.toFixed(1)}M steps/s,{" "}
            {data.guest_steps.toLocaleString()} guest steps)
          </span>
        )}
      </h2>
      <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
        {data.tower.map((level, i) => {
          const mhp = maxHeadPos.current[i] ?? level.head_pos;
          const end = Math.max(mhp, level.head_pos) + 10;
          const tape = reconstructTape(unblemished, level.overwritten, end);
          return (
            <div
              key={i}
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
                  fontSize: "11px",
                  color: "#888",
                  marginBottom: "4px",
                }}
              >
                L{i} &middot; {level.state} &middot;{" "}
                {level.tape_len.toLocaleString()} symbols
              </div>
              <div
                style={{
                  fontFamily: "var(--mono)",
                  fontSize: "12px",
                  lineHeight: "1.3",
                  overflowWrap: "break-word",
                }}
                dangerouslySetInnerHTML={{
                  __html: colorizeTape(tape, level.head_pos),
                }}
              />
            </div>
          );
        })}
      </div>
    </div>
  );
}
