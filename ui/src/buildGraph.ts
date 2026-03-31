import { type TuringMachineSpec } from "./types";

export type GraphNode = {
  id: string;
  label: string;
  cluster?: string;
};

export type GraphEdge = {
  id: string;
  source: string;
  target: string;
  label: string;
  symbols: string[];
};

export type GraphCluster = {
  id: string;
  label: string;
  parent?: string;
};

export type GraphSpec = {
  nodes: GraphNode[];
  edges: GraphEdge[];
  clusters: GraphCluster[];
};

export type ClusterConfig = {
  /** Maps a state name to its leaf cluster (id, label). */
  stateCluster: (state: string) => { id: string; label: string } | undefined;
  /** Extra meta-clusters and parent assignments. */
  hierarchy?: GraphCluster[];
};

/**
 * Build a graph spec (nodes, edges, clusters) from a TuringMachineSpec.
 *
 * Edges are grouped: if multiple (state, symbol) pairs share the same
 * (source, target, right-of-arrow), they're merged into one edge.
 * If sym === newSym, the label abbreviates to just the direction.
 */
export function buildGraph(
  spec: TuringMachineSpec,
  clusterConfig?: ClusterConfig,
  stateDescriptions?: Record<string, string>,
): GraphSpec {
  // -- Nodes --
  const seenClusters = new Map<string, string>(); // id -> label
  const nodes: GraphNode[] = spec.allStates.map((state) => {
    const name = String(state);
    const cluster = clusterConfig?.stateCluster(name);
    if (cluster) {
      seenClusters.set(cluster.id, cluster.label);
    }
    const desc = stateDescriptions?.[name];
    const label = desc && desc !== name ? `${name}\n${desc}` : name;
    return { id: name, label, cluster: cluster?.id };
  });

  // -- Clusters --
  const clusters: GraphCluster[] = [...seenClusters.entries()].map(
    ([id, label]) => ({ id, label }),
  );
  // Merge in hierarchy (meta-clusters + parent assignments)
  if (clusterConfig?.hierarchy) {
    for (const h of clusterConfig.hierarchy) {
      const existing = clusters.find((c) => c.id === h.id);
      if (existing) {
        if (h.parent) existing.parent = h.parent;
        if (h.label) existing.label = h.label;
      } else {
        clusters.push({ ...h });
      }
    }
  }

  // -- Edges (grouped) --
  // Key: "source\0target\0rightSide" -> list of display-char symbols
  const edgeGroups = new Map<string, string[]>();

  for (const [state, symbolMap] of spec.rules) {
    for (const [sym, [newState, newSym, dir]] of symbolMap) {
      const src = String(state);
      const tgt = String(newState);
      const sc = String(sym);
      const nsc = String(newSym);
      const right = sc === nsc ? dir : `${nsc},${dir}`;
      const key = `${src}\0${tgt}\0${right}`;
      let group = edgeGroups.get(key);
      if (!group) {
        group = [];
        edgeGroups.set(key, group);
      }
      group.push(sc);
    }
  }

  const edges: GraphEdge[] = [];
  for (const [key, syms] of edgeGroups) {
    const [source, target, right] = key.split("\0");
    const left = syms.join("");
    edges.push({
      id: `${source}--${left}`,
      source,
      target,
      label: `${left} → ${right}`,
      symbols: syms,
    });
  }

  // -- Filter out nodes with no incoming edges (except initial state) --
  const initId = String(spec.initial);
  const hasIncoming = new Set<string>();
  for (const edge of edges) {
    if (edge.target !== edge.source) {
      hasIncoming.add(edge.target);
    }
  }
  const keepNodes = new Set(
    nodes
      .filter((n) => n.id === initId || hasIncoming.has(n.id))
      .map((n) => n.id),
  );
  const filteredNodes = nodes.filter((n) => keepNodes.has(n.id));
  const filteredEdges = edges.filter(
    (e) => keepNodes.has(e.source) && keepNodes.has(e.target),
  );

  return { nodes: filteredNodes, edges: filteredEdges, clusters };
}

// ════════════════════════════════════════════════════════════════════
// UTM-specific cluster configuration
// ════════════════════════════════════════════════════════════════════

