// ── UTM tape decoder and tower builder ──

export interface UtmMeta {
  utmStates: string[];
  utmSymbolChars: string;
}

export interface TowerLevel {
  state: string;
  headPos: number;
  tape: string;
  tapeLen: number;
}

interface DecodedLevel {
  state: string;
  headPos: number;
  tape: string;
}

export function numBits(count: number): number {
  return Math.max(1, Math.ceil(Math.log2(Math.max(count, 2))));
}

export function fromBinary(
  s: string,
  start: number,
  width: number,
): number | null {
  let val = 0;
  for (let i = start; i < start + width; i++) {
    const ch = s[i];
    if (ch === "0" || ch === "X") val = val * 2;
    else if (ch === "1" || ch === "Y") val = val * 2 + 1;
    else return null;
  }
  return val;
}

export function decodeUtmTape(
  tape: string,
  utmStates: string[],
  utmSymbolChars: string,
): DecodedLevel | null {
  const hashes: number[] = [];
  for (let i = 0; i < tape.length; i++) {
    if (tape[i] === "#") {
      hashes.push(i);
      if (hashes.length >= 5) break;
    }
  }
  if (hashes.length < 5) return null;

  const rulesStr = tape.slice(hashes[0] + 1, hashes[1]);
  const stateStr = tape.slice(hashes[2] + 1, hashes[3]);
  const tapeStr = tape.slice(hashes[4] + 1);

  const nStateBits = numBits(utmStates.length);
  const nSymBits = numBits(utmSymbolChars.length);

  if (rulesStr.length > 0) {
    const firstRule = rulesStr.split(";")[0];
    if (firstRule.length > 1) {
      const content = firstRule.slice(1);
      const pipes = content.split("|");
      if (pipes.length >= 2) {
        if (pipes[0].length !== nStateBits || pipes[1].length !== nSymBits) {
          return null;
        }
      }
    }
  }

  if (stateStr.length !== nStateBits) return null;
  const stateIdx = fromBinary(stateStr, 0, nStateBits);
  if (stateIdx === null || stateIdx >= utmStates.length) return null;
  const stateName = utmStates[stateIdx];

  const cells: number[] = [];
  let headPos = 0;
  let cellIdx = 0;
  let i = 0;
  while (i < tapeStr.length) {
    const ch = tapeStr[i];
    if (ch === "_" || ch === "$") break;
    if (ch === ",") {
      cellIdx++;
      i++;
      continue;
    }
    if (ch === "^" || ch === ">") {
      if (ch === "^") headPos = cellIdx;
      i++;
      continue;
    }
    if (i + nSymBits > tapeStr.length) break;
    const val = fromBinary(tapeStr, i, nSymBits);
    if (val === null || val >= utmSymbolChars.length) break;
    cells.push(val);
    i += nSymBits;
  }

  if (cells.length === 0) return null;

  const decodedTape = cells.map((idx) => utmSymbolChars[idx]).join("");
  return { state: stateName, headPos, tape: decodedTape };
}

const MAX_TOWER_DEPTH = 10;

/** Build tower from scratch by recursively decoding. Gold standard. */
export function buildTower(l0: TowerLevel, meta: UtmMeta): TowerLevel[] {
  const levels: TowerLevel[] = [l0];

  let currentTape = l0.tape;
  for (let depth = 0; depth < MAX_TOWER_DEPTH; depth++) {
    const decoded = decodeUtmTape(
      currentTape,
      meta.utmStates,
      meta.utmSymbolChars,
    );
    if (!decoded) break;
    levels.push({
      state: decoded.state,
      headPos: decoded.headPos,
      tape: decoded.tape,
      tapeLen: decoded.tape.length,
    });
    currentTape = decoded.tape;
  }

  return levels;
}

/**
 * Incrementally update tower in-place. Only re-decodes level N+1 if
 * level N's state changed, then stops early if the decoded state matches
 * what was already stored.
 */
export function updateTower(
  newL0: TowerLevel,
  tower: TowerLevel[],
  meta: UtmMeta,
): void {
  const oldL0State = tower[0]?.state;
  tower[0] = newL0;

  if (newL0.state === oldL0State && tower.length > 1) {
    return;
  }

  let currentTape = newL0.tape;
  for (let depth = 0; depth < MAX_TOWER_DEPTH; depth++) {
    const decoded = decodeUtmTape(
      currentTape,
      meta.utmStates,
      meta.utmSymbolChars,
    );
    if (!decoded) {
      tower.length = depth + 1;
      return;
    }

    const oldState = tower[depth + 1]?.state;
    tower[depth + 1] = {
      state: decoded.state,
      headPos: decoded.headPos,
      tape: decoded.tape,
      tapeLen: decoded.tape.length,
    };

    if (decoded.state === oldState && tower.length > depth + 2) {
      // State at this level didn't change; preserve deeper levels as-is.
      return;
    }
    currentTape = decoded.tape;
  }
}
