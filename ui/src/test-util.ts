import { expect } from "vitest";

export function must<T>(x: T | undefined): T {
  expect(x).toBeDefined();
  return x!;
}

export function makeBreaker(everyMs: number = 100): () => Promise<void> {
  let lastBreak = performance.now();
  return async () => {
    const now = performance.now();
    if (now - lastBreak > everyMs) {
      await new Promise((resolve) => setTimeout(resolve, 1e-9));
      lastBreak = now;
    }
  };
}