const UTM_STATE_CLUSTERS: [RegExp | string, string, string][] = [
  // [pattern, clusterId, clusterLabel]
  // pattern can be a string (exact match) or regex (prefix match)
  [/^Init/, "init", "Init"],
  ["MarkRule", "mark_rule", "Mark Rule"],
  ["MarkRuleNoMatch", "mark_rule", "Mark Rule"],
  [/^CmpSt/, "cmp_state", "Compare State"],
  [/^Stm/, "cmp_state", "Compare State"],
  [/^Stf/, "cmp_state", "Compare State"],
  ["StMatchCleanup", "cmp_state", "Compare State"],
  ["SymSkipState", "cmp_state", "Compare State"],
  [/^CmpSym/, "cmp_sym", "Compare Symbol"],
  [/^Symf/, "cmp_sym", "Compare Symbol"],
  ["SymMatchCleanup", "cmp_sym", "Compare Symbol"],
  [/^Smc/, "cmp_sym", "Compare Symbol"],
  [/^CpNst/, "cp_nst", "Copy New State"],
  ["ApplyReadNst", "cp_nst", "Copy New State"],
  [/^CpNsym/, "cp_nsym", "Copy New Symbol"],
  [/^Rd/, "read_dir", "Read Direction"],
  ["ReadDir", "read_dir", "Read Direction"],
  [/^Mr/, "move_right", "Move Right"],
  ["MoveRight", "move_right", "Move Right"],
  [/^Ml/, "move_left", "Move Left"],
  ["MoveLeft", "move_left", "Move Left"],
  ["DoneSeekHome", "seek_home", "Seek Home"],
  ["DoneSeekHomeThroughState", "seek_home", "Seek Home"],
  [/^ChkAcc/, "chk_acc", "Check Accept"],
  [/^Nm/, "chk_acc", "Check Accept"],
  [/^Acc/, "accept", "Accept"],
  ["Accept", "accept", "Accept"],
  [/^Rej/, "reject", "Reject"],
  ["Reject", "reject", "Reject"],
  [/^Np/, "noop", "Noop Compact"],
];

function utmStateCluster(
  state: string,
): { id: string; label: string } | undefined {
  for (const [pattern, id, label] of UTM_STATE_CLUSTERS) {
    if (typeof pattern === "string") {
      if (state === pattern) return { id, label };
    } else {
      if (pattern.test(state)) return { id, label };
    }
  }
  return undefined;
}

const UTM_CLUSTER_HIERARCHY: GraphCluster[] = [
  // Meta-clusters
  { id: "main_loop", label: "Main Loop" },
  { id: "find_rule", label: "Find Rule", parent: "main_loop" },
  { id: "apply_rule", label: "Apply Rule", parent: "main_loop" },
  { id: "move_head", label: "Move Head" },
  { id: "halt", label: "Halt" },
  // Parent assignments for leaf clusters
  { id: "mark_rule", label: "Mark Rule", parent: "find_rule" },
  { id: "cmp_state", label: "Compare State", parent: "find_rule" },
  { id: "cmp_sym", label: "Compare Symbol", parent: "find_rule" },
  { id: "noop", label: "Noop Compact", parent: "cmp_sym" },
  { id: "cp_nst", label: "Copy New State", parent: "apply_rule" },
  { id: "cp_nsym", label: "Copy New Symbol", parent: "apply_rule" },
  { id: "read_dir", label: "Read Direction", parent: "apply_rule" },
  { id: "seek_home", label: "Seek Home", parent: "apply_rule" },
  { id: "move_head", label: "Move Head", parent: "apply_rule" },
  { id: "move_left", label: "Move Left", parent: "move_head" },
  { id: "move_right", label: "Move Right", parent: "move_head" },
  { id: "chk_acc", label: "Check Accept", parent: "halt" },
  { id: "accept", label: "Accept", parent: "halt" },
  { id: "reject", label: "Reject", parent: "halt" },
];

export const UTM_CLUSTER_CONFIG: ClusterConfig = {
  stateCluster: utmStateCluster,
  hierarchy: UTM_CLUSTER_HIERARCHY,
};
