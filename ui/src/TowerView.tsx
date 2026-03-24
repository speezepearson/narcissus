import { useEffect, useRef, useState, useMemo } from "react";

interface TowerViewState {
  steps: number;
  guestSteps: number;
  stepsPerSec: number;
  tower: Array<{
    state: string;
    headPos: number;
    maxHeadPos: number;
    tape: string;
    tapeLen: number;
  }>;
}

// ── Semantic tape structure ──

interface TapeSections {
  prefix: string; // "$"
  rules: string[]; // individual rules (starting with "." or "*")
  accepting: string;
  state: string;
  blank: string;
  tapeCells: string[]; // individual cells
  unparsed?: string; // fallback if format doesn't match
}

function parseTape(tape: string): TapeSections | null {
  // Tape format: $#.rule1;.rule2;*activeRule#accepting#state#blank#^cell,cell,...
  // Find all # positions
  const hashPositions: number[] = [];
  for (let i = 0; i < tape.length; i++) {
    if (tape[i] === "#") hashPositions.push(i);
  }
  if (hashPositions.length < 5) return null;

  const prefix = tape.slice(0, hashPositions[0]); // "$"
  const rulesStr = tape.slice(hashPositions[0] + 1, hashPositions[1]);
  const accepting = tape.slice(hashPositions[1] + 1, hashPositions[2]);
  const state = tape.slice(hashPositions[2] + 1, hashPositions[3]);
  const blank = tape.slice(hashPositions[3] + 1, hashPositions[4]);
  const tapeStr = tape.slice(hashPositions[4] + 1);

  // Parse rules: split by ";" keeping each rule's leading "." or "*"
  const rules = rulesStr.length > 0 ? rulesStr.split(";") : [];

  // Parse tape cells: split by ","
  const tapeCells = tapeStr.length > 0 ? tapeStr.split(",") : [];

  return { prefix, rules, accepting, state, blank, tapeCells };
}

function HeadChar({ ch, isHead }: { ch: string; isHead: boolean }) {
  if (isHead) {
    return <span className="st-head">{ch}</span>;
  }
  return <>{ch}</>;
}

function CharSpan({ text, headPos, startIdx }: { text: string; headPos: number; startIdx: number }) {
  // Render a string of characters, highlighting the one at headPos
  const parts: React.ReactNode[] = [];
  let run = "";
  let runStart = startIdx;
  for (let i = 0; i < text.length; i++) {
    const globalIdx = startIdx + i;
    if (globalIdx === headPos) {
      if (run) parts.push(<span key={runStart}>{run}</span>);
      parts.push(<HeadChar key={globalIdx} ch={text[i]} isHead={true} />);
      run = "";
      runStart = globalIdx + 1;
    } else {
      run += text[i];
    }
  }
  if (run) parts.push(<span key={runStart}>{run}</span>);
  return <>{parts}</>;
}

function SemanticTape({ tape, headPos }: { tape: string; headPos: number }) {
  const parsed = useMemo(() => parseTape(tape), [tape]);

  if (!parsed) {
    // Fallback: render raw tape with head highlight
    return <CharSpan text={tape} headPos={headPos} startIdx={0} />;
  }

  // Build position tracking: we need to know where each section starts
  // in the original string to correctly place the head highlight
  let pos = parsed.prefix.length; // after "$"
  const rulesStart = pos + 1; // after first "#"

  // Calculate positions for each rule
  const rulePositions: number[] = [];
  let rp = rulesStart;
  for (let i = 0; i < parsed.rules.length; i++) {
    rulePositions.push(rp);
    rp += parsed.rules[i].length;
    if (i < parsed.rules.length - 1) rp += 1; // ";"
  }
  const afterRules = rp + 1; // skip second "#"

  const acceptingStart = afterRules;
  const afterAccepting = acceptingStart + parsed.accepting.length + 1; // skip "#"

  const stateStart = afterAccepting;
  const afterState = stateStart + parsed.state.length + 1; // skip "#"

  const blankStart = afterState;
  const afterBlank = blankStart + parsed.blank.length + 1; // skip "#"

  // Tape cell positions
  const cellPositions: number[] = [];
  let cp = afterBlank;
  for (let i = 0; i < parsed.tapeCells.length; i++) {
    cellPositions.push(cp);
    cp += parsed.tapeCells[i].length;
    if (i < parsed.tapeCells.length - 1) cp += 1; // ","
  }

  return (
    <span className="st-tape">
      <CharSpan text={parsed.prefix + "#"} headPos={headPos} startIdx={0} />
      <span className="st-section st-rules">
        <span className="st-label">rules</span>
        {parsed.rules.map((rule, i) => {
          const isActive = rule.startsWith("*");
          return (
            <span key={i}>
              {i > 0 && <span className="st-delim">;</span>}
              <span className={`st-rule${isActive ? " st-rule-active" : ""}`}>
                <CharSpan text={rule} headPos={headPos} startIdx={rulePositions[i]} />
              </span>
            </span>
          );
        })}
      </span>
      <CharSpan text="#" headPos={headPos} startIdx={afterRules - 1} />
      <span className="st-section st-accepting">
        <span className="st-label">accept</span>
        <CharSpan text={parsed.accepting} headPos={headPos} startIdx={acceptingStart} />
      </span>
      <CharSpan text="#" headPos={headPos} startIdx={afterAccepting - 1} />
      <span className="st-section st-state">
        <span className="st-label">state</span>
        <CharSpan text={parsed.state} headPos={headPos} startIdx={stateStart} />
      </span>
      <CharSpan text="#" headPos={headPos} startIdx={afterState - 1} />
      <span className="st-section st-blank">
        <span className="st-label">blank</span>
        <CharSpan text={parsed.blank} headPos={headPos} startIdx={blankStart} />
      </span>
      <CharSpan text="#" headPos={headPos} startIdx={afterBlank - 1} />
      <span className="st-section st-tape-cells">
        <span className="st-label">tape</span>
        {parsed.tapeCells.map((cell, i) => {
          const isActive = cell.startsWith("^") || cell.startsWith(">");
          return (
            <span key={i}>
              {i > 0 && <span className="st-delim">,</span>}
              <span className={`st-cell${isActive ? " st-cell-active" : ""}`}>
                <CharSpan text={cell} headPos={headPos} startIdx={cellPositions[i]} />
              </span>
            </span>
          );
        })}
      </span>
    </span>
  );
}

