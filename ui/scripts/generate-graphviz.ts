import { writeFileSync } from "node:fs";
import { myUtmSpec } from "../src/my-utm-spec";
import type { Dir } from "../src/types";

function escapeLabel(s: string): string {
  return s.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
}

type Transition = { readSym: string; writeSym: string; dir: Dir };

/**
 * Given a list of transitions for a single (src, dst) edge, produce
 * compact label lines. Transitions where readSym === writeSym are
 * grouped by direction into a single "{sym1} {sym2} …: {dir}" line.
 * Other transitions keep their "read→write,dir" form.
 */
function buildEdgeLabel(transitions: Transition[]): string {
  // Separate identity (read===write) from non-identity transitions
  const identityByDir = new Map<Dir, string[]>();
  const nonIdentity: string[] = [];

  for (const { readSym, writeSym, dir } of transitions) {
    if (readSym === writeSym) {
      if (!identityByDir.has(dir)) identityByDir.set(dir, []);
      identityByDir.get(dir)!.push(readSym);
    } else {
      nonIdentity.push(`${readSym}→${writeSym},${dir}`);
    }
  }

  const parts: string[] = [];

  for (const [dir, syms] of identityByDir) {
    // Only lump when there are multiple identity transitions
    if (syms.length > 1) {
      parts.push(`${syms.join(" ")}: ${dir}`);
    } else {
      parts.push(`${syms[0]}→${syms[0]},${dir}`);
    }
  }

  parts.push(...nonIdentity);
  return parts.join("\n");
}

function generateDot(rankdir: string, engine: string): string {
  const lines: string[] = [];
  lines.push("digraph UTM {");
  lines.push(`  rankdir=${rankdir};`);
  if (engine === "neato" || engine === "fdp" || engine === "sfdp") {
    lines.push("  edge [len=2.5];");
    lines.push('  sep="+20";');
    lines.push("  overlap=false;");
  }
  lines.push("  node [shape=circle, fontsize=10];");
  lines.push("");

  // Highlight initial state
  lines.push(
    `  "${escapeLabel(myUtmSpec.initial)}" [style=filled, fillcolor=lightblue, penwidth=2];`,
  );

  // Mark accepting states with double circles
  for (const state of myUtmSpec.acceptingStates) {
    lines.push(`  "${escapeLabel(state)}" [shape=doublecircle];`);
  }

  // Initial state indicator
  lines.push(`  __start__ [shape=point, width=0];`);
  lines.push(`  __start__ -> "${escapeLabel(myUtmSpec.initial)}";`);
  lines.push("");

  // Collect edges: group transitions between the same (src, dst) pair
  const edgeTransitions = new Map<string, Transition[]>();

  for (const [srcState, symbolMap] of myUtmSpec.rules) {
    for (const [readSym, [dstState, writeSym, dir]] of symbolMap) {
      const key = `${srcState}\0${dstState}`;
      if (!edgeTransitions.has(key)) {
        edgeTransitions.set(key, []);
      }
      edgeTransitions.get(key)!.push({ readSym, writeSym, dir });
    }
  }

  for (const [key, transitions] of edgeTransitions) {
    const [src, dst] = key.split("\0");
    const label = escapeLabel(buildEdgeLabel(transitions));
    lines.push(
      `  "${escapeLabel(src)}" -> "${escapeLabel(dst)}" [label="${label}", fontsize=8];`,
    );
  }

  lines.push("}");
  return lines.join("\n") + "\n";
}

const LAYOUTS = ["dot", "neato", "fdp", "sfdp", "circo", "twopi"] as const;
const RANKDIRS = ["TB", "LR", "BT", "RL"] as const;

const usage = `Usage: npx tsx scripts/generate-graphviz.ts [--layout=${LAYOUTS.join("|")}] [--rankdir=${RANKDIRS.join("|")}] [--all] [outpath]

  --all     Generate one .dot per (layout × rankdir) combo into outpath dir (default: ./graphviz-out/)
`;

const args = process.argv.slice(2);
const flagAll = args.includes("--all");
const layout =
  args.find((a) => a.startsWith("--layout="))?.split("=")[1] ?? "dot";
const rankdir =
  args.find((a) => a.startsWith("--rankdir="))?.split("=")[1] ?? "TB";
const positional = args.find((a) => !a.startsWith("--"));

if (args.includes("--help")) {
  console.log(usage);
  process.exit(0);
}

if (flagAll) {
  const { mkdirSync } = await import("node:fs");
  const outDir = positional ?? "graphviz-out";
  mkdirSync(outDir, { recursive: true });
  for (const l of LAYOUTS) {
    for (const rd of RANKDIRS) {
      const dot = generateDot(rd, l);
      const filename = `${outDir}/utm-${l}-${rd}.dot`;
      writeFileSync(filename, dot);
      console.log(`Wrote ${filename}`);
    }
  }
  console.log(
    `\nRender all with e.g.:\n  for f in ${outDir}/*.dot; do layout=$(echo $f | sed 's/.*utm-//' | sed 's/-.*//' ); dot -K$layout -Tpdf "$f" -o "\${f%.dot}.pdf"; done`,
  );
} else {
  const outPath = positional ?? "utm-state-machine.dot";
  const dot = generateDot(rankdir, layout);
  writeFileSync(outPath, dot);
  console.log(
    `Wrote ${outPath} (${myUtmSpec.allStates.length} states, rankdir=${rankdir})`,
  );
  console.log(
    `Render with: ${layout} -Tpdf ${outPath} -o ${outPath.replace(/\.dot$/, ".pdf")}`,
  );
}
