import { type Dir, type TuringMachineSpec } from "./types";

export const write1sForeverSpec: TuringMachineSpec<"init", "_" | "1"> = {
  allStates: ["init"] as const,
  allSymbols: ["_", "1"] as const,
  initial: "init",
  blank: "_",
  acceptingStates: new Set(["init"]),
  rules: new Map([
    [
      "init",
      new Map([
        ["_", ["init", "1", "R"]],
        ["1", ["init", "1", "R"]],
      ]),
    ],
  ]),
};

export const acceptImmediatelySpec: TuringMachineSpec<"init", "_"> = {
  allStates: ["init"] as const,
  allSymbols: ["_"] as const,
  initial: "init",
  blank: "_",
  acceptingStates: new Set(["init"]),
  rules: new Map([]),
};

export const rejectImmediatelySpec: TuringMachineSpec<"init", "_"> = {
  allStates: ["init"] as const,
  allSymbols: ["_"] as const,
  initial: "init",
  acceptingStates: new Set([]),
  blank: "_",
  rules: new Map(),
};

export const flipBitsSpec: TuringMachineSpec<"init", "_" | "0" | "1"> = {
  allStates: ["init"] as const,
  allSymbols: ["_", "0", "1"] as const,
  initial: "init",
  blank: "_",
  acceptingStates: new Set(["init"]),
  rules: new Map([
    [
      "init",
      new Map([
        ["0", ["init", "1", "R"]],
        ["1", ["init", "0", "R"]],
      ]),
    ],
  ]),
};

type Letter = "a" | "b" | "c";

type PalindromeSymbol = "_" | Letter;
type PalindromeState =
  | "start"
  | "accept"
  | "seekL"
  | `seekR_${Letter}`
  | `check_${Letter}`;

export const checkPalindromeSpec = ((): TuringMachineSpec<
  PalindromeState,
  PalindromeSymbol
> => {
  // The classic approach: repeatedly match & erase the outermost pair of characters.
  //
  // States:
  //   start     – read (and erase) the leftmost symbol
  //   seekR_x   – scan right to find the end, carrying letter x
  //   check_x   – at the rightmost symbol, verify it matches x
  //   seekL     – scan left back to the start
  //   accept    – halting accept state

  const letters = ["a", "b", "c"] as const;

  const allSymbols: readonly PalindromeSymbol[] = ["_", ...letters];

  const allStates: readonly PalindromeState[] = [
    "start",
    "accept",
    "seekL",
    ...letters.map((l): `seekR_${Letter}` => `seekR_${l}`),
    ...letters.map((l): `check_${Letter}` => `check_${l}`),
  ];

  function buildRules(): ReadonlyMap<
    PalindromeState,
    ReadonlyMap<PalindromeSymbol, [PalindromeState, PalindromeSymbol, Dir]>
  > {
    const rules = new Map<
      PalindromeState,
      Map<PalindromeSymbol, [PalindromeState, PalindromeSymbol, Dir]>
    >();

    // ── start: read leftmost symbol, erase it, seek right ──
    const startRules = new Map<
      PalindromeSymbol,
      [PalindromeState, PalindromeSymbol, Dir]
    >();
    startRules.set("_", ["accept", "_", "R"]); // empty tape → palindrome
    for (const l of letters) {
      startRules.set(l, [`seekR_${l}`, "_", "R"]); // erase & remember
    }
    rules.set("start", startRules);

    // ── seekR_x: scan right past all letters until blank ──
    for (const l of letters) {
      const m = new Map<
        PalindromeSymbol,
        [PalindromeState, PalindromeSymbol, Dir]
      >();
      m.set("_", [`check_${l}`, "_", "L"]); // hit the end, step back
      for (const l2 of letters) {
        m.set(l2, [`seekR_${l}`, l2, "R"]); // keep scanning
      }
      rules.set(`seekR_${l}`, m);
    }

    // ── check_x: verify rightmost symbol matches x ──
    for (const l of letters) {
      const m = new Map<
        PalindromeSymbol,
        [PalindromeState, PalindromeSymbol, Dir]
      >();
      m.set("_", ["accept", "_", "R"]); // single char was left → ok
      m.set(l, ["seekL", "_", "L"]); // match → erase & go back
      // any other letter → no rule → halt & reject
      rules.set(`check_${l}`, m);
    }

    // ── seekL: scan left back to the erased region ──
    const seekLRules = new Map<
      PalindromeSymbol,
      [PalindromeState, PalindromeSymbol, Dir]
    >();
    seekLRules.set("_", ["start", "_", "R"]); // reached left edge, restart
    for (const l of letters) {
      seekLRules.set(l, ["seekL", l, "L"]); // keep scanning
    }
    rules.set("seekL", seekLRules);

    return rules;
  }

  return {
    allStates,
    allSymbols,
    initial: "start",
    blank: "_",
    acceptingStates: new Set<PalindromeState>(["accept"]),
    rules: buildRules(),
  };
})();

