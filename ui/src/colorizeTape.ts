const GREEN_SYMS = new Set(["*", "X", "Y", "^", ">"]);

export function colorizeTape(tape: string[], headPos: number): string {
  let out = "";
  for (let i = 0; i < tape.length; i++) {
    const ch = tape[i];
    const escaped =
      ch === "&" ? "&amp;" : ch === "<" ? "&lt;" : ch === ">" ? "&gt;" : ch;
    if (i === headPos) {
      out += `<span style="outline:1px solid #f87171">${escaped}</span>`;
    } else if (GREEN_SYMS.has(ch)) {
      out += `<span style="outline:1px solid #4ade80">${escaped}</span>`;
    } else {
      out += escaped;
    }
  }
  return out;
}
