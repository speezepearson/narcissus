import { useCallback, useEffect, useRef, useState, useMemo } from "react";
import {
  type TuringMachineSpec,
  type TuringMachineSnapshot,
  type UtmSpec,
  copySnapshot,
  makeInitSnapshot,
  step,
  getStatus,
} from "./types";

function padTape<State extends string, Symbol extends string>(
  snapshot: TuringMachineSnapshot<State, Symbol>,
  blank: Symbol,
) {
  while (snapshot.tape.length <= snapshot.pos) {
    snapshot.tape.push(blank);
  }
}

// Section layout: $#RULES#ACCEPT#STATE#TAPELEN#BLANK#TAPE
type SectionName = "$" | "rules" | "accept states" | "state" | "tapelen" | "blank" | "tape";

const SECTION_COLORS: Record<SectionName, string> = {
  "$":             "#78716c",  // stone
  "rules":         "#6366f1",  // indigo
  "accept states": "#16a34a",  // green
  "state":         "#ea580c",  // orange
  "tapelen":       "#9333ea",  // purple
  "blank":         "#db2777",  // pink
  "tape":          "#0891b2",  // cyan
};

const SECTION_ORDER: SectionName[] = [
  "$", "rules", "accept states", "state", "tapelen", "blank", "tape",
];

function parseSections(tape: readonly string[]): { name: SectionName; start: number; end: number }[] {
  // Find all # positions
  const hashes: number[] = [];
  for (let i = 0; i < tape.length; i++) {
    if (tape[i] === "#") hashes.push(i);
  }

  const sections: { name: SectionName; start: number; end: number }[] = [];

  // $ is index 0 (if tape starts with $)
  if (tape.length > 0 && tape[0] === "$") {
    sections.push({ name: "$", start: 0, end: 0 });
  }

  // Each # starts the next section (the # itself is a separator, content is after it)
  // But we want to color the # as part of the section it introduces
  for (let i = 0; i < hashes.length && i < 6; i++) {
    const sectionName = SECTION_ORDER[i + 1]; // skip "$"
    const start = hashes[i]; // include the # itself
    const end = i + 1 < hashes.length ? hashes[i + 1] - 1 : tape.length - 1;
    sections.push({ name: sectionName, start, end });
  }

  return sections;
}

function sectionForIndex(
  sections: { name: SectionName; start: number; end: number }[],
  idx: number,
): SectionName | undefined {
  for (const s of sections) {
    if (idx >= s.start && idx <= s.end) return s.name;
  }
  return undefined;
}

/** Render a tape as a wrapping grid of colored characters with head highlight. */
function TapeDisplay({
  tape,
  headPos,
  sections,
  label,
  stateLabel,
  visibleRadius,
}: {
  tape: readonly string[];
  headPos: number;
  sections?: { name: SectionName; start: number; end: number }[];
  label: string;
  stateLabel?: string;
  visibleRadius?: number;
}) {
  // Compute visible range
  const showFrom = visibleRadius != null ? Math.max(0, headPos - visibleRadius) : 0;
  const showTo = visibleRadius != null ? Math.min(tape.length - 1, headPos + visibleRadius) : tape.length - 1;

  // Build labeled section starts
  const sectionStarts = new Map<number, SectionName>();
  if (sections) {
    for (const s of sections) {
      if (s.name === "$") {
        if (0 >= showFrom && 0 <= showTo) sectionStarts.set(0, "$");
      } else {
        // If the section's # is before our window, label at the window start if we're inside that section
        if (s.start >= showFrom && s.start <= showTo) {
          sectionStarts.set(s.start, s.name);
        } else if (s.start < showFrom && s.end >= showFrom) {
          sectionStarts.set(showFrom, s.name);
        }
      }
    }
  }

  const clipped = showFrom > 0 || showTo < tape.length - 1;

  return (
    <div style={{ marginBottom: 12 }}>
      <div className="utm-tape-label">{label}{stateLabel && <span style={{ color: "var(--text)" }}>{stateLabel}</span>}</div>
      <div className="utm-tape-wrap">
        {clipped && showFrom > 0 && <span className="utm-ellipsis">...</span>}
        {tape.map((ch, i) => {
          if (i < showFrom || i > showTo) return null;
          const section = sections ? sectionForIndex(sections, i) : undefined;
          const color = section ? SECTION_COLORS[section] : "var(--text)";
          const isHead = i === headPos;
          const sectionLabel = sectionStarts.get(i);

          return (
            <span key={i} style={{ position: "relative", display: "inline" }}>
              {sectionLabel && sectionLabel !== "$" && (
                <span className="utm-section-label" style={{ color }}>{sectionLabel}</span>
              )}
              <span
                className={isHead ? "utm-cell utm-cell-head" : "utm-cell"}
                style={isHead ? undefined : { color }}
              >
                {ch}
              </span>
            </span>
          );
        })}
        {clipped && showTo < tape.length - 1 && <span className="utm-ellipsis">...</span>}
      </div>
    </div>
  );
}

type UTMViewerProps<
  UState extends string,
  USymbol extends string,
  SimState extends string,
  SimSymbol extends string,
> = {
  utmSpec: UtmSpec<UState, USymbol>;
  simSpec: TuringMachineSpec<SimState, SimSymbol>;
  initialSimTape: readonly SimSymbol[];
};

