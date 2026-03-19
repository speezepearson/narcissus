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

// ── Section collapsing ──
// For sections longer than CAP, show: first BOOKEND + sliding middle + last BOOKEND.
// The middle shrinks by ELLIPSIS_WIDTH when it's disjoint from both bookends,
// so the total visual width (chars + ellipses) stays constant.

type IndexRange = { from: number; to: number }; // inclusive

const SECTION_CAP = 100;
const SECTION_BOOKEND = 20;
const ELLIPSIS_WIDTH = 3; // visual width of "..." in monospace chars

function collapseSection(
  start: number,
  end: number,
  headPos: number,
): IndexRange[] {
  const len = end - start + 1;
  if (len <= SECTION_CAP) return [{ from: start, to: end }];

  const aTo = start + SECTION_BOOKEND - 1;
  const cFrom = end - SECTION_BOOKEND + 1;

  // When middle is disjoint from both bookends, there are 2 internal gaps (2 ellipses).
  // When adjacent to one bookend, 1 gap. Budget the middle so total visual width is constant:
  // baseline = SECTION_CAP + 1 ellipsis (prefix...suffix when merged with middle).
  // With 2 gaps we need middleBudget + ELLIPSIS_WIDTH = fullMiddleBudget, so
  // middleBudget = fullMiddleBudget - ELLIPSIS_WIDTH.
  const fullMiddleBudget = SECTION_CAP - 2 * SECTION_BOOKEND;

  // First compute position with full budget to determine adjacency
  const bMinStart = aTo + 1;
  const bMaxEnd = cFrom - 1;
  const fullBMaxStart = bMaxEnd - fullMiddleBudget + 1;

  let bFrom: number;
  if (headPos < bMinStart) {
    bFrom = bMinStart;
  } else if (headPos > fullBMaxStart + fullMiddleBudget - 1) {
    bFrom = fullBMaxStart;
  } else {
    bFrom = Math.max(bMinStart, Math.min(fullBMaxStart, headPos - Math.floor(fullMiddleBudget / 2)));
  }

  // Check adjacency with full budget
  const adjacentToPrefix = bFrom === bMinStart;
  const adjacentToSuffix = bFrom + fullMiddleBudget - 1 === bMaxEnd;

  if (adjacentToPrefix || adjacentToSuffix) {
    // 1 internal gap — use full budget
    return [
      { from: start, to: aTo },
      { from: bFrom, to: bFrom + fullMiddleBudget - 1 },
      { from: cFrom, to: end },
    ];
  }

  // 2 internal gaps — shrink middle to compensate for extra ellipsis
  const shrunkBudget = fullMiddleBudget - ELLIPSIS_WIDTH;
  const shrunkBMaxStart = bMaxEnd - shrunkBudget + 1;
  bFrom = Math.max(bMinStart, Math.min(shrunkBMaxStart, headPos - Math.floor(shrunkBudget / 2)));

  return [
    { from: start, to: aTo },
    { from: bFrom, to: bFrom + shrunkBudget - 1 },
    { from: cFrom, to: end },
  ];
}