export type DoubleXSymbol = "_" | "$" | "X" | "Y" | "Z";
export type DoubleXState =
  | "start"
  | "findX"
  | "goRight"
  | "goBack"
  | "cleanL"
  | "cleanR"
  | "done";

export const doubleXSpec: TuringMachineSpec<DoubleXState, DoubleXSymbol> =
  (() => {
    // Doubles a string of X's preceded by $.
    // Algorithm:
    //   1. For each X, mark it as Y and append a Z at the right end.
    //   2. When no unmarked X's remain, convert all Y's and Z's back to X's.
    //
    // States:
    //   start   – skip past $
    //   findX   – find leftmost unmarked X
    //   goRight – scan right to append a Z
    //   goBack  – return left to find next X
    //   cleanL  – convert Y→X going left
    //   cleanR  – convert Z→X going right
    //   done    – halting accept state

    type S = DoubleXSymbol;
    type Q = DoubleXState;
    type Dir = "L" | "R";
    type Rule = [Q, S, Dir];

    const rules = new Map<Q, Map<S, Rule>>();

    rules.set("start", new Map<S, Rule>([["$", ["findX", "$", "R"]]]));

    rules.set(
      "findX",
      new Map<S, Rule>([
        ["X", ["goRight", "Y", "R"]], // mark & go append
        ["Y", ["findX", "Y", "R"]], // skip marked
        ["Z", ["cleanL", "Z", "L"]], // all X's marked, clean up
        ["_", ["done", "_", "L"]], // no X's at all (0 case)
      ]),
    );

    rules.set(
      "goRight",
      new Map<S, Rule>([
        ["X", ["goRight", "X", "R"]], // skip remaining X's
        ["Z", ["goRight", "Z", "R"]], // skip already-appended Z's
        ["_", ["goBack", "Z", "L"]], // append Z, head back
      ]),
    );

    rules.set(
      "goBack",
      new Map<S, Rule>([
        ["X", ["goBack", "X", "L"]],
        ["Y", ["goBack", "Y", "L"]],
        ["Z", ["goBack", "Z", "L"]],
        ["$", ["findX", "$", "R"]], // back at start, find next X
      ]),
    );

    rules.set(
      "cleanL",
      new Map<S, Rule>([
        ["Y", ["cleanL", "X", "L"]], // convert Y→X
        ["$", ["cleanR", "$", "R"]], // reached $, now go right
      ]),
    );

    rules.set(
      "cleanR",
      new Map<S, Rule>([
        ["X", ["cleanR", "X", "R"]], // skip converted X's
        ["Z", ["cleanR", "X", "R"]], // convert Z→X
        ["_", ["done", "_", "L"]], // finished
      ]),
    );

    return {
      allStates: [
        "start",
        "findX",
        "goRight",
        "goBack",
        "cleanL",
        "cleanR",
        "done",
      ],
      allSymbols: ["_", "$", "X", "Y", "Z"],
      initial: "start",
      blank: "_",
      acceptingStates: new Set<Q>(["done"]),
      rules,
    };
  })();
