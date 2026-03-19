import { expect } from "vitest";
import { tapesEqual } from "./util";
import type { TuringMachineSnapshot } from "./types";

export function expectTmsEqual<State extends string, Symbol extends string>(a: TuringMachineSnapshot<State, Symbol>, b: TuringMachineSnapshot<State, Symbol>): void {
  expect(a.spec).toBe(b.spec);
  expect(a.state).toBe(b.state);
  expect(a.pos).toBe(b.pos);
  if (!tapesEqual(a.tape, b.tape, a.spec.blank)) {
    expect(a.tape).toEqual(b.tape);
    expect(false).toBe(true); // if tapesEqual fails, a.tape sure as heck shouldn't equal b.tape
  }
}

export function must<T>(x: T | undefined): T {
  expect(x).toBeDefined();
  return x!;
}