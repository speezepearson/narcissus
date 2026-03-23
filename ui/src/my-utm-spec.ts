import {
  makeSimpleTapeOverlay,
  type Dir,
  type StateIdx,
  type SymbolIdx,
  type TapeIdx,
  type TapeOverlay,
  type TuringMachineSnapshot,
  type TuringMachineSpec,
  type UtmSnapshot,
  type UtmSpec,
} from "./types";
import {
  indexOf,
  must,
  mustStateIndex,
  mustSymbolIndex,
  tapeIndexOf,
} from "./util";

// ════════════════════════════════════════════════════════════════════
// UTM Alphabet
// ════════════════════════════════════════════════════════════════════
const allSymbols = [
  "_", // blank (UTM blank)
  "0",
  "1", // binary data
  "X",
  "Y", // marked 0/1 during compare/copy
  "#", // section separator
  "|", // field separator within a rule
  ";", // rule separator / accept-state separator
  ",", // cell separator (head not here)
  "^", // head marker (head is here)
  "l", // left direction
  "d", // right direction
  ".", // inactive rule prefix
  "*", // active rule prefix
  ">", // temp marker (for head movement)
  "$", // left boundary marker (position 0)
] as const;
export type MyUtmSymbol = (typeof allSymbols)[number];

// ════════════════════════════════════════════════════════════════════
// Binary encoding helpers
// ════════════════════════════════════════════════════════════════════
export function numBits(count: number): number {
  return Math.max(1, Math.ceil(Math.log2(Math.max(2, count))));
}

export function toBinary(
  index: StateIdx | SymbolIdx,
  width: number,
): ("1" | "0")[] {
  const bits: ("1" | "0")[] = [];
  for (let i = width - 1; i >= 0; i--) {
    bits.push(((index >> i) & 1) === 1 ? "1" : "0");
  }
  return bits;
}

/** Read `width` bits from a TapeOverlay starting at `start` and interpret as a binary number. */
function fromBinaryAt(
  tape: TapeOverlay<MyUtmSymbol>,
  start: TapeIdx,
  width: number,
): number {
  let val = 0;
  for (let i = 0; i < width; i++) {
    const b = tape.get(start + i);
    switch (b) {
      case "1":
      case "Y":
        val = val * 2 + 1;
        break;
      case "0":
      case "X":
        val = val * 2;
        break;
      default:
        throw new Error(`invalid binary symbol: ${b}`);
    }
  }
  return val;
}

// ════════════════════════════════════════════════════════════════════
// Tape Layout
// ════════════════════════════════════════════════════════════════════
//
// $#RULES#ACCEPTSTATES#STATE#BLANK#,CELL0^CELL1,CELL2,...
//
// $ = left boundary (position 0), only appears here
// # = section separator (5 of them)
//
// RULES: ;-separated rules, each prefixed . (inactive) or * (active)
//   Format: .STATEBITS|SYMBITS|NEWSTATEBITS|NEWSYMBITS|l_or_d
//
// ACCEPTSTATES: ;-separated binary-encoded state indices (may be empty)
//
// STATE: binary-encoded current simulated state index
//
// BLANK: binary-encoded blank symbol index (so UTM can extend tape)
//
// TAPE: cells separated by , (inactive) or ^ (head position)
//   Each cell is fixed-width binary-encoded symbol index

// ════════════════════════════════════════════════════════════════════
// Encode
// ════════════════════════════════════════════════════════════════════
function encodeToTape<SimState extends string, SimSymbol extends string>(
  snapshot: TuringMachineSnapshot<SimState, SimSymbol>,
  optimizationHints: Array<[SimState, SimSymbol]> = [],
): TapeOverlay<MyUtmSymbol> {
  const { spec, state, tape, pos } = snapshot;
  const stateIdx = mustStateIndex(spec.allStates, state);

  const header = buildHeader(spec, stateIdx, optimizationHints);

  return makeSimpleTapeOverlay(
    makeEncodeTapeOverlayBackground(spec, header, pos, tape),
  );
}

/** Build the header portion of the UTM tape (everything before the TAPE section cells). */
function buildHeader<SimState extends string, SimSymbol extends string>(
  spec: TuringMachineSpec<SimState, SimSymbol>,
  stateIdx: StateIdx,
  optimizationHints: Array<[SimState, SimSymbol]>,
): MyUtmSymbol[] {
  const sBits = numBits(spec.allStates.length);
  const symBits = numBits(spec.allSymbols.length);

  /** The header portion of the UTM tape (everything before the TAPE section cells). */
  const header: MyUtmSymbol[] = [];

  // Boundary marker
  header.push("$");

  // RULES section
  const transitionOrder = new Map<SimState, Map<SimSymbol, number>>();
  for (let i = 0; i < optimizationHints.length; i++) {
    const [st, sym] = optimizationHints[i];
    if (!transitionOrder.has(st)) transitionOrder.set(st, new Map());
    transitionOrder.get(st)!.set(sym, i);
  }
  const transitions: Array<[SimState, SimSymbol, [SimState, SimSymbol, Dir]]> =
    [...spec.rules.entries()].flatMap(([st, ruleMap]) =>
      [...ruleMap.entries()].map(([sym, rule]) => [st, sym, rule]),
    );
  transitions.sort(
    (a, b) =>
      (transitionOrder.get(a[0])?.get(a[1]) ?? 0) -
      (transitionOrder.get(b[0])?.get(b[1]) ?? 0),
  );

  header.push("#");
  let first = true;
  for (const [st, sym, rule] of transitions) {
    const stIdx = mustStateIndex(spec.allStates, st);
    const symIdx = mustSymbolIndex(spec.allSymbols, sym);
    if (!first) header.push(";");
    first = false;
    const [newSt, newSym, dir] = rule;
    const newStIdx = mustStateIndex(spec.allStates, newSt);
    const newSymIdx = mustSymbolIndex(spec.allSymbols, newSym);
    header.push(".");
    header.push(...toBinary(stIdx, sBits));
    header.push("|");
    header.push(...toBinary(symIdx, symBits));
    header.push("|");
    header.push(...toBinary(newStIdx, sBits));
    header.push("|");
    header.push(...toBinary(newSymIdx, symBits));
    header.push("|");
    header.push(dir === "L" ? "l" : "d");
  }

  // ACCEPTSTATES section
  header.push("#");
  const accStates = spec.allStates.filter((s) => spec.acceptingStates.has(s));
  for (let i = 0; i < accStates.length; i++) {
    if (i > 0) header.push(";");
    const accStIdx = must(indexOf(spec.allStates, accStates[i])) as StateIdx;
    header.push(...toBinary(accStIdx, sBits));
  }

  // STATE section
  header.push("#");
  header.push(...toBinary(stateIdx, sBits));

  // BLANK section
  header.push("#");
  header.push(
    ...toBinary(mustSymbolIndex(spec.allSymbols, spec.blank), symBits),
  );

  // TAPE section marker
  header.push("#");

  return header;
}

/**
 * Construct a TapeOverlay for an encoded UTM tape.
 * The header portion is stored concretely.
 * The tape section is lazily derived from the simulated machine's TapeOverlay:
 * each sim cell at index `cellIdx` maps to (1 + symBits) UTM cells:
 * marker (^ if cellIdx === pos, else ,) followed by symBits binary digits.
 */
function makeEncodeTapeOverlayBackground<SimSymbol extends string>(
  simSpec: TuringMachineSpec<string, SimSymbol>,
  header: readonly MyUtmSymbol[],
  pos: TapeIdx,
  simTape: TapeOverlay<SimSymbol>,
): (i: TapeIdx) => MyUtmSymbol | undefined {
  header = header.slice();
  simTape = simTape.clone();

  const symBits = numBits(simSpec.allSymbols.length);
  const tapeSecStart = header.length;
  const cellSize = 1 + symBits;

  return (idx: TapeIdx): MyUtmSymbol | undefined => {
    if (idx < tapeSecStart) return header[idx];
    const offset = idx - tapeSecStart;
    const cellIdx = Math.floor(offset / cellSize);
    const within = offset % cellSize;
    // If the sim cell is blank (or beyond the overlay), this UTM cell is not in the overlay.
    const simSym = simTape.get(cellIdx);
    if ((simSym ?? simSpec.blank) === simSpec.blank && cellIdx !== pos) {
      return undefined;
    }
    if (within === 0) {
      return cellIdx === pos ? "^" : ",";
    }
    const sym = simSym ?? simSpec.blank;
    const symIdx = must(indexOf(simSpec.allSymbols, sym)) as SymbolIdx;
    const bits = toBinary(symIdx, symBits);
    return bits[within - 1];
  };
}

