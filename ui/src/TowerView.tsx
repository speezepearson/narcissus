import { useEffect, useState } from "react";

const GREEN_SYMS = new Set(["*", "X", "Y", "^", ">"]);

interface TowerLevelData {
  tape: string;
  head_pos: number;
  state: string;
  tape_len: number;
}

interface TowerData {
  steps: number;
  guest_steps: number;
  steps_per_sec: number;
  tower: TowerLevelData[];
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
  const [data, setData] = useState<TowerData | null>(null);

  useEffect(() => {
    let active = true;
    const poll = async () => {
      while (active) {
        try {
          const res = await fetch("/api/tower");
          if (res.ok) {
            const json: TowerData = await res.json();
            setData(json);
          }
        } catch {
          // SWALLOW_EXCEPTION: server may not be ready yet; we'll retry
        }
        await new Promise((r) => setTimeout(r, 100));
      }
    };
    poll();
    return () => {
      active = false;
    };
  }, []);

  if (!data) {
    return <div style={{ padding: "16px" }}>Loading...</div>;
  }

  return (
    <div style={{ textAlign: "left", padding: "16px" }}>
      <h2 style={{ marginBottom: "8px" }}>
        Tower &mdash; {data.steps.toLocaleString()} steps
        {data.steps_per_sec > 0 && (
          <span style={{ fontWeight: "normal", fontSize: "14px", marginLeft: "12px" }}>
            ({data.steps_per_sec.toFixed(1)}M steps/s, {data.guest_steps.toLocaleString()} guest steps)
          </span>
        )}
      </h2>
      <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
        {data.tower.map((level, i) => (
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
              L{i} &middot; {level.state} &middot; {level.tape_len.toLocaleString()} symbols
            </div>
            <div
              style={{
                fontFamily: "var(--mono)",
                fontSize: "12px",
                lineHeight: "1.3",
                overflowWrap: "break-word",
              }}
              dangerouslySetInnerHTML={{
                __html: colorizeTape(level.tape, level.head_pos),
              }}
            />
          </div>
        ))}
      </div>
    </div>
  );
}