export function UTMViewer<
  UState extends string,
  USymbol extends string,
  SimState extends string,
  SimSymbol extends string,
>({ utmSpec, simSpec, initialSimTape }: UTMViewerProps<UState, USymbol, SimState, SimSymbol>) {
  const makeInitial = useCallback(() => {
    const simSnapshot = makeInitSnapshot(simSpec, initialSimTape);
    const utmTape = utmSpec.encode(simSnapshot);
    const utmSnapshot = makeInitSnapshot(utmSpec, utmTape);
    padTape(utmSnapshot, utmSpec.blank);
    const decoded = utmSpec.decode(simSpec, utmSnapshot);
    return { utmSnapshot, decoded };
  }, [utmSpec, simSpec, initialSimTape]);

  const [utmSnapshot, setUtmSnapshot] = useState(() => makeInitial().utmSnapshot);
  const [utmStatus, setUtmStatus] = useState<"accept" | "reject" | "running">("running");
  const [lastDecoded, setLastDecoded] = useState<TuringMachineSnapshot<SimState, SimSymbol> | null>(
    () => makeInitial().decoded ?? null,
  );
  const [playing, setPlaying] = useState(false);
  const [logFps, setLogFps] = useState(Math.log10(5));
  const fps = Math.round(10 ** logFps);
  const [logRadius, setLogRadius] = useState(3); // default: show all (10^3 = 1000)
  const visibleRadius = Math.round(10 ** logRadius);
  const [stepCount, setStepCount] = useState(0);

  const utmRef = useRef(utmSnapshot);
  const statusRef = useRef(utmStatus);
  const lastDecodedRef = useRef(lastDecoded);

  useEffect(() => { utmRef.current = utmSnapshot; }, [utmSnapshot]);
  useEffect(() => { statusRef.current = utmStatus; }, [utmStatus]);
  useEffect(() => { lastDecodedRef.current = lastDecoded; }, [lastDecoded]);

  const doStep = useCallback(() => {
    if (statusRef.current !== "running") return;
    const next = copySnapshot(utmRef.current);
    padTape(next, utmSpec.blank);
    const status = getStatus(step(next));
    padTape(next, utmSpec.blank);

    utmRef.current = next;
    statusRef.current = status;
    setUtmSnapshot(next);
    setUtmStatus(status);
    setStepCount((c) => c + 1);

    const decoded = utmSpec.decode(simSpec, next);
    if (decoded) {
      lastDecodedRef.current = decoded;
      setLastDecoded(decoded);
    }

    if (status !== "running") {
      setPlaying(false);
    }
  }, [utmSpec, simSpec]);

  const reset = useCallback(() => {
    const { utmSnapshot: snap, decoded } = makeInitial();
    utmRef.current = snap;
    statusRef.current = "running";
    lastDecodedRef.current = decoded ?? null;
    setUtmSnapshot(snap);
    setUtmStatus("running");
    setLastDecoded(decoded ?? null);
    setPlaying(false);
    setStepCount(0);
  }, [makeInitial]);

  useEffect(() => {
    if (!playing) return;
    const interval = setInterval(doStep, 1000 / fps);
    return () => clearInterval(interval);
  }, [playing, fps, doStep]);

  const halted = utmStatus !== "running";

  const utmSections = useMemo(
    () => parseSections(utmSnapshot.tape as string[]),
    [utmSnapshot.tape],
  );

  // Pad simulated tape for display
  const simTapeDisplay = useMemo(() => {
    if (!lastDecoded) return null;
    const tape = lastDecoded.tape.slice() as string[];
    while (tape.length <= lastDecoded.pos) {
      tape.push(simSpec.blank);
    }
    return tape;
  }, [lastDecoded, simSpec.blank]);

  return (
    <div className="tm-viewer">
      <div className="utm-status-line">
        UTM step: {stepCount} | UTM state: {utmSnapshot.state}
      </div>

      <TapeDisplay
        tape={utmSnapshot.tape as string[]}
        headPos={utmSnapshot.pos}
        sections={utmSections}
        label="UTM tape"
        visibleRadius={visibleRadius}
      />

      {lastDecoded && simTapeDisplay && (
        <TapeDisplay
          tape={simTapeDisplay}
          headPos={lastDecoded.pos}
          label="Simulated TM tape"
          stateLabel={` (state=${lastDecoded.state})`}
        />
      )}

      {halted && (
        <div className={`tm-result tm-result-${utmStatus}`}>
          UTM {utmStatus.toUpperCase()}
        </div>
      )}

      <div className="tm-controls">
        <button onClick={doStep} disabled={halted}>
          Step
        </button>
        <button onClick={() => setPlaying((p) => !p)} disabled={halted}>
          {playing ? "Pause" : "Play"}
        </button>
        <button onClick={reset}>Reset</button>
        <label className="tm-fps">
          FPS:
          <input
            type="range"
            min={0}
            max={6}
            step={0.1}
            value={logFps}
            onChange={(e) => setLogFps(Number(e.target.value))}
          />
          <span>{fps}</span>
        </label>
        <label className="tm-fps">
          Radius:
          <input
            type="range"
            min={0}
            max={3}
            step={0.1}
            value={logRadius}
            onChange={(e) => setLogRadius(Number(e.target.value))}
          />
          <span>{visibleRadius}</span>
        </label>
      </div>
    </div>
  );
}