// ════════════════════════════════════════════════════════════════════
// Decode
// ════════════════════════════════════════════════════════════════════
function decode<SimState extends string, SimSymbol extends string>(
  spec: TuringMachineSpec<SimState, SimSymbol>,
  utm: TuringMachineSnapshot<MyUtmState, MyUtmSymbol>,
  { sparse = true }: { sparse?: boolean } = {},
): undefined | TuringMachineSnapshot<SimState, SimSymbol> {
  if (
    sparse &&
    !(
      (utm.state === "init" && utm.pos <= 1) ||
      utm.state === "accept" ||
      utm.state === "reject"
    )
  ) {
    return undefined;
  }
  if (
    utm.state.startsWith("cp_nst") ||
    utm.state.startsWith("mr_ext") ||
    utm.state.startsWith("cp_nsym")
  ) {
    // cp_nst: while copying the new state bits over the old, the state can be a frankenstein of the old and new states.
    // cp_nsym: while overwriting a symbol, it can be a frankenstein of the old and new symbols.
    // mr_ext: while extending the tape with the blank pattern, it can be a frankenstein of the blank pattern and the UTM's blank symbol.
    return undefined;
  }

  // Clone the UTM tape so the decoded machine is forked from the UTM:
  // reads reflect the UTM state at time of decoding, writes don't propagate back.
  const utmTapeSnapshot = utm.tape.clone();

  const sBits = numBits(spec.allStates.length);
  const symBits = numBits(spec.allSymbols.length);

  // Find section separators (#)
  // Layout: $#RULES#ACC#STATE#BLANK#TAPE → 5 # signs
  const rulesStart = must(tapeIndexOf(utmTapeSnapshot, "#")) + 1;
  const accStart = must(tapeIndexOf(utmTapeSnapshot, "#", rulesStart)) + 1;
  const stateStart = must(tapeIndexOf(utmTapeSnapshot, "#", accStart)) + 1;
  const blankStart = must(tapeIndexOf(utmTapeSnapshot, "#", stateStart)) + 1;
  const tapeSecStart = must(tapeIndexOf(utmTapeSnapshot, "#", blankStart)) + 1;

  // STATE section
  const stateEnd = must(tapeIndexOf(utmTapeSnapshot, "#", stateStart));
  const stateBitsLen = stateEnd - stateStart;
  if (stateBitsLen !== sBits)
    throw new Error(
      `nonsensical decode: state bits length mismatch: ${stateBitsLen} !== ${sBits}`,
    );
  const stIdx = fromBinaryAt(utmTapeSnapshot, stateStart, sBits);
  if (stIdx >= spec.allStates.length)
    throw new Error(
      `nonsensical decode: state index out of bounds: ${stIdx} >= ${spec.allStates.length}`,
    );
  const simState = spec.allStates[stIdx];

  // TAPE section — find the head position by scanning for ^
  // Each cell is (1 + symBits) UTM cells: marker followed by binary symbol.
  const cellSize = 1 + symBits;
  let headPos = 0;
  // Scan for the ^ marker to find head position.
  // Blank sim cells may produce undefined from the UTM tape overlay, so skip those.
  for (let j = tapeSecStart; ; j += cellSize) {
    const marker = utmTapeSnapshot.get(j);
    if (marker === undefined) continue; // blank sim cell — skip
    if (marker === "^" || marker === ">") {
      headPos = (j - tapeSecStart) / cellSize;
      break;
    }
    if (marker !== ",") {
      // Mid-operation marker we don't recognise — best-effort: skip it
      continue;
    }
  }

  // Build a lazy TapeOverlay backed by the cloned UTM tape, with its own writes overlay.
  const decodedTape = makeSimpleTapeOverlay(
    makeDecodedTapeOverlayBackground(
      utmTapeSnapshot,
      spec,
      tapeSecStart,
      cellSize,
      symBits,
    ),
  );

  return {
    spec,
    state: simState,
    tape: decodedTape,
    pos: headPos,
  };
}

function makeDecodedTapeOverlayBackground<SimSymbol extends string>(
  utmTape: TapeOverlay<MyUtmSymbol>,
  spec: { allSymbols: ReadonlyArray<SimSymbol>; blank: SimSymbol },
  tapeSecStart: TapeIdx,
  cellSize: number,
  symBits: number,
): (i: TapeIdx) => SimSymbol | undefined {
  utmTape = utmTape.clone();
  return (cellIdx: TapeIdx): SimSymbol | undefined => {
    const utmIdx = tapeSecStart + cellIdx * cellSize;
    const marker = utmTape.get(utmIdx);
    if (marker === undefined) return undefined;
    if (marker !== "," && marker !== "^" && marker !== ">") return undefined;
    const symIdx = fromBinaryAt(utmTape, utmIdx + 1, symBits);
    if (symIdx >= spec.allSymbols.length) return undefined;
    return spec.allSymbols[symIdx];
  };
}

// ════════════════════════════════════════════════════════════════════
// UTM State Machine Builder
// ════════════════════════════════════════════════════════════════════

type RuleMap = Map<
  MyUtmState,
  Map<MyUtmSymbol, [MyUtmState, MyUtmSymbol, Dir]>
>;

function addRule(
  rules: RuleMap,
  state: MyUtmState,
  sym: MyUtmSymbol,
  newState: MyUtmState,
  newSym: MyUtmSymbol,
  dir: Dir,
) {
  if (!rules.has(state)) rules.set(state, new Map());
  const existing = rules.get(state)!.get(sym);
  if (existing) {
    throw new Error(
      `Duplicate rule: state=${state}, sym=${sym} (existing: ${existing}, new: [${newState}, ${newSym}, ${dir}])`,
    );
  }
  rules.get(state)!.set(sym, [newState, newSym, dir]);
}

function scanRight(
  rules: RuleMap,
  state: MyUtmState,
  syms: readonly MyUtmSymbol[],
) {
  for (const s of syms) {
    addRule(rules, state, s, state, s, "R");
  }
}

function scanLeft(
  rules: RuleMap,
  state: MyUtmState,
  syms: readonly MyUtmSymbol[],
) {
  for (const s of syms) {
    addRule(rules, state, s, state, s, "L");
  }
}

