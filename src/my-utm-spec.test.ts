import type { UtmSpec } from "./types";

const allStates = ["TODO"] as const;
export type MyUtmState = (typeof allStates)[number];

const allSymbols = ["TODO"] as const;
export type MyUtmSymbol = (typeof allSymbols)[number];

// This UTM specification should be **fully generic** and work for *any* TuringMachineSpec.
// Even itself!
export const myUtmSpec: UtmSpec<MyUtmState, MyUtmSymbol> = {
  allStates,
  allSymbols,
  initial: "TODO",
  blank: "TODO",
  rules: { TODO: { TODO: { type: "accept" } } },

  encode(snapshot) {
    throw new Error("Not implemented");
  },

  decode(spec, utm) {
    throw new Error("Not implemented");
  },
};
