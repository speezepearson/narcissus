import { useEffect, useRef, useState } from "react";
import { updateTower, type TowerLevel, type UtmMeta } from "./tower";

const GREEN_SYMS = new Set(["*", "X", "Y", "^", ">"]);

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

// ── L0 state from server ──

interface L0State {
  steps: number;
  guestSteps: number;
  stepsPerSec: number;
  state: string;
  headPos: number;
  maxHeadPos: number;
  tape: string;
  tapeLen: number;
}

// ── Main component ──

export function TowerView() {
  const [l0, setL0] = useState<L0State | null>(null);
  const metaRef = useRef<UtmMeta | null>(null);
  const tapeRef = useRef<string>("");
  const unblemishedRef = useRef<string>("");
  const towerRef = useRef<TowerLevel[]>([]);
  const [tower, setTower] = useState<TowerLevel[] | null>(null);

  useEffect(() => {
    const es = new EventSource("/api/tower");
    es.onmessage = (event) => {
      const msg = JSON.parse(event.data);

      if (msg.type === "total") {
        unblemishedRef.current = msg.unblemished;
        tapeRef.current = msg.tape;

        if (msg.utm_states && msg.utm_symbol_chars) {
          metaRef.current = {
            utmStates: msg.utm_states,
            utmSymbolChars: msg.utm_symbol_chars,
          };
        }

        const newL0: L0State = {
          steps: msg.steps,
          guestSteps: msg.guest_steps,
          stepsPerSec: msg.steps_per_sec,
          state: msg.state,
          headPos: msg.head_pos,
          maxHeadPos: msg.max_head_pos,
          tape: msg.tape,
          tapeLen: msg.tape_len,
        };
        setL0(newL0);

        if (metaRef.current) {
          const newL0Level: TowerLevel = {
            state: newL0.state,
            headPos: newL0.headPos,
            tape: newL0.tape,
            tapeLen: newL0.tapeLen,
          };
          // On total, reset tower
          towerRef.current = [];
          updateTower(newL0Level, towerRef.current, metaRef.current);
          setTower([...towerRef.current]);
        }
      } else if (msg.type === "delta") {
        const chars = tapeRef.current.split("");
        const end = Math.max(msg.max_head_pos, msg.head_pos) + 10;
        const ub = unblemishedRef.current;
        while (chars.length < end) {
          const pos = chars.length;
          chars.push(pos < ub.length ? ub[pos] : "_");
        }
        for (const [pos, ch] of msg.new_overwrites as [number, string][]) {
          while (chars.length <= pos) chars.push("_");
          chars[pos] = ch;
        }
        tapeRef.current = chars.join("");

        const newL0: L0State = {
          steps: msg.total_steps,
          guestSteps: msg.guest_steps,
          stepsPerSec: msg.steps_per_sec,
          state: msg.state,
          headPos: msg.head_pos,
          maxHeadPos: msg.max_head_pos,
          tape: tapeRef.current,
          tapeLen: msg.tape_len,
        };
        setL0(newL0);

        if (metaRef.current) {
          const newL0Level: TowerLevel = {
            state: newL0.state,
            headPos: newL0.headPos,
            tape: newL0.tape,
            tapeLen: newL0.tapeLen,
          };
          updateTower(newL0Level, towerRef.current, metaRef.current);
          setTower([...towerRef.current]);
        }
      }
    };
    return () => es.close();
  }, []);

  if (!l0 || !tower) {
    return <div style={{ padding: "16px" }}>Loading...</div>;
  }

  return (
    <div style={{ textAlign: "left", padding: "16px" }}>
      <h2 style={{ marginBottom: "8px" }}>
        Tower &mdash; {l0.steps.toLocaleString()} steps
        {l0.stepsPerSec > 0 && (
          <span
            style={{
              fontWeight: "normal",
              fontSize: "14px",
              marginLeft: "12px",
            }}
          >
            ({l0.stepsPerSec.toFixed(1)}M steps/s,{" "}
            {l0.guestSteps.toLocaleString()} guest steps)
          </span>
        )}
      </h2>
      <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
        {tower.map((level, i) => {
          // For L0, trim to maxHeadPos + 10; for decoded levels show full tape
          const tape =
            i === 0
              ? level.tape.slice(
                  0,
                  Math.max(l0.maxHeadPos, l0.headPos) + 10,
                )
              : level.tape;
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
                {level.tapeLen.toLocaleString()} symbols
              </div>
              <div
                style={{
                  fontFamily: "var(--mono)",
                  fontSize: "12px",
                  lineHeight: "1.3",
                  overflowWrap: "break-word",
                }}
                dangerouslySetInnerHTML={{
                  __html: colorizeTape(tape, level.headPos),
                }}
              />
            </div>
          );
        })}
      </div>
    </div>
  );
}
