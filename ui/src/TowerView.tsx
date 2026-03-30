import { z } from "zod";
import { useEffect, useMemo, useRef, useState } from "react";

export interface UtmMeta {
  utmStates: string[];
  utmSymbolChars: string;
}

import { colorizeTape } from "./colorizeTape";

// ── L0 state from server ──

interface TowerLevel {
  steps: number;
  maxHeadPos: number;
  state: string;
  headPos: number;
  tape: string[];
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

function useSseTower(): {
  meta: UtmMeta | null;
  tower: TowerLevel[] | null;
  emptyLevel: TowerLevel;
} {
  const unblemishedRef = useRef<string[]>([]);
  const [meta, setMeta] = useState<UtmMeta | null>(null);

  const towerRef = useRef<TowerLevel[] | null>(null);
  const [tower, setTower] = useState<TowerLevel[] | null>(null);

  const [emptyLevel, setEmptyLevel] = useState<TowerLevel>({
    headPos: 0,
    state: "Init",
    steps: 0,
    tape: ["$"],
    maxHeadPos: 0,
  });

  useEffect(() => {
    const es = new EventSource("/api/tower");
    es.onmessage = (event) => {
      const msg = SseEvent.parse(JSON.parse(event.data));

      switch (msg.type) {
        case "total": {
          unblemishedRef.current = msg.unblemished.split("");
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
                length:
                  1 +
                  Math.max(
                    level.head_pos,
                    ...Object.keys(level.overwrites).map(Number),
                  ),
              },
              (_, i) => level.overwrites[i] ?? unblemishedRef.current[i] ?? "_",
            ),
          }));
          setTower(towerRef.current);
          setEmptyLevel((l) => ({
            ...l,
            tape: unblemishedRef.current.slice(0, 1),
          }));
          break;
        }
        case "delta": {
          towerRef.current = msg.levels.map((level, depth) => {
            const res = {
              tape: [],
              ...towerRef.current?.[depth],
              steps: level.steps,
              maxHeadPos: level.max_head_pos,
              state: level.state,
              headPos: level.head_pos,
            };
            const realizedLength =
              1 +
              Math.max(
                level.max_head_pos,
                ...Object.keys(level.overwrites).map(Number),
              );
            if (res.tape.length < realizedLength) {
              // console.log("tape.length < realizedLength", res.tape.length, realizedLength);
              res.tape.push(
                ...unblemishedRef.current.slice(
                  res.tape.length,
                  realizedLength,
                ),
              );
            }
            for (const [pos, ch] of Object.entries(level.overwrites)) {
              res.tape[Number(pos)] = ch;
            }
            return res;
          });
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

function toExponential(
  x: number,
  nDecimal: number,
): { mantissa: string; exponent: number } {
  const exponent = 3 * Math.floor(Math.log10(x) / 3);
  const mantissa = x / 10 ** exponent;
  return { mantissa: mantissa.toFixed(nDecimal), exponent };
}

function TowerLevelView({ level, name }: { level: TowerLevel; name: string }) {
  const fontSize = useMemo(() => {
    return `${Math.min(1, Math.max(0.2, Math.pow(7000 / level.tape.length, 2)))}em`;
  }, [level.tape.length]);

  const prettyNSteps = useMemo(() => {
    if (level.steps < 1000) return level.steps;
    const { mantissa, exponent } = toExponential(level.steps, 2);
    return (
      <>
        {mantissa} x 10<sup>{exponent}</sup>
      </>
    );
  }, [level.steps]);

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
          fontSize: "11px",
          color: "#888",
          marginBottom: "4px",
        }}
      >
        {name} &middot;{" "}
        <span style={{ fontFamily: "monospace" }}>{prettyNSteps}</span> step
        {level.steps === 1 ? "" : "s"}
        &middot; <span style={{ fontFamily: "monospace" }}>{level.state}</span>
      </div>
      <div
        style={{
          fontFamily: "var(--mono)",
          lineHeight: "1.3",
          overflowWrap: "break-word",
          fontSize,
        }}
        dangerouslySetInnerHTML={{
          __html:
            colorizeTape(level.tape, level.headPos) +
            " &nbsp;&nbsp;&nbsp; ...and so on",
        }}
      />
    </div>
  );
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
        {[...tower].map((level, i) => (
          <TowerLevelView key={i} level={level} name={`L${i}`} />
        ))}
        {Array.from({ length: 5 }).map((_, i) => (
          <TowerLevelView
            key={i}
            level={emptyLevel}
            name={`L${i + tower.length}`}
          />
        ))}
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
            ...and so on
          </div>
        </div>
      </div>
    </div>
  );
}
