import type { TuringMachineSnapshot } from "./types";

export function tapesEqual<Tape extends readonly string[]>(a: Tape, b: Tape, blank: Tape[number]): boolean {
  if (a.length > b.length) return tapesEqual(b, a, blank);
  if (a.length < b.length) {
    for (let i=a.length; i<b.length; i++) {
      if (b[i] !== blank) return false;
    }
  }
  for (let i=0; i<a.length; i++) {
    if (a[i] !== b[i]) return false;
  }
  return true;
}

export function tmsEqual<State extends string, Symbol extends string>(a: TuringMachineSnapshot<State, Symbol>, b: TuringMachineSnapshot<State, Symbol>): boolean {
  if (a.spec !== b.spec) return false;  // object equality is fine here at time of writing, since the spec is readonly and unique, never reconstructed
  if (a.state !== b.state) return false;
  if (a.pos !== b.pos) return false;
  if (!tapesEqual(a.tape, b.tape, a.spec.blank)) return false;
  return true;
}