function buildUtmRules(): RuleMap {
  const rules: RuleMap = new Map();

  function st(name: MyUtmState): MyUtmState {
    return name;
  }

  // Ensure accept/reject are registered
  st("accept");
  st("reject");

  // Symbols that appear inside rule bodies (between ; separators)
  const ruleInternals: MyUtmSymbol[] = ["0", "1", "X", "Y", "|", "l", "d"];
  // Everything scannable in the rules section
  const ruleAll: MyUtmSymbol[] = [...ruleInternals, ";", ".", "*"];
  // Data bits that appear in STATE, BLANK, ACCEPTSTATES, cells
  const bits: MyUtmSymbol[] = ["0", "1"];
  const markedBits: MyUtmSymbol[] = ["X", "Y"];
  const bitsAndMarked: MyUtmSymbol[] = ["0", "1", "X", "Y"];

  // Navigate from current position back to $ (left boundary)
  function seekHome(fromState: MyUtmState, toState: MyUtmState) {
    // Scan left past everything until we hit $
    scanLeft(rules, fromState, [
      "0",
      "1",
      "X",
      "Y",
      "#",
      "|",
      ";",
      ",",
      "^",
      ".",
      "*",
      ">",
      "l",
      "d",
    ]);
    addRule(rules, fromState, "$", toState, "$", "R");
  }

  // Navigate from inside rules back to the * rule
  function seekStar(fromState: MyUtmState, targetState: MyUtmState) {
    scanLeft(rules, fromState, [
      "0",
      "1",
      "X",
      "Y",
      "#",
      "|",
      ";",
      ",",
      "^",
      ".",
      "l",
      "d",
    ]);
    addRule(rules, fromState, "*", targetState, "*", "R");
  }

  // ══════════════════════════════════════════════════════════════
  // PHASE 0: INIT
  // ══════════════════════════════════════════════════════════════
  // UTM starts at pos 0 reading $. Move right past $ and first #.
  // Then scan right to the END of the rules section, so we can
  // search rules right-to-left (most-used rules are at the end).
  addRule(rules, st("init"), "$", st("init_skip"), "$", "R");
  addRule(rules, st("init"), "#", st("init_seek_end"), "#", "R");
  addRule(rules, st("init_skip"), "#", st("init_seek_end"), "#", "R");
  // Scan right through rules section to find the # at the end
  scanRight(rules, st("init_seek_end"), [...ruleInternals, ";", "."]);
  // If we immediately hit # → no rules at all → check accept/reject
  addRule(rules, st("init_seek_end"), "#", st("mark_rule"), "#", "L");

  // ══════════════════════════════════════════════════════════════
  // PHASE 1: MARK RULE (right-to-left search)
  // ══════════════════════════════════════════════════════════════
  // Scan LEFT to find next . (inactive rule), mark as *
  scanLeft(rules, st("mark_rule"), ruleInternals);
  addRule(rules, st("mark_rule"), ";", st("mark_rule"), ";", "L");
  addRule(rules, st("mark_rule"), ".", st("cmp_st_read"), "*", "R");
  // Hit # on the left -> no more rules match. Scan right to ACC section.
  addRule(rules, st("mark_rule"), "#", st("mark_rule_no_match"), "#", "R");
  scanRight(rules, st("mark_rule_no_match"), [...ruleInternals, ";", "."]);
  addRule(rules, st("mark_rule_no_match"), "#", st("chk_acc_init"), "#", "R");

  // ══════════════════════════════════════════════════════════════
  // PHASE 2: COMPARE STATE BITS
  // ══════════════════════════════════════════════════════════════
  // We're right after * in the active rule. Read the state bits one by one.
  // Mark bit as X(was 0) or Y(was 1), carry to STATE section, compare.

  // Read current bit
  addRule(rules, st("cmp_st_read"), "0", st("cmp_st_c0"), "X", "R");
  addRule(rules, st("cmp_st_read"), "1", st("cmp_st_c1"), "Y", "R");
  // Hit | -> all state bits compared successfully!
  addRule(rules, st("cmp_st_read"), "|", st("st_match_cleanup"), "|", "R");

  // Carry bit to STATE section (pass 2 # from rules: #ACC#STATE)
  for (const c of ["0", "1"] as const) {
    const carry = st(`cmp_st_c${c}`);
    // Skip rest of rules
    scanRight(rules, carry, ruleAll);
    addRule(rules, carry, "#", st(`cmp_st_c${c}_sk1`), "#", "R");
    // Skip ACCEPTSTATES
    const sk1 = st(`cmp_st_c${c}_sk1`);
    scanRight(rules, sk1, [...bitsAndMarked, ";"]);
    addRule(rules, sk1, "#", st(`cmp_st_c${c}_find`), "#", "R");
    // Find next unmarked bit in STATE
    const find = st(`cmp_st_c${c}_find`);
    scanRight(rules, find, markedBits);
    if (c === "0") {
      addRule(rules, find, "0", st("cmp_st_ok"), "X", "L");
      addRule(rules, find, "1", st("cmp_st_fail"), "Y", "L"); // was 1 → Y
    } else {
      addRule(rules, find, "1", st("cmp_st_ok"), "Y", "L");
      addRule(rules, find, "0", st("cmp_st_fail"), "X", "L"); // was 0 → X
    }
  }

  // Bit matched -> return to * to read next bit
  {
    seekStar(st("cmp_st_ok"), st("cmp_st_nextbit"));
    // Skip past already-marked bits to next unmarked or |
    const nb = st("cmp_st_nextbit");
    scanRight(rules, nb, markedBits);
    addRule(rules, nb, "0", st("cmp_st_c0"), "X", "R");
    addRule(rules, nb, "1", st("cmp_st_c1"), "Y", "R");
    addRule(rules, nb, "|", st("st_match_cleanup"), "|", "R");
  }

  // ══════════════════════════════════════════════════════════════
  // STATE MATCH CLEANUP
  // ══════════════════════════════════════════════════════════════
  // State matched! Restore marks in both rule state field and STATE section.
  // Then proceed to symbol comparison.

  // We're past the | after the state field in the rule.
  // First restore rule state field: go back to * and restore X->0, Y->1
  {
    // We're at the start of the symbol field. Go left to find *
    addRule(
      rules,
      st("st_match_cleanup"),
      "0",
      st("st_match_cleanup"),
      "0",
      "R",
    );
    addRule(
      rules,
      st("st_match_cleanup"),
      "1",
      st("st_match_cleanup"),
      "1",
      "R",
    );
    // Actually we're past | already moving right into sym bits. We need to go left.
    // Let me re-approach. st_match_cleanup is entered when we see | and we're
    // moving right. But we want to go left to restore. Let me go left.
  }
  // Oops, I added conflicting rules. Let me redo this.
  // Remove the wrong rules
  rules.get(st("st_match_cleanup"))?.clear();

  // When we enter st_match_cleanup, we just read | and moved right.
  // So we're at the first bit of the symbol field. But we need to go back left
  // to restore the state field marks. Let me enter a left-seeking state.
  addRule(rules, st("st_match_cleanup"), "0", st("stm_go_left"), "0", "L");
  addRule(rules, st("st_match_cleanup"), "1", st("stm_go_left"), "1", "L");
  // Edge case: | immediately (empty symbol field, shouldn't happen but handle)
  addRule(rules, st("st_match_cleanup"), "|", st("stm_go_left"), "|", "L");

  {
    // Go left past | and marked state bits, restoring them
    const gl = st("stm_go_left");
    addRule(rules, gl, "|", st("stm_restore_rule"), "|", "L");
    scanLeft(rules, gl, bits); // skip any unmarked sym bits
  }
  {
    const rr = st("stm_restore_rule");
    addRule(rules, rr, "X", rr, "0", "L");
    addRule(rules, rr, "Y", rr, "1", "L");
    scanLeft(rules, rr, bits);
    // Hit * -> rule state field restored. Now restore STATE section.
    addRule(rules, rr, "*", st("stm_goto_state"), "*", "R");
  }
  {
    // Navigate to STATE section to restore marks
    // Skip: rule content to end of rules, ACC, then into STATE
    const gs = st("stm_goto_state");
    scanRight(rules, gs, [...ruleInternals, ";", "."]);
    addRule(rules, gs, "#", st("stm_gs_sk1"), "#", "R");
  }
  {
    const sk = st("stm_gs_sk1");
    scanRight(rules, sk, [...bitsAndMarked, ";"]);
    addRule(rules, sk, "#", st("stm_restore_state"), "#", "R");
  }
  {
    const rs = st("stm_restore_state");
    addRule(rules, rs, "X", rs, "0", "R");
    addRule(rules, rs, "Y", rs, "1", "R");
    scanRight(rules, rs, bits);
    // Hit # -> STATE restored. Go back to * for symbol comparison.
    addRule(rules, rs, "#", st("stm_back_to_rule"), "#", "L");
  }
  {
    seekStar(st("stm_back_to_rule"), st("sym_skip_state"));
  }
  {
    // Skip state field to reach | then symbol field
    const ss = st("sym_skip_state");
    scanRight(rules, ss, bits);
    addRule(rules, ss, "|", st("cmp_sym_read"), "|", "R");
  }

  // ══════════════════════════════════════════════════════════════
  // STATE MISMATCH
  // ══════════════════════════════════════════════════════════════
  // Restore marks, deactivate rule, try next rule.
  {
    // We're in STATE section. Go left to start of STATE to restore.
    const sf = st("cmp_st_fail");
    scanLeft(rules, sf, bitsAndMarked);
    addRule(rules, sf, "#", st("stf_restore_state"), "#", "R");
  }
  {
    const rs = st("stf_restore_state");
    addRule(rules, rs, "X", rs, "0", "R");
    addRule(rules, rs, "Y", rs, "1", "R");
    scanRight(rules, rs, bits);
    // Hit # -> STATE restored. Go back to restore rule state field and deactivate.
    addRule(rules, rs, "#", st("stf_find_star"), "#", "L");
  }
  {
    // Seek left to find * in rules
    scanLeft(rules, st("stf_find_star"), [
      ...bitsAndMarked,
      ";",
      "#",
      "|",
      ".",
      "l",
      "d",
    ]);
    addRule(rules, st("stf_find_star"), "*", st("stf_restore_rule"), ".", "R");
  }
  {
    const rr = st("stf_restore_rule");
    addRule(rules, rr, "X", rr, "0", "R");
    addRule(rules, rr, "Y", rr, "1", "R");
    scanRight(rules, rr, bits);
    // Hit | -> rule state field restored. Go left past state field and .
    // to search for previous rule.
    addRule(rules, rr, "|", st("stf_go_prev"), "|", "L");
  }
  {
    const gp = st("stf_go_prev");
    scanLeft(rules, gp, bits);
    // Hit . (current deactivated rule prefix) -> skip it, enter mark_rule
    addRule(rules, gp, ".", st("mark_rule"), ".", "L");
  }

  // ══════════════════════════════════════════════════════════════
  // PHASE 3: COMPARE SYMBOL BITS
  // ══════════════════════════════════════════════════════════════
  // Same pattern as state comparison but compare against head cell on TAPE.
  // From the * rule, skip state field and | to reach symbol field.
  // Navigate to TAPE section to find ^, then compare bits.

  addRule(rules, st("cmp_sym_read"), "0", st("cmp_sym_c0"), "X", "R");
  addRule(rules, st("cmp_sym_read"), "1", st("cmp_sym_c1"), "Y", "R");
  addRule(rules, st("cmp_sym_read"), "|", st("sym_match_cleanup"), "|", "R");

  // Carry symbol bit to head cell
  // Need to pass: rest of rules → #ACC → #STATE → #BLANK → #TAPE → find ^
  for (const c of ["0", "1"] as const) {
    const carry = st(`cmp_sym_c${c}`);
    scanRight(rules, carry, ruleAll);
    addRule(rules, carry, "#", st(`cmp_sym_c${c}_s1`), "#", "R");

    // Skip ACC
    const s1 = st(`cmp_sym_c${c}_s1`);
    scanRight(rules, s1, [...bitsAndMarked, ";"]);
    addRule(rules, s1, "#", st(`cmp_sym_c${c}_s2`), "#", "R");

    // Skip STATE
    const s2 = st(`cmp_sym_c${c}_s2`);
    scanRight(rules, s2, bitsAndMarked);
    addRule(rules, s2, "#", st(`cmp_sym_c${c}_s3`), "#", "R");

    // Skip BLANK
    const s3 = st(`cmp_sym_c${c}_s3`);
    scanRight(rules, s3, bits);
    addRule(rules, s3, "#", st(`cmp_sym_c${c}_fh`), "#", "R");

    // Find ^
    const fh = st(`cmp_sym_c${c}_fh`);
    scanRight(rules, fh, [...bitsAndMarked, ","]);
    addRule(rules, fh, "^", st(`cmp_sym_c${c}_fb`), "^", "R");

    // Find next unmarked bit in head cell
    const fb = st(`cmp_sym_c${c}_fb`);
    scanRight(rules, fb, markedBits);
    if (c === "0") {
      addRule(rules, fb, "0", st("cmp_sym_ok"), "X", "L");
      addRule(rules, fb, "1", st("cmp_sym_fail"), "Y", "L"); // was 1 → Y
    } else {
      addRule(rules, fb, "1", st("cmp_sym_ok"), "Y", "L");
      addRule(rules, fb, "0", st("cmp_sym_fail"), "X", "L"); // was 0 → X
    }
  }

  // Symbol bit matched -> return to * to read next bit
  {
    seekStar(st("cmp_sym_ok"), st("cmp_sym_nextbit"));
    // Skip state field | to reach symbol field
    const nb = st("cmp_sym_nextbit");
    scanRight(rules, nb, bits);
    addRule(rules, nb, "|", st("cmp_sym_nb2"), "|", "R");
  }
  {
    // Now skip past already-marked sym bits
    const nb2 = st("cmp_sym_nb2");
    scanRight(rules, nb2, markedBits);
    addRule(rules, nb2, "0", st("cmp_sym_c0"), "X", "R");
    addRule(rules, nb2, "1", st("cmp_sym_c1"), "Y", "R");
    addRule(rules, nb2, "|", st("sym_match_cleanup"), "|", "R");
  }

  // ══════════════════════════════════════════════════════════════
  // SYMBOL MISMATCH
  // ══════════════════════════════════════════════════════════════
  {
    // Restore head cell marks
    const sf = st("cmp_sym_fail");
    scanLeft(rules, sf, bitsAndMarked);
    addRule(rules, sf, "^", st("symf_rest_head"), "^", "R");
  }
  {
    const rh = st("symf_rest_head");
    addRule(rules, rh, "X", rh, "0", "R");
    addRule(rules, rh, "Y", rh, "1", "R");
    scanRight(rules, rh, bits);
    // After restoring, we hit , or _ or end
    addRule(rules, rh, ",", st("symf_seek_star"), ",", "L");
    addRule(rules, rh, "_", st("symf_seek_star"), "_", "L");
  }
  {
    // Seek back to * to restore sym field and deactivate
    seekStar(st("symf_seek_star"), st("symf_skip_st"));
  }
  {
    // Skip state field
    const ss = st("symf_skip_st");
    scanRight(rules, ss, bits);
    addRule(rules, ss, "|", st("symf_rest_sym"), "|", "R");
  }
  {
    // Restore symbol field marks
    const rs = st("symf_rest_sym");
    addRule(rules, rs, "X", rs, "0", "R");
    addRule(rules, rs, "Y", rs, "1", "R");
    scanRight(rules, rs, bits);
    // Hit | -> sym field restored. Now deactivate rule: go left to *, change to .
    addRule(rules, rs, "|", st("symf_deactivate"), "|", "L");
  }
  {
    // Go left to find *, deactivate, then search left for previous rule
    const da = st("symf_deactivate");
    scanLeft(rules, da, [...bits, "|"]);
    addRule(rules, da, "*", st("mark_rule"), ".", "L");
  }

  // ══════════════════════════════════════════════════════════════
  // SYMBOL MATCH CLEANUP
  // ══════════════════════════════════════════════════════════════
  // Both state and symbol matched! Restore all marks, then apply the rule.
  // We're right after the second | in the rule (entering newstate field).

  {
    // First restore head cell marks
    // Navigate to head cell: we need to go right past rest of rule to TAPE
    const sc = st("sym_match_cleanup");
    // We're in the newstate part of the rule. Skip to end of rules, then to tape.
    scanRight(rules, sc, [...ruleInternals, ";", "."]);
    addRule(rules, sc, "#", st("smc_s1"), "#", "R");
  }
  {
    const s1 = st("smc_s1");
    scanRight(rules, s1, [...bitsAndMarked, ";"]);
    addRule(rules, s1, "#", st("smc_s2"), "#", "R");
  }
  {
    const s2 = st("smc_s2");
    scanRight(rules, s2, bitsAndMarked);
    addRule(rules, s2, "#", st("smc_s3"), "#", "R");
  }
  {
    const s3 = st("smc_s3");
    scanRight(rules, s3, bits);
    addRule(rules, s3, "#", st("smc_fh"), "#", "R");
  }
  {
    const fh = st("smc_fh");
    scanRight(rules, fh, [...bitsAndMarked, ","]);
    addRule(rules, fh, "^", st("smc_rest_head"), "^", "R");
  }
  {
    const rh = st("smc_rest_head");
    addRule(rules, rh, "X", rh, "0", "R");
    addRule(rules, rh, "Y", rh, "1", "R");
    scanRight(rules, rh, bits);
    addRule(rules, rh, ",", st("smc_rest_done"), ",", "L");
    addRule(rules, rh, "_", st("smc_rest_done"), "_", "L");
  }
  {
    // Head cell restored. Now restore rule sym field and go to apply.
    seekStar(st("smc_rest_done"), st("smc_skip_st"));
  }
  {
    const ss = st("smc_skip_st");
    scanRight(rules, ss, bits);
    addRule(rules, ss, "|", st("smc_rest_sym"), "|", "R");
  }
  {
    const rs = st("smc_rest_sym");
    addRule(rules, rs, "X", rs, "0", "R");
    addRule(rules, rs, "Y", rs, "1", "R");
    scanRight(rules, rs, bits);
    addRule(rules, rs, "|", st("apply_read_nst"), "|", "R");
  }

  // ══════════════════════════════════════════════════════════════
  // PHASE 4: APPLY RULE - COPY NEW STATE
  // ══════════════════════════════════════════════════════════════
  // We're at the start of the NEWSTATE field in the active * rule.
  // Copy bits to STATE section.

  addRule(rules, st("apply_read_nst"), "0", st("cp_nst_c0"), "X", "R");
  addRule(rules, st("apply_read_nst"), "1", st("cp_nst_c1"), "Y", "R");
  addRule(rules, st("apply_read_nst"), "|", st("cp_nst_done"), "|", "L");

  for (const c of ["0", "1"] as const) {
    const carry = st(`cp_nst_c${c}`);
    // Skip rest of rules to STATE section (pass 2 #: ACC, STATE)
    scanRight(rules, carry, ruleAll);
    addRule(rules, carry, "#", st(`cp_nst_c${c}_s1`), "#", "R");

    const s1 = st(`cp_nst_c${c}_s1`);
    scanRight(rules, s1, [...bitsAndMarked, ";"]);
    addRule(rules, s1, "#", st(`cp_nst_c${c}_w`), "#", "R");

    // Find next unmarked bit in STATE, overwrite
    const w = st(`cp_nst_c${c}_w`);
    scanRight(rules, w, markedBits);
    // Overwrite whatever is there with our carried bit (marked)
    const mark: MyUtmSymbol = c === "0" ? "X" : "Y";
    addRule(rules, w, "0", st("cp_nst_ret"), mark, "L");
    addRule(rules, w, "1", st("cp_nst_ret"), mark, "L");
  }
  {
    // Return to * to read next NEWSTATE bit
    seekStar(st("cp_nst_ret"), st("cp_nst_next"));
  }
  {
    // Skip: state | sym | to reach newstate, then skip marked bits
    const n = st("cp_nst_next");
    scanRight(rules, n, bits);
    addRule(rules, n, "|", st("cp_nst_next2"), "|", "R");
  }
  {
    const n2 = st("cp_nst_next2");
    scanRight(rules, n2, bits);
    addRule(rules, n2, "|", st("cp_nst_next3"), "|", "R");
  }
  {
    const n3 = st("cp_nst_next3");
    scanRight(rules, n3, markedBits);
    addRule(rules, n3, "0", st("cp_nst_c0"), "X", "R");
    addRule(rules, n3, "1", st("cp_nst_c1"), "Y", "R");
    addRule(rules, n3, "|", st("cp_nst_done"), "|", "L");
  }

  // cp_nst_done: all NEWSTATE bits copied. Restore marks.
  {
    const d = st("cp_nst_done");
    // Restore rule's NEWSTATE field
    addRule(rules, d, "X", d, "0", "L");
    addRule(rules, d, "Y", d, "1", "L");
    // Hit | -> go right past it to navigate to STATE section to restore
    addRule(rules, d, "|", st("cp_nst_rest_nav"), "|", "R");
  }
  {
    // Skip newsym | dir, skip rest of rules to reach STATE
    const nav = st("cp_nst_rest_nav");
    scanRight(rules, nav, [...ruleInternals, ";", "."]);
    addRule(rules, nav, "#", st("cp_nst_rest_s1"), "#", "R");
  }
  {
    const s1 = st("cp_nst_rest_s1");
    scanRight(rules, s1, [...bits, ";"]);
    addRule(rules, s1, "#", st("cp_nst_rest_do"), "#", "R");
  }
  {
    const rd = st("cp_nst_rest_do");
    addRule(rules, rd, "X", rd, "0", "R");
    addRule(rules, rd, "Y", rd, "1", "R");
    scanRight(rules, rd, bits);
    // Hit # -> STATE restored. Go back to rule for NEWSYM copy.
    addRule(rules, rd, "#", st("cp_nsym_seek"), "#", "L");
  }
  {
    // Seek back to * rule
    seekStar(st("cp_nsym_seek"), st("cp_nsym_nav"));
  }

  // ══════════════════════════════════════════════════════════════
  // PHASE 5: COPY NEW SYMBOL
  // ══════════════════════════════════════════════════════════════
  // Skip: state | sym | newstate | to reach NEWSYM
  {
    const n = st("cp_nsym_nav");
    scanRight(rules, n, bits);
    addRule(rules, n, "|", st("cp_nsym_nav2"), "|", "R");
  }
  {
    const n2 = st("cp_nsym_nav2");
    scanRight(rules, n2, bits);
    addRule(rules, n2, "|", st("cp_nsym_nav3"), "|", "R");
  }
  {
    const n3 = st("cp_nsym_nav3");
    scanRight(rules, n3, bits);
    addRule(rules, n3, "|", st("cp_nsym_read"), "|", "R");
  }

  addRule(rules, st("cp_nsym_read"), "0", st("cp_nsym_c0"), "X", "R");
  addRule(rules, st("cp_nsym_read"), "1", st("cp_nsym_c1"), "Y", "R");
  addRule(rules, st("cp_nsym_read"), "|", st("cp_nsym_done"), "|", "L");

  // Carry to head cell: skip rest of rules, ACC, STATE, BLANK, into TAPE, find ^
  for (const c of ["0", "1"] as const) {
    const carry = st(`cp_nsym_c${c}`);
    scanRight(rules, carry, ruleAll);
    addRule(rules, carry, "#", st(`cp_nsym_c${c}_s1`), "#", "R");

    const s1 = st(`cp_nsym_c${c}_s1`);
    scanRight(rules, s1, [...bitsAndMarked, ";"]);
    addRule(rules, s1, "#", st(`cp_nsym_c${c}_s2`), "#", "R");

    const s2 = st(`cp_nsym_c${c}_s2`);
    scanRight(rules, s2, bitsAndMarked);
    addRule(rules, s2, "#", st(`cp_nsym_c${c}_s3`), "#", "R");

    const s3 = st(`cp_nsym_c${c}_s3`);
    scanRight(rules, s3, bits);
    addRule(rules, s3, "#", st(`cp_nsym_c${c}_fh`), "#", "R");

    const fh = st(`cp_nsym_c${c}_fh`);
    scanRight(rules, fh, [...bitsAndMarked, ","]);
    addRule(rules, fh, "^", st(`cp_nsym_c${c}_fb`), "^", "R");

    const fb = st(`cp_nsym_c${c}_fb`);
    scanRight(rules, fb, markedBits);
    const mark: MyUtmSymbol = c === "0" ? "X" : "Y";
    addRule(rules, fb, "0", st("cp_nsym_ret"), mark, "L");
    addRule(rules, fb, "1", st("cp_nsym_ret"), mark, "L");
  }
  {
    seekStar(st("cp_nsym_ret"), st("cp_nsym_fnext"));
  }
  {
    // Skip: state | sym | newstate | then skip marked NEWSYM bits
    const fn = st("cp_nsym_fnext");
    scanRight(rules, fn, bits);
    addRule(rules, fn, "|", st("cp_nsym_fn2"), "|", "R");
  }
  {
    const fn2 = st("cp_nsym_fn2");
    scanRight(rules, fn2, bits);
    addRule(rules, fn2, "|", st("cp_nsym_fn3"), "|", "R");
  }
  {
    const fn3 = st("cp_nsym_fn3");
    scanRight(rules, fn3, bits);
    addRule(rules, fn3, "|", st("cp_nsym_fn4"), "|", "R");
  }
  {
    const fn4 = st("cp_nsym_fn4");
    scanRight(rules, fn4, markedBits);
    addRule(rules, fn4, "0", st("cp_nsym_c0"), "X", "R");
    addRule(rules, fn4, "1", st("cp_nsym_c1"), "Y", "R");
    addRule(rules, fn4, "|", st("cp_nsym_done"), "|", "L");
  }

  // cp_nsym_done: restore newsym field and head cell
  {
    const d = st("cp_nsym_done");
    addRule(rules, d, "X", d, "0", "L");
    addRule(rules, d, "Y", d, "1", "L");
    addRule(rules, d, "|", st("cp_nsym_rest_nav"), "|", "R");
  }
  {
    // Navigate to head cell to restore marks
    const nav = st("cp_nsym_rest_nav");
    scanRight(rules, nav, [...ruleInternals, ";", "."]);
    addRule(rules, nav, "#", st("cp_nsym_rn_s1"), "#", "R");
  }
  {
    const s1 = st("cp_nsym_rn_s1");
    scanRight(rules, s1, [...bits, ";"]);
    addRule(rules, s1, "#", st("cp_nsym_rn_s2"), "#", "R");
  }
  {
    const s2 = st("cp_nsym_rn_s2");
    scanRight(rules, s2, bits);
    addRule(rules, s2, "#", st("cp_nsym_rn_s3"), "#", "R");
  }
  {
    const s3 = st("cp_nsym_rn_s3");
    scanRight(rules, s3, bits);
    addRule(rules, s3, "#", st("cp_nsym_rn_fh"), "#", "R");
  }
  {
    const fh = st("cp_nsym_rn_fh");
    scanRight(rules, fh, [...bits, ","]);
    addRule(rules, fh, "^", st("cp_nsym_rn_do"), "^", "R");
  }
  {
    const rd = st("cp_nsym_rn_do");
    addRule(rules, rd, "X", rd, "0", "R");
    addRule(rules, rd, "Y", rd, "1", "R");
    scanRight(rules, rd, bits);
    // After head cell, we should read dir and move head
    addRule(rules, rd, ",", st("read_dir"), ",", "L");
    addRule(rules, rd, "_", st("read_dir"), "_", "L");
  }

  // ══════════════════════════════════════════════════════════════
  // PHASE 6: READ DIRECTION AND MOVE HEAD
  // ══════════════════════════════════════════════════════════════
  {
    // Go back to * rule to read the direction field
    seekStar(st("read_dir"), st("rd_skip_to_dir"));
  }
  {
    // Skip: state | sym | newstate | newsym | DIR
    const sk = st("rd_skip_to_dir");
    scanRight(rules, sk, bits);
    addRule(rules, sk, "|", st("rd_sk2"), "|", "R");
  }
  {
    const sk2 = st("rd_sk2");
    scanRight(rules, sk2, bits);
    addRule(rules, sk2, "|", st("rd_sk3"), "|", "R");
  }
  {
    const sk3 = st("rd_sk3");
    scanRight(rules, sk3, bits);
    addRule(rules, sk3, "|", st("rd_sk4"), "|", "R");
  }
  {
    const sk4 = st("rd_sk4");
    scanRight(rules, sk4, bits);
    addRule(rules, sk4, "|", st("rd_read"), "|", "R");
  }
  {
    addRule(rules, st("rd_read"), "l", st("move_left"), "l", "L");
    addRule(rules, st("rd_read"), "d", st("move_right"), "d", "L");
  }

  // ══════════════════════════════════════════════════════════════
  // DEACTIVATE RULE AND NAVIGATE TO TAPE
  // ══════════════════════════════════════════════════════════════
  // Before moving the head, deactivate the * rule (change to .)
  // and navigate to the tape section.

  // MOVE RIGHT
  {
    // First deactivate: go left to find *
    const mr = st("move_right");
    scanLeft(rules, mr, [...bits, "|", "l", "d"]);
    addRule(rules, mr, "*", st("mr_nav"), ".", "R");
  }
  {
    // Navigate to tape: skip rest of rules, ACC, STATE, BLANK
    const nav = st("mr_nav");
    scanRight(rules, nav, [...ruleInternals, ";", "."]);
    addRule(rules, nav, "#", st("mr_s1"), "#", "R");
  }
  {
    const s1 = st("mr_s1");
    scanRight(rules, s1, [...bits, ";"]);
    addRule(rules, s1, "#", st("mr_s2"), "#", "R");
  }
  {
    const s2 = st("mr_s2");
    scanRight(rules, s2, bits);
    addRule(rules, s2, "#", st("mr_s3"), "#", "R");
  }
  {
    const s3 = st("mr_s3");
    scanRight(rules, s3, bits);
    addRule(rules, s3, "#", st("mr_find_head"), "#", "R");
  }
  {
    // Find ^
    const fh = st("mr_find_head");
    scanRight(rules, fh, [...bits, ","]);
    addRule(rules, fh, "^", st("mr_skip_cell"), ">", "R");
  }
  {
    // Skip current cell bits
    const sc = st("mr_skip_cell");
    scanRight(rules, sc, bits);
    // Hit , -> next cell exists. Change , to ^, go back to change > to ,
    addRule(rules, sc, ",", st("mr_place_head"), "^", "L");
    // Hit _ -> need to extend tape (no more cells)
    addRule(rules, sc, "_", st("mr_extend_init"), "_", "L");
  }
  {
    // Go back to >, change to ,
    const ph = st("mr_place_head");
    scanLeft(rules, ph, bits);
    addRule(rules, ph, ">", st("done_seek_home"), ",", "L");
  }

  // EXTEND TAPE (move right past end)
  // We need to create a new cell with the blank symbol encoding.
  // The blank encoding is stored in the BLANK section.
  // Strategy: go back to BLANK section, read bits one by one, carry to tape end.
  {
    // We're at the last bit of the last cell. Go left to find > first.
    const ei = st("mr_extend_init");
    scanLeft(rules, ei, bits);
    addRule(rules, ei, ">", st("mr_ext_to_blank"), ",", "R");
  }
  {
    // Now we need to: put ^ at current position (after last cell), then write blank bits.
    // But first we need to figure out where the new cell starts.
    // Currently: ,LASTCELL (no more cells). We changed > to , so head goes back.
    // We need to go right past LASTCELL to the first _, write ^, then copy BLANK bits.
    const tb = st("mr_ext_to_blank");
    scanRight(rules, tb, bits);
    // Hit _ -> this is where the new cell starts
    addRule(rules, tb, "_", st("mr_ext_write_head"), "^", "L");
  }
  {
    // Now go back to BLANK section to read blank bits.
    // Seek left to $ then right to find BLANK section.
    // Actually, let's seek left past # signs to count to BLANK section.
    // BLANK is between 5th and 6th # from the left.
    // Or: seek left past tape cells, past # (before TAPE), past BLANK, and we overshoot.
    // Let me just seek home and count # from the left.
    seekHome(st("mr_ext_write_head"), st("mr_ext_home"));
  }
  {
    // From $, skip to BLANK section: $#RULES#ACC#STATE#BLANK
    // That's 4 # signs to skip
    const eh = st("mr_ext_home");
    addRule(rules, eh, "#", st("mr_ext_h1"), "#", "R");
  }
  {
    // Skip RULES
    const h1 = st("mr_ext_h1");
    scanRight(rules, h1, [...ruleInternals, ";", "."]);
    addRule(rules, h1, "#", st("mr_ext_h2"), "#", "R");
  }
  {
    // Skip ACC
    const h2 = st("mr_ext_h2");
    scanRight(rules, h2, [...bits, ";"]);
    addRule(rules, h2, "#", st("mr_ext_h3"), "#", "R");
  }
  {
    // Skip STATE
    const h3 = st("mr_ext_h3");
    scanRight(rules, h3, bits);
    addRule(rules, h3, "#", st("mr_ext_read_blank"), "#", "R");
  }
  {
    // Read BLANK bits one by one, mark, carry to end of tape
    const rb = st("mr_ext_read_blank");
    addRule(rules, rb, "0", st("mr_ext_bc0"), "X", "R");
    addRule(rules, rb, "1", st("mr_ext_bc1"), "Y", "R");
    // Hit # -> all blank bits copied, we're done
    addRule(rules, rb, "#", st("mr_ext_rest_blank"), "#", "L");
  }
  for (const c of ["0", "1"] as const) {
    // Carry blank bit to end of tape
    const carry = st(`mr_ext_bc${c}`);
    scanRight(rules, carry, [...bits, "#", ",", "^"]);
    // Hit _ -> write the bit here
    addRule(rules, carry, "_", st("mr_ext_bc_ret"), c as MyUtmSymbol, "L");
  }
  {
    // Go back to BLANK section to read next bit
    const ret = st("mr_ext_bc_ret");
    scanLeft(rules, ret, [...bits, "#", ",", "^"]);
    // Hit X or Y -> we're back in BLANK section
    addRule(rules, ret, "X", st("mr_ext_bc_next"), "X", "R");
    addRule(rules, ret, "Y", st("mr_ext_bc_next"), "Y", "R");
  }
  {
    // Find next unmarked BLANK bit
    const next = st("mr_ext_bc_next");
    scanRight(rules, next, markedBits);
    addRule(rules, next, "0", st("mr_ext_bc0"), "X", "R");
    addRule(rules, next, "1", st("mr_ext_bc1"), "Y", "R");
    addRule(rules, next, "#", st("mr_ext_rest_blank"), "#", "L");
  }
  {
    // Restore BLANK section marks
    const rb = st("mr_ext_rest_blank");
    addRule(rules, rb, "X", rb, "0", "L");
    addRule(rules, rb, "Y", rb, "1", "L");
    scanLeft(rules, rb, bits);
    // Hit # -> BLANK restored. Seek home.
    addRule(rules, rb, "#", st("done_seek_home"), "#", "L");
  }

  // MOVE LEFT
  {
    // Deactivate rule first
    const ml = st("move_left");
    scanLeft(rules, ml, [...bits, "|", "l", "d"]);
    addRule(rules, ml, "*", st("ml_nav"), ".", "R");
  }
  {
    // Navigate to tape
    const nav = st("ml_nav");
    scanRight(rules, nav, [...ruleInternals, ";", "."]);
    addRule(rules, nav, "#", st("ml_s1"), "#", "R");
  }
  {
    const s1 = st("ml_s1");
    scanRight(rules, s1, [...bits, ";"]);
    addRule(rules, s1, "#", st("ml_s2"), "#", "R");
  }
  {
    const s2 = st("ml_s2");
    scanRight(rules, s2, bits);
    addRule(rules, s2, "#", st("ml_s3"), "#", "R");
  }
  {
    const s3 = st("ml_s3");
    scanRight(rules, s3, bits);
    addRule(rules, s3, "#", st("ml_find_head"), "#", "R");
  }
  {
    // Find ^
    const fh = st("ml_find_head");
    scanRight(rules, fh, [...bits, ","]);
    addRule(rules, fh, "^", st("ml_mark"), ">", "L");
  }
  {
    // Go left past previous cell's bits to find its , marker
    const mk = st("ml_mark");
    scanLeft(rules, mk, bits);
    addRule(rules, mk, ",", st("ml_restore"), "^", "R");
    // # would mean we're at tape start - can't go left (shouldn't happen)
  }
  {
    // Go right to find > and change back to ,
    const rs = st("ml_restore");
    scanRight(rules, rs, bits);
    addRule(rules, rs, ">", st("done_seek_home"), ",", "L");
  }

  // ══════════════════════════════════════════════════════════════
  // PHASE 7: SEEK HOME AND RESTART
  // ══════════════════════════════════════════════════════════════
  seekHome(st("done_seek_home"), st("init"));

  // ══════════════════════════════════════════════════════════════
  // PHASE 8: CHECK ACCEPT STATES
  // ══════════════════════════════════════════════════════════════
  // No rule matched. Compare STATE against each entry in ACCEPTSTATES.
  // If match found -> accept. If exhausted -> reject.
  //
  // chk_acc_init: we just passed the # after RULES into ACCEPTSTATES section.
  // We need to read ACCEPTSTATES entries and compare each against STATE.

  {
    // Check if ACCEPTSTATES is empty (immediate #)
    const ci = st("chk_acc_init");
    // If # -> no accept states -> reject (no marks to restore)
    addRule(rules, ci, "#", st("rej_final_home"), "#", "L");
    // Otherwise, start comparing first accept state entry
    addRule(rules, ci, "0", st("chk_acc_c0"), "X", "R");
    addRule(rules, ci, "1", st("chk_acc_c1"), "Y", "R");
  }

  // Carry accept state bit to STATE section for comparison
  for (const c of ["0", "1"] as const) {
    const carry = st(`chk_acc_c${c}`);
    // Skip remaining accept state bits and other entries to reach STATE
    scanRight(rules, carry, [...bitsAndMarked, ";"]);
    addRule(rules, carry, "#", st(`chk_acc_c${c}_find`), "#", "R");

    const find = st(`chk_acc_c${c}_find`);
    scanRight(rules, find, markedBits);
    if (c === "0") {
      addRule(rules, find, "0", st("chk_acc_ok"), "X", "L");
      addRule(rules, find, "1", st("chk_acc_fail_bit"), "Y", "L"); // was 1 → Y
    } else {
      addRule(rules, find, "1", st("chk_acc_ok"), "Y", "L");
      addRule(rules, find, "0", st("chk_acc_fail_bit"), "X", "L"); // was 0 → X
    }
  }

  // Bit matched -> go back for next bit
  {
    const ok = st("chk_acc_ok");
    // Go left from STATE section. We pass STATE bits, hit # between ACC and STATE.
    scanLeft(rules, ok, bitsAndMarked);
    // Hit # between ACC and STATE -> continue left through ACC
    addRule(rules, ok, "#", st("chk_acc_ok_acc"), "#", "L");
  }
  {
    // Scan left through ACC to find # before ACC
    const oa = st("chk_acc_ok_acc");
    scanLeft(rules, oa, [...bitsAndMarked, ";"]);
    addRule(rules, oa, "#", st("chk_acc_ok_find"), "#", "R");
  }
  {
    // Find the current (partially marked) entry in ACC.
    // Skip past restored entries (all 0/1) and ; separators.
    // The current entry has X/Y marks.
    const of_ = st("chk_acc_ok_find");
    // Skip past restored entries: scan right past 0, 1, ;
    scanRight(rules, of_, [...bits, ";"]);
    // When we hit X or Y, we're in the current entry → skip past already-compared marks
    addRule(rules, of_, "X", st("chk_acc_ok_skip"), "X", "R");
    addRule(rules, of_, "Y", st("chk_acc_ok_skip"), "Y", "R");
    // If we hit # → all entries checked (shouldn't happen in this path but be safe)
    addRule(rules, of_, "#", st("accept_seek_home"), "#", "L");
  }
  {
    const skip = st("chk_acc_ok_skip");
    scanRight(rules, skip, markedBits);
    addRule(rules, skip, "0", st("chk_acc_c0"), "X", "R");
    addRule(rules, skip, "1", st("chk_acc_c1"), "Y", "R");
    // Hit ; or # -> all bits matched! This is an accept state -> accept
    addRule(rules, skip, ";", st("accept_seek_home"), ";", "L");
    addRule(rules, skip, "#", st("accept_seek_home"), "#", "L");
  }

  // Bit mismatch -> restore STATE marks, restore acc entry marks, try next entry
  {
    const fb = st("chk_acc_fail_bit");
    // We're in STATE section (just marked a bit that didn't match).
    // Go left to # between ACC and STATE.
    scanLeft(rules, fb, bitsAndMarked);
    addRule(rules, fb, "#", st("chk_acc_rest_state"), "#", "R");
  }
  {
    // Restore STATE marks
    const rs = st("chk_acc_rest_state");
    addRule(rules, rs, "X", rs, "0", "R");
    addRule(rules, rs, "Y", rs, "1", "R");
    scanRight(rules, rs, bits);
    // Hit # after STATE -> go back left to ACC
    addRule(rules, rs, "#", st("chk_acc_back2acc"), "#", "L");
  }
  {
    // Go left through STATE (restored, all 0/1), past # (ACC/STATE), into ACC
    const ba = st("chk_acc_back2acc");
    scanLeft(rules, ba, bits);
    // Hit # between ACC and STATE -> continue left into ACC
    addRule(rules, ba, "#", st("chk_acc_into_acc"), "#", "L");
  }
  {
    // Go left through ACC to # before ACC (RULES/ACC separator)
    const ia = st("chk_acc_into_acc");
    scanLeft(rules, ia, [...bitsAndMarked, ";"]);
    addRule(rules, ia, "#", st("chk_acc_do_rest"), "#", "R");
  }
  {
    // Find and restore the marked entry
    const dr = st("chk_acc_do_rest");
    scanRight(rules, dr, bits);
    addRule(rules, dr, "X", st("chk_acc_do_rest2"), "0", "R");
    addRule(rules, dr, "Y", st("chk_acc_do_rest2"), "1", "R");
    // If we hit ; or # without finding marks, the entry was already clean
    addRule(rules, dr, ";", st("chk_acc_next_entry"), ";", "R");
    addRule(rules, dr, "#", st("reject_seek_home"), "#", "L");
  }
  {
    const dr2 = st("chk_acc_do_rest2");
    addRule(rules, dr2, "X", dr2, "0", "R");
    addRule(rules, dr2, "Y", dr2, "1", "R");
    scanRight(rules, dr2, bits);
    // Hit ; -> entry restored, try next
    addRule(rules, dr2, ";", st("chk_acc_next_entry"), ";", "R");
    // Hit # -> entry restored, no more entries -> reject
    addRule(rules, dr2, "#", st("reject_seek_home"), "#", "L");
  }
  {
    // Try next accept state entry
    const ne = st("chk_acc_next_entry");
    addRule(rules, ne, "0", st("chk_acc_c0"), "X", "R");
    addRule(rules, ne, "1", st("chk_acc_c1"), "Y", "R");
    // # -> no more entries
    addRule(rules, ne, "#", st("reject_seek_home"), "#", "L");
  }

  // Accept: restore marks in ACCEPTSTATES and STATE, then seek home
  // When we enter accept_seek_home, we're in ACCEPTSTATES area with X/Y marks
  // in both the current entry and STATE section.
  {
    // Go left to find start of ACCEPTSTATES section (the # before it)
    const ash = st("accept_seek_home");
    scanLeft(rules, ash, [...bitsAndMarked, ";"]);
    addRule(rules, ash, "#", st("acc_rest_acc"), "#", "R");
  }
  {
    // Restore all X/Y in ACCEPTSTATES
    const ra = st("acc_rest_acc");
    addRule(rules, ra, "X", ra, "0", "R");
    addRule(rules, ra, "Y", ra, "1", "R");
    scanRight(rules, ra, [...bits, ";"]);
    // Hit # -> end of ACC, now restore STATE
    addRule(rules, ra, "#", st("acc_rest_state"), "#", "R");
  }
  {
    const rs = st("acc_rest_state");
    addRule(rules, rs, "X", rs, "0", "R");
    addRule(rules, rs, "Y", rs, "1", "R");
    scanRight(rules, rs, bits);
    addRule(rules, rs, "#", st("acc_final_home"), "#", "L");
  }
  seekHome(st("acc_final_home"), st("accept"));

  // Reject: similarly restore marks
  // When we enter reject_seek_home, we might have marks in ACC and/or STATE.
  // But for reject, the STATE should already be restored (mismatch path does it).
  // However, we might have partial marks from the last comparison. Let's be safe.
  {
    // For reject, we may be anywhere. Let's scan left to ACC section start.
    const rsh = st("reject_seek_home");
    scanLeft(rules, rsh, [...bitsAndMarked, ";"]);
    addRule(rules, rsh, "#", st("rej_rest_acc"), "#", "R");
  }
  {
    const ra = st("rej_rest_acc");
    addRule(rules, ra, "X", ra, "0", "R");
    addRule(rules, ra, "Y", ra, "1", "R");
    scanRight(rules, ra, [...bits, ";"]);
    addRule(rules, ra, "#", st("rej_rest_state"), "#", "R");
  }
  {
    const rs = st("rej_rest_state");
    addRule(rules, rs, "X", rs, "0", "R");
    addRule(rules, rs, "Y", rs, "1", "R");
    scanRight(rules, rs, bits);
    addRule(rules, rs, "#", st("rej_final_home"), "#", "L");
  }
  seekHome(st("rej_final_home"), st("reject"));

  return rules;
}

