import { useEffect, useState } from "react";

const GREEN_SYMS = new Set(["*", "X", "Y", "^", ">"]);

/** Colorize plain tower text into HTML.
 *  - Head cell (parsed from pos=N) gets red background
 *  - Special symbols (*, X, Y, ^, >) get green background
 */
function colorize(text: string): string {
  return text
    .split("\n")
    .map((line) => {
      const headCol = parseHeadCol(line);
      let out = "";
      for (let i = 0; i < line.length; i++) {
        const ch = line[i];
        const escaped =
          ch === "&" ? "&amp;" : ch === "<" ? "&lt;" : ch === ">" ? "&gt;" : ch;
        if (i === headCol) {
          out += `<span style="background:#f87171">${escaped}</span>`;
        } else if (GREEN_SYMS.has(ch)) {
          out += `<span style="background:#4ade80">${escaped}</span>`;
        } else {
          out += escaped;
        }
      }
      return out;
    })
    .join("\n");
}

function parseHeadCol(line: string): number | null {
  const match = line.match(/pos=(\d+)\)$/);
  if (!match) return null;
  return 4 + parseInt(match[1], 10);
}

export function TowerView() {
  const [html, setHtml] = useState("");

  useEffect(() => {
    let active = true;
    const poll = async () => {
      while (active) {
        try {
          const res = await fetch("/api/tower");
          if (res.ok) {
            const text = await res.text();
            setHtml(colorize(text));
          }
        } catch {
          // SWALLOW_EXCEPTION: server may not be ready yet; we'll retry
        }
        await new Promise((r) => setTimeout(r, 100));
      }
    };
    poll();
    return () => {
      active = false;
    };
  }, []);

  return (
    <div style={{ textAlign: "left", padding: "16px" }}>
      <h2>Tower</h2>
      <div
        style={{
          fontFamily: "var(--mono)",
          fontSize: "12px",
          lineHeight: "1.3",
          background: "var(--code-bg)",
          padding: "12px",
          borderRadius: "8px",
          overflowWrap: "break-word",
        }}
        dangerouslySetInnerHTML={{ __html: html }}
      />
    </div>
  );
}
