import { z } from "zod";
import { useEffect, useRef, useState } from "react";

export interface UtmMeta {
  utmStates: string[];
  utmSymbolChars: string;
}

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

interface TowerLevel {
  steps: number;
  maxHeadPos: number;
  state: string;
  headPos: number;
  tape: string;
}

const TotalEvent = z.object({
  type: z.literal("total"),
  unblemished: z.string(),
  utm_states: z.array(z.string()),
  utm_symbol_chars: z.string(),
  levels: z.array(
    z.object({
      steps: z.number(),
      max_head_pos: z.number(),
      state: z.string(),
      head_pos: z.number(),
      overwrites: z.record(z.number(), z.string()),
    }),
  ),
});
type TotalEvent = z.infer<typeof TotalEvent>;
const DeltaEvent = z.object({
  type: z.literal("delta"),
  levels: z.array(
    z.object({
      steps: z.number(),
      max_head_pos: z.number(),
      state: z.string(),
      head_pos: z.number(),
      overwrites: z.record(z.number(), z.string()),
    }),
  ),
});
type DeltaEvent = z.infer<typeof DeltaEvent>;
const SseEvent = z.union([TotalEvent, DeltaEvent]);

function useSseTower(): { meta: UtmMeta | null; tower: TowerLevel[] | null, emptyLevel: TowerLevel } {
  const unblemishedRef = useRef<string>("");
  const [meta, setMeta] = useState<UtmMeta | null>(null);

  const towerRef = useRef<TowerLevel[] | null>(null);
  const [tower, setTower] = useState<TowerLevel[] | null>(null);

  const [emptyLevel, setEmptyLevel] = useState<TowerLevel>({
    headPos: 0,
    state: 'Init',
    steps: 0,
    tape: '',
    maxHeadPos: 0,
  });

  useEffect(() => {
    const es = new EventSource("/api/tower");
    es.onmessage = (event) => {
      const msg = SseEvent.parse(JSON.parse(event.data));

      switch (msg.type) {
        case "total": {
          unblemishedRef.current = msg.unblemished;
          setMeta({
            utmStates: msg.utm_states,
            utmSymbolChars: msg.utm_symbol_chars,
          });
          towerRef.current = msg.levels.map((level) => ({
            steps: level.steps,
            maxHeadPos: level.max_head_pos,
            state: level.state,
            headPos: level.head_pos,
            tape: Array.from(
              {
                length: 1+Math.max(
                  level.head_pos,
                  ...Object.keys(level.overwrites).map(Number),
                ),
              },
              (_, i) =>
                level.overwrites[i] ??
                unblemishedRef.current.charAt(i) ??
                "_",
            ).join(""),
          }));
          setTower(towerRef.current);
          setEmptyLevel(l => ({...l, tape: unblemishedRef.current.slice(0, 200)}))
          break;
        }
        case "delta": {
          towerRef.current = msg.levels.map((level, depth) => ({
            steps: level.steps,
            maxHeadPos: level.max_head_pos,
            state: level.state,
            headPos: level.head_pos,
            tape: Array.from(
              {
                length: 1+Math.max(
                  towerRef.current?.[depth]?.tape.length ?? 0,
                  level.head_pos,
                  ...Object.keys(level.overwrites).map(Number),
                ),
              },
              (_, i) =>
                level.overwrites[i] ??
                towerRef.current?.[depth]?.tape[i] ??
                unblemishedRef.current[i] ??
                "_",
            ).join(""),
          }));
          setTower(towerRef.current);
          break;
        }
      }
    };
    return () => es.close();
  }, []);

  return {
    meta,
    tower,
    emptyLevel,
  };
}

function TowerLevelView({ level, name }: { level: TowerLevel, name: string }) {
  return <div
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
    {name} &middot; <span style={{fontFamily: 'monospace'}}>{level.steps}</span> steps &middot; <span style={{fontFamily: 'monospace'}}>{level.state}</span>
  </div>
  <div
    style={{
      fontFamily: "var(--mono)",
      fontSize: "12px",
      lineHeight: "1.3",
      overflowWrap: "break-word",
    }}
    dangerouslySetInnerHTML={{
      __html: colorizeTape(level.tape.slice(0, level.maxHeadPos+100) + ' and so on', level.headPos),
    }}
  />
</div>
}

// ── Main component ──

export function TowerView() {
  const { meta, tower, emptyLevel } = useSseTower();

  if (!meta || !tower) {
    return <div style={{ padding: "16px" }}>Loading...</div>;
  }

  return (
    <div style={{ textAlign: "left", padding: "16px" }}>
      <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
        {[...tower].map((level, i) => <TowerLevelView key={i} level={level} name={`L${i}`} />)}
        {Array.from({length: 5}).map((_, i) => <TowerLevelView key={i} level={emptyLevel} name={`L${i+tower.length}`} />)}
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
              fontFamily: "var(--mono)",
              fontSize: "12px",
              lineHeight: "1.3",
              overflowWrap: "break-word",
            }}
          >
            and so on
          </div>
        </div>
      </div>
    </div>
  );
}