// ════════════════════════════════════════════════════════════════════
// Build and export
// ════════════════════════════════════════════════════════════════════

const rules = buildUtmRules();

const allStates = [
  "acc_final_home",
  "acc_rest_acc",
  "acc_rest_state",
  "accept",
  "accept_seek_home",
  "chk_acc_back2acc",
  "chk_acc_c0",
  "chk_acc_c0_find",
  "chk_acc_c1",
  "chk_acc_c1_find",
  "chk_acc_do_rest",
  "chk_acc_do_rest2",
  "chk_acc_fail_bit",
  "chk_acc_init",
  "chk_acc_into_acc",
  "chk_acc_next_entry",
  "chk_acc_ok",
  "chk_acc_ok_acc",
  "chk_acc_ok_find",
  "chk_acc_ok_skip",
  "chk_acc_rest_state",
  "mark_rule_no_match",
  "ml_find_head",
  "ml_mark",
  "ml_nav",
  "ml_restore",
  "ml_s1",
  "ml_s2",
  "ml_s3",
  "move_left",
  "mr_ext_bc_next",
  "mr_ext_bc_ret",
  "mr_ext_bc0",
  "mr_ext_bc1",
  "mr_ext_h1",
  "mr_ext_h2",
  "mr_ext_h3",
  "mr_ext_home",
  "mr_ext_read_blank",
  "mr_ext_rest_blank",
  "mr_ext_to_blank",
  "mr_ext_write_head",
  "mr_extend_init",
  "rej_final_home",
  "rej_rest_acc",
  "rej_rest_state",
  "reject",
  "reject_seek_home",
  "stf_skip_rest",
  "symf_skip_rest",
  "init_skip",
  "apply_read_nst",
  "cp_nsym_read",
  "init",
  "rd_read",
  "cp_nsym_c1_fb",
  "cp_nsym_done",
  "cp_nsym_nav2",
  "cp_nsym_rn_do",
  "cp_nsym_rn_s3",
  "mr_place_head",
  "mr_s3",
  "mr_skip_cell",
  "rd_sk2",
  "rd_sk4",
  "smc_rest_head",
  "smc_rest_sym",
  "smc_s3",
  "cp_nsym_c0_fb",
  "cp_nsym_c1_s3",
  "cmp_sym_read",
  "st_match_cleanup",
  "stm_go_left",
  "cp_nst_done",
  "cp_nst_rest_do",
  "cp_nst_rest_s1",
  "cp_nsym_nav",
  "cp_nsym_nav3",
  "cp_nsym_rn_s1",
  "cp_nsym_rn_s2",
  "mr_s1",
  "mr_s2",
  "rd_sk3",
  "rd_skip_to_dir",
  "smc_s1",
  "smc_s2",
  "smc_skip_st",
  "cp_nsym_c1_s1",
  "cp_nsym_c1_s2",
  "cmp_sym_c0_fb",
  "cp_nsym_c0_s3",
  "cmp_sym_fail",
  "cp_nsym_fn4",
  "cp_nst_c1_w",
  "cp_nst_c0_w",
  "cmp_sym_c1_fb",
  "cp_nsym_fn2",
  "cp_nsym_c0_s1",
  "cp_nsym_c0_s2",
  "move_right",
  "cmp_sym_nb2",
  "cp_nst_c1_s1",
  "cp_nsym_fn3",
  "cp_nsym_fnext",
  "cp_nst_c0_s1",
  "symf_rest_head",
  "symf_rest_sym",
  "cp_nst_next2",
  "cmp_sym_c0_s3",
  "cp_nst_next3",
  "cmp_sym_c1_s3",
  "symf_skip_st",
  "cp_nst_next",
  "stm_gs_sk1",
  "stm_restore_rule",
  "stm_restore_state",
  "sym_skip_state",
  "cmp_sym_c0_s1",
  "cmp_sym_c0_s2",
  "cmp_sym_c1_s1",
  "cmp_sym_c1_s2",
  "cmp_sym_nextbit",
  "symf_deactivate",
  "cmp_st_read",
  "cmp_st_fail",
  "cmp_st_c0_find",
  "stf_go_prev",
  "stf_restore_rule",
  "stf_restore_state",
  "cmp_st_c1_find",
  "cmp_st_nextbit",
  "cmp_st_c0_sk1",
  "cp_nsym_rn_fh",
  "mr_find_head",
  "smc_fh",
  "cp_nsym_c1_fh",
  "cmp_st_c1_sk1",
  "cp_nsym_rest_nav",
  "cp_nst_rest_nav",
  "sym_match_cleanup",
  "mark_rule",
  "mr_nav",
  "cp_nsym_seek",
  "cp_nsym_c1",
  "cp_nsym_c0_fh",
  "read_dir",
  "smc_rest_done",
  "cp_nsym_c0",
  "cp_nst_c1",
  "cp_nst_c0",
  "cmp_sym_c0_fh",
  "cmp_sym_c1_fh",
  "init_seek_end",
  "cp_nsym_ret",
  "done_seek_home",
  "cp_nst_ret",
  "stm_goto_state",
  "stm_back_to_rule",
  "cmp_sym_c0",
  "cmp_sym_c1",
  "symf_seek_star",
  "cmp_sym_ok",
  "stf_find_star",
  "cmp_st_c0",
  "cmp_st_c1",
  "cmp_st_ok",
] as const;

