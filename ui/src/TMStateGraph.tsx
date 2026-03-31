import { useEffect, useRef } from "react";
import cytoscape from "cytoscape";
import dagre from "cytoscape-dagre";
import type { GraphSpec } from "./parseSpec";
import type { State, Symbol } from "./types";

cytoscape.use(dagre);

type Props = {
  graph: GraphSpec;
  currentState: State;
  /** The symbol currently under the head (for highlighting the active edge). */
  currentSymbol?: Symbol;
};

const CLUSTER_COLORS: Record<string, string> = {
  init: "#6366f1",
  mark_rule: "#8b5cf6",
  cmp_state: "#3b82f6",
  cmp_sym: "#0ea5e9",
  cp_nst: "#14b8a6",
  cp_nsym: "#10b981",
  read_dir: "#f59e0b",
  move_right: "#f97316",
  move_left: "#ef4444",
  seek_home: "#ec4899",
  chk_acc: "#a855f7",
  accept: "#16a34a",
  reject: "#dc2626",
  noop: "#64748b",
  other: "#94a3b8",
};

function clusterColor(clusterId: string | undefined): string {
  if (!clusterId) return "#94a3b8";
  return CLUSTER_COLORS[clusterId] ?? "#94a3b8";
}

export function TMStateGraph({ graph, currentState, currentSymbol }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const cyRef = useRef<cytoscape.Core | null>(null);

  // Build cytoscape elements once per graph identity
  useEffect(() => {
    if (!containerRef.current) return;

    const elements: cytoscape.ElementDefinition[] = [];

    // Compound (parent) nodes for clusters
    for (const cluster of graph.clusters) {
      elements.push({
        data: { id: `cluster-${cluster.id}`, label: cluster.label },
        classes: "cluster",
      });
    }

    // State nodes
    for (const node of graph.nodes) {
      elements.push({
        data: {
          id: node.id,
          label: node.label,
          parent: node.cluster ? `cluster-${node.cluster}` : undefined,
          clusterId: node.cluster,
        },
      });
    }

    // Edges
    for (const edge of graph.edges) {
      elements.push({
        data: {
          id: edge.id,
          source: edge.source,
          target: edge.target,
          label: edge.label,
          symbol: edge.symbol,
        },
      });
    }

    const cy = cytoscape({
      container: containerRef.current,
      elements,
      style: [
        {
          selector: "node",
          style: {
            label: "data(label)",
            "text-valign": "center",
            "text-halign": "center",
            "font-size": "8px",
            "font-family": "ui-monospace, Consolas, monospace",
            width: 20,
            height: 20,
            "background-color": "#e2e8f0",
            "border-width": 1,
            "border-color": "#94a3b8",
            color: "#1e293b",
            "text-wrap": "ellipsis",
            "text-max-width": "60px",
          },
        },
        {
          selector: "node.cluster",
          style: {
            "text-valign": "top",
            "text-halign": "center",
            "font-size": "10px",
            "font-weight": "bold",
            "background-opacity": 0.08,
            "border-width": 1,
            "border-color": "#cbd5e1",
            "border-opacity": 0.5,
            padding: "12px",
            shape: "roundrectangle",
            color: "#475569",
          },
        },
        {
          selector: "node.active-state",
          style: {
            "background-color": "#6366f1",
            "border-color": "#4338ca",
            "border-width": 3,
            color: "#ffffff",
            "font-weight": "bold",
            "z-index": 999,
          },
        },
        {
          selector: "edge",
          style: {
            width: 1,
            "line-color": "#cbd5e1",
            "target-arrow-color": "#cbd5e1",
            "target-arrow-shape": "triangle",
            "curve-style": "bezier",
            label: "data(label)",
            "font-size": "6px",
            "font-family": "ui-monospace, Consolas, monospace",
            "text-rotation": "autorotate",
            color: "#64748b",
            "text-background-color": "#ffffff",
            "text-background-opacity": 0.8,
            "text-background-padding": "1px",
            "arrow-scale": 0.6,
          },
        },
        {
          selector: "edge.active-edge",
          style: {
            width: 3,
            "line-color": "#ef4444",
            "target-arrow-color": "#ef4444",
            "font-weight": "bold",
            color: "#ef4444",
            "z-index": 999,
          },
        },
      ],
      layout: {
        name: "dagre",
        rankDir: "LR",
        nodeSep: 15,
        rankSep: 40,
        edgeSep: 5,
        padding: 20,
      } as cytoscape.LayoutOptions,
      minZoom: 0.1,
      maxZoom: 5,
      wheelSensitivity: 0.3,
    });

    cyRef.current = cy;

    return () => {
      cy.destroy();
      cyRef.current = null;
    };
  }, [graph]);

  // Update highlighting when currentState/currentSymbol change
  useEffect(() => {
    const cy = cyRef.current;
    if (!cy) return;

    // Clear previous highlights
    cy.elements(".active-state").removeClass("active-state");
    cy.elements(".active-edge").removeClass("active-edge");

    // Highlight current state
    const stateNode = cy.getElementById(String(currentState));
    if (stateNode.length) {
      stateNode.addClass("active-state");
    }

    // Highlight the edge about to be taken
    if (currentSymbol !== undefined) {
      const edgeId = `${String(currentState)}--${String(currentSymbol)}`;
      const edge = cy.getElementById(edgeId);
      if (edge.length) {
        edge.addClass("active-edge");
      }
    }
  }, [currentState, currentSymbol]);

  return (
    <div
      ref={containerRef}
      className="tm-state-graph"
      style={{
        width: "100%",
        height: "500px",
        border: "1px solid var(--border, #ccc)",
        borderRadius: "8px",
        margin: "8px 0",
        background: "var(--code-bg, #f8fafc)",
      }}
    />
  );
}
