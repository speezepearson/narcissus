import { expect } from "vitest";

export function must<T>(x: T | undefined): T {
  expect(x).toBeDefined();
  return x!;
}