export type MyUtmState = (typeof allStates)[number];

export class MyUtmSnapshot<
  SimState extends string,
  SimSymbol extends string,
> implements UtmSnapshot<MyUtmState, MyUtmSymbol, SimState, SimSymbol> {
  pos: TapeIdx;
  state: MyUtmState;
  tape: TapeOverlay<MyUtmSymbol>;
  simSpec: TuringMachineSpec<SimState, SimSymbol>;

  constructor({
    pos,
    state,
    tape,
    simSpec,
  }: {
    pos: TapeIdx;
    state: MyUtmState;
    tape: TapeOverlay<MyUtmSymbol>;
    simSpec: TuringMachineSpec<SimState, SimSymbol>;
  }) {
    this.pos = pos;
    this.state = state;
    this.tape = tape.clone();
    this.simSpec = simSpec;
  }

  static fromSimSnapshot<SimState extends string, SimSymbol extends string>(
    simSnapshot: TuringMachineSnapshot<SimState, SimSymbol>,
    {
      optimizationHints = [],
    }: { optimizationHints?: Array<[SimState, SimSymbol]> } = {},
  ): MyUtmSnapshot<SimState, SimSymbol> {
    return new MyUtmSnapshot({
      pos: 0,
      state: myUtmSpec.initial,
      tape: encodeToTape(simSnapshot, optimizationHints),
      simSpec: simSnapshot.spec,
    });
  }

  get spec() {
    return myUtmSpec;
  }

  decode(optimizationHints?: {
    sparse?: boolean;
  }): undefined | TuringMachineSnapshot<SimState, SimSymbol> {
    return decode(this.simSpec, this, optimizationHints);
  }
}

export const myUtmSpec: UtmSpec<MyUtmState, MyUtmSymbol> = {
  allStates,
  allSymbols,
  initial: "init",
  blank: "_",
  acceptingStates: new Set<MyUtmState>(["accept"]),
  rules,

  encode: (snap, opts) => MyUtmSnapshot.fromSimSnapshot(snap, opts),
};
