import type { UtmSpec } from "./types";

const allStates = ['TODO'] as const;
export type UtmState = (typeof allStates)[number];

const allSymbols = ['TODO'] as const;
export type UtmSymbol = (typeof allSymbols)[number];

export const utmSpec: UtmSpec<UtmState, UtmSymbol> = {
  allStates,
  allSymbols,
  initial: 'TODO',
  accept: 'TODO',
  reject: 'TODO',
  blank: 'TODO',
  rules: {TODO: {TODO: ['TODO', 'TODO', 'R']}},

  encode(snapshot) {
    throw new Error('Not implemented');
  },

  decode(spec, tape) {
    throw new Error('Not implemented');
  },
}