function computeVisibleRanges(
  tapeLen: number,
  headPos: number,
  sections: { name: SectionName; start: number; end: number }[] | undefined,
  visibleRadius: number | undefined,
): IndexRange[] {
  let ranges: IndexRange[];

  if (sections && sections.length > 0) {
    ranges = [];
    for (const s of sections) {
      ranges.push(...collapseSection(s.start, s.end, headPos));
    }
  } else {
    ranges = [{ from: 0, to: tapeLen - 1 }];
  }

  // Intersect with global radius
  if (visibleRadius != null) {
    const gFrom = Math.max(0, headPos - visibleRadius);
    const gTo = Math.min(tapeLen - 1, headPos + visibleRadius);
    ranges = ranges
      .map((r) => ({ from: Math.max(r.from, gFrom), to: Math.min(r.to, gTo) }))
      .filter((r) => r.from <= r.to);
  }

  // Merge adjacent/overlapping
  if (ranges.length === 0) return [];
  ranges.sort((a, b) => a.from - b.from);
  const merged: IndexRange[] = [{ ...ranges[0] }];
  for (let i = 1; i < ranges.length; i++) {
    const last = merged[merged.length - 1];
    if (ranges[i].from <= last.to + 1) {
      last.to = Math.max(last.to, ranges[i].to);
    } else {
      merged.push({ ...ranges[i] });
    }
  }
  return merged;
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
  const visibleRanges = useMemo(
    () => computeVisibleRanges(tape.length, headPos, sections, visibleRadius),
    [tape.length, headPos, sections, visibleRadius],
  );

  // Build section label positions: label at first visible index of each section
  const sectionLabelAt = useMemo(() => {
    const map = new Map<number, SectionName>();
    if (!sections) return map;
    for (const s of sections) {
      if (s.name === "$") continue;
      // Find the first visible index within this section
      for (const r of visibleRanges) {
        if (r.to < s.start || r.from > s.end) continue;
        const firstVisible = Math.max(r.from, s.start);
        map.set(firstVisible, s.name);
        break;
      }
    }
    return map;
  }, [sections, visibleRanges]);

  const elements: React.ReactNode[] = [];

  for (let ri = 0; ri < visibleRanges.length; ri++) {
    const range = visibleRanges[ri];

    // Leading ellipsis
    if (ri === 0 && range.from > 0) {
      elements.push(<span key="pre-ellipsis" className="utm-ellipsis">...</span>);
    } else if (ri > 0) {
      elements.push(<span key={`gap-${ri}`} className="utm-ellipsis">...</span>);
    }

    for (let i = range.from; i <= range.to; i++) {
      const section = sections ? sectionForIndex(sections, i) : undefined;
      const color = section ? SECTION_COLORS[section] : "var(--text)";
      const isHead = i === headPos;
      const sLabel = sectionLabelAt.get(i);

      elements.push(
        <span key={i} style={{ position: "relative", display: "inline" }}>
          {sLabel && (
            <span className="utm-section-label" style={{ color }}>{sLabel}</span>
          )}
          <span
            className={isHead ? "utm-cell utm-cell-head" : "utm-cell"}
            style={isHead ? undefined : { color }}
          >
            {tape[i]}
          </span>
        </span>,
      );
    }
  }

  // Trailing ellipsis
  if (visibleRanges.length > 0 && visibleRanges[visibleRanges.length - 1].to < tape.length - 1) {
    elements.push(<span key="post-ellipsis" className="utm-ellipsis">...</span>);
  }

  return (
    <div style={{ marginBottom: 12 }}>
      <div className="utm-tape-label">{label}{stateLabel && <span style={{ color: "var(--text)" }}>{stateLabel}</span>}</div>
      <div className="utm-tape-wrap">{elements}</div>
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
  const [logRadius, setLogRadius] = useState(5);
  const visibleRadius = Math.round(10 ** logRadius);
  const [stepCount, setStepCount] = useState(0);

  const utmRef = useRef(utmSnapshot);
  const statusRef = useRef(utmStatus);
  const lastDecodedRef = useRef(lastDecoded);

  useEffect(() => { utmRef.current = utmSnapshot; }, [utmSnapshot]);
  useEffect(() => { statusRef.current = utmStatus; }, [utmStatus]);
  useEffect(() => { lastDecodedRef.current = lastDecoded; }, [lastDecoded]);

  const stepOnce = useCallback((snap: TuringMachineSnapshot<UState, USymbol>) => {
    padTape(snap, utmSpec.blank);
    const st = getStatus(step(snap));
    padTape(snap, utmSpec.blank);
    return st;
  }, [utmSpec.blank]);

  const doStep = useCallback(() => {
    if (statusRef.current !== "running") return;
    const next = copySnapshot(utmRef.current);
    const st = stepOnce(next);

    utmRef.current = next;
    statusRef.current = st;
    setUtmSnapshot(next);
    setUtmStatus(st);
    setStepCount((c) => c + 1);

    const decoded = utmSpec.decode(simSpec, next);
    if (decoded) {
      lastDecodedRef.current = decoded;
      setLastDecoded(decoded);
    }

    if (st !== "running") {
      setPlaying(false);
    }
  }, [utmSpec, simSpec, stepOnce]);

  const doStepState = useCallback(() => {
    if (statusRef.current !== "running") return;
    const snap = copySnapshot(utmRef.current);
    const startState = snap.state;
    let st: "accept" | "reject" | "running" = "running";
    let steps = 0;
    while (st === "running" && snap.state === startState) {
      st = stepOnce(snap);
      steps++;
    }

    utmRef.current = snap;
    statusRef.current = st;
    setUtmSnapshot(snap);
    setUtmStatus(st);
    setStepCount((c) => c + steps);

    const decoded = utmSpec.decode(simSpec, snap);
    if (decoded) {
      lastDecodedRef.current = decoded;
      setLastDecoded(decoded);
    }

    if (st !== "running") {
      setPlaying(false);
    }
  }, [utmSpec, simSpec, stepOnce]);

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

  const fpsRef = useRef(fps);
  useEffect(() => { fpsRef.current = fps; }, [fps]);
  const accumRef = useRef(0);
  const stepCountRef = useRef(0);
  useEffect(() => { stepCountRef.current = stepCount; }, [stepCount]);

  useEffect(() => {
    if (!playing) {
      accumRef.current = 0;
      return;
    }
    const MAX_RENDER_FPS = 30;
    const interval = setInterval(() => {
      if (statusRef.current !== "running") return;
      accumRef.current += fpsRef.current / MAX_RENDER_FPS;
      const stepsThisFrame = Math.floor(accumRef.current);
      accumRef.current -= stepsThisFrame;
      if (stepsThisFrame === 0) return;

      const snap = copySnapshot(utmRef.current);
      let st: "accept" | "reject" | "running" = "running";
      for (let i = 0; i < stepsThisFrame; i++) {
        st = stepOnce(snap);
        if (st !== "running") break;
      }

      utmRef.current = snap;
      statusRef.current = st;
      stepCountRef.current += stepsThisFrame;
      setUtmSnapshot(snap);
      setUtmStatus(st);
      setStepCount(stepCountRef.current);

      // Only decode once per render frame (on the final state)
      const decoded = utmSpec.decode(simSpec, snap);
      if (decoded) {
        lastDecodedRef.current = decoded;
        setLastDecoded(decoded);
      }

      if (st !== "running") {
        setPlaying(false);
      }
    }, 1000 / MAX_RENDER_FPS);
    return () => clearInterval(interval);
  }, [playing, stepOnce, utmSpec, simSpec]);

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
        UTM step: {stepCount} | UTM state: {utmSnapshot.state} | head: {utmSnapshot.pos}
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
        <button onClick={doStepState} disabled={halted}>
          Step State
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