export function TowerView() {
  const [data, setData] = useState<TowerViewState | null>(null);
  const tapesRef = useRef<string[]>([]);
  const unblemishedRef = useRef<string>("");

  useEffect(() => {
    const es = new EventSource("/api/tower");
    es.onmessage = (event) => {
      const msg = JSON.parse(event.data);

      if (msg.type === "total") {
        // Store unblemished reference tape
        unblemishedRef.current = msg.unblemished;
        // Initialize tapes from total event
        tapesRef.current = msg.tower.map(
          (l: { tape: string }) => l.tape,
        );
        setData({
          steps: msg.steps,
          guestSteps: msg.guest_steps,
          stepsPerSec: msg.steps_per_sec,
          tower: msg.tower.map(
            (
              l: {
                state: string;
                head_pos: number;
                max_head_pos: number;
                tape: string;
                tape_len: number;
              },
            ) => ({
              state: l.state,
              headPos: l.head_pos,
              maxHeadPos: l.max_head_pos,
              tape: l.tape,
              tapeLen: l.tape_len,
            }),
          ),
        });
      } else if (msg.type === "delta") {
        // Apply new_overwrites to stored tapes
        for (let i = 0; i < msg.tower.length; i++) {
          const level = msg.tower[i];
          let tape = tapesRef.current[i] || "";
          const chars = tape.split("");
          // Extend tape to max_head_pos + 10 using unblemished content
          const end = Math.max(level.max_head_pos, level.head_pos) + 10;
          const ub = unblemishedRef.current;
          while (chars.length < end) {
            const pos = chars.length;
            chars.push(pos < ub.length ? ub[pos] : "_");
          }
          for (const [pos, ch] of level.new_overwrites as [
            number,
            string,
          ][]) {
            while (chars.length <= pos) chars.push("_");
            chars[pos] = ch;
          }
          tapesRef.current[i] = chars.join("");
        }
        // Trim if tower shrank
        tapesRef.current.length = msg.tower.length;

        setData({
          steps: msg.total_steps,
          guestSteps: msg.guest_steps,
          stepsPerSec: msg.steps_per_sec,
          tower: msg.tower.map(
            (
              l: {
                state: string;
                head_pos: number;
                max_head_pos: number;
                new_overwrites: [number, string][];
                tape_len: number;
              },
              i: number,
            ) => ({
              state: l.state,
              headPos: l.head_pos,
              maxHeadPos: l.max_head_pos,
              tape: tapesRef.current[i],
              tapeLen: l.tape_len,
            }),
          ),
        });
      }
    };
    return () => es.close();
  }, []);

  if (!data) {
    return <div style={{ padding: "16px" }}>Loading...</div>;
  }

  return (
    <div style={{ textAlign: "left", padding: "16px" }}>
      <h2 style={{ marginBottom: "8px" }}>
        Tower &mdash; {data.steps.toLocaleString()} steps
        {data.stepsPerSec > 0 && (
          <span
            style={{
              fontWeight: "normal",
              fontSize: "14px",
              marginLeft: "12px",
            }}
          >
            ({data.stepsPerSec.toFixed(1)}M steps/s,{" "}
            {data.guestSteps.toLocaleString()} guest steps)
          </span>
        )}
      </h2>
      <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
        {data.tower.map((level, i) => {
          const end = Math.max(level.maxHeadPos, level.headPos) + 10;
          const tape = level.tape.slice(0, end);
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
                  lineHeight: "1.6",
                  overflowWrap: "break-word",
                }}
              >
                <SemanticTape tape={tape} headPos={level.headPos} />
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
