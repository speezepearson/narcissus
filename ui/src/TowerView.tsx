import { useEffect, useState } from "react";

/** Convert ANSI color codes to HTML spans. */
function ansiToHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/\x1b\[101m(.*?)\x1b\[0m/g, '<span style="background:#f87171">$1</span>')
    .replace(/\x1b\[102m(.*?)\x1b\[0m/g, '<span style="background:#4ade80">$1</span>')
    .replace(/\x1b\[\d+m/g, "");
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
            setHtml(ansiToHtml(text));
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
