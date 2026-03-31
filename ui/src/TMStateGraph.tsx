import { useEffect, useRef, useState } from "react";
import cytoscape from "cytoscape";
// @ts-expect-error no type declarations for cytoscape-elk
import elk from "cytoscape-elk";
import type { GraphSpec } from "./buildGraph";
import type { State, Symbol } from "./types";

cytoscape.use(elk);

type Props = {
  graph: GraphSpec;
  currentState: State;
  currentSymbol?: Symbol;
};

type ElkParams = {
  algorithm: string;
  direction: string;
  nodeSpacing: number;
  layerSpacing: number;
  edgeEdgeSpacing: number;
  edgeNodeSpacing: number;
  compactionStrategy: string;
  hierarchyHandling: string;
};

const DEFAULT_ELK: ElkParams = {
  algorithm: "layered",
  direction: "RIGHT",
  nodeSpacing: 15,
  layerSpacing: 30,
  edgeEdgeSpacing: 8,
  edgeNodeSpacing: 10,
  compactionStrategy: "IMPROVE_STRAIGHTNESS",
  hierarchyHandling: "INCLUDE_CHILDREN",
};

const ALGORITHMS = [
  "layered",
  "mrtree",
  "stress",
  "force",
  "radial",
  "box",
  "disco",
];
const DIRECTIONS = ["RIGHT", "DOWN", "LEFT", "UP"];
const COMPACTION_STRATEGIES = ["NONE", "IMPROVE_STRAIGHTNESS", "NODE_SIZE"];
const HIERARCHY_MODES = ["INCLUDE_CHILDREN", "SEPARATE_CHILDREN", "INHERIT"];

function ElkControls({
  params,
  onChange,
}: {
  params: ElkParams;
  onChange: (p: ElkParams) => void;
}) {
  const set = <K extends keyof ElkParams>(key: K, val: ElkParams[K]) =>
    onChange({ ...params, [key]: val });

  return (
    <div className="tm-elk-controls">
      <label>
        Algorithm
        <select
          value={params.algorithm}
          onChange={(e) => set("algorithm", e.target.value)}
        >
          {ALGORITHMS.map((a) => (
            <option key={a}>{a}</option>
          ))}
        </select>
      </label>
      <label>
        Direction
        <select
          value={params.direction}
          onChange={(e) => set("direction", e.target.value)}
        >
          {DIRECTIONS.map((d) => (
            <option key={d}>{d}</option>
          ))}
        </select>
      </label>
      <label>
        Node spacing
        <input
          type="range"
          min={1}
          max={100}
          value={params.nodeSpacing}
          onChange={(e) => set("nodeSpacing", +e.target.value)}
        />
        <span>{params.nodeSpacing}</span>
      </label>
      <label>
        Layer spacing
        <input
          type="range"
          min={1}
          max={200}
          value={params.layerSpacing}
          onChange={(e) => set("layerSpacing", +e.target.value)}
        />
        <span>{params.layerSpacing}</span>
      </label>
      <label>
        Edge-edge spacing
        <input
          type="range"
          min={1}
          max={50}
          value={params.edgeEdgeSpacing}
          onChange={(e) => set("edgeEdgeSpacing", +e.target.value)}
        />
        <span>{params.edgeEdgeSpacing}</span>
      </label>
      <label>
        Edge-node spacing
        <input
          type="range"
          min={1}
          max={50}
          value={params.edgeNodeSpacing}
          onChange={(e) => set("edgeNodeSpacing", +e.target.value)}
        />
        <span>{params.edgeNodeSpacing}</span>
      </label>
      <label>
        Compaction
        <select
          value={params.compactionStrategy}
          onChange={(e) => set("compactionStrategy", e.target.value)}
        >
          {COMPACTION_STRATEGIES.map((s) => (
            <option key={s}>{s}</option>
          ))}
        </select>
      </label>
      <label>
        Hierarchy
        <select
          value={params.hierarchyHandling}
          onChange={(e) => set("hierarchyHandling", e.target.value)}
        >
          {HIERARCHY_MODES.map((m) => (
            <option key={m}>{m}</option>
          ))}
        </select>
      </label>
    </div>
  );
}

export function TMStateGraph({ graph, currentState, currentSymbol }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const cyRef = useRef<cytoscape.Core | null>(null);
  const [elkParams, setElkParams] = useState<ElkParams>(DEFAULT_ELK);

  // Build cytoscape elements + run layout
  useEffect(() => {
    if (!containerRef.current) return;

    const elements: cytoscape.ElementDefinition[] = [];

    for (const cluster of graph.clusters) {
      elements.push({
        data: {
          id: `cluster-${cluster.id}`,
          label: cluster.label,
          parent: cluster.parent ? `cluster-${cluster.parent}` : undefined,
        },
        classes: "cluster",
      });
    }

    for (const node of graph.nodes) {
      elements.push({
        data: {
          id: node.id,
          label: node.label,
          parent: node.cluster ? `cluster-${node.cluster}` : undefined,
        },
      });
    }

    for (const edge of graph.edges) {
      elements.push({
        data: {
          id: edge.id,
          source: edge.source,
          target: edge.target,
          label: edge.label,
          symbols: edge.symbols,
        },
        classes: edge.source === edge.target ? "selfloop" : undefined,
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
            "font-size": "10px",
            "font-family": "ui-monospace, Consolas, monospace",
            width: "label",
            height: "label",
            shape: "roundrectangle",
            "background-color": "#e2e8f0",
            "border-width": 1,
            "border-color": "#94a3b8",
            color: "#1e293b",
            "text-wrap": "wrap",
            "text-max-width": "10em",
            padding: "6px",
          },
        },
        {
          selector: "node.cluster",
          style: {
            "text-valign": "top",
            "text-halign": "center",
            "font-size": "81px",
            "font-weight": "bold",
            "background-opacity": 0.08,
            "border-width": 1,
            "border-color": "#cbd5e1",
            "border-opacity": 0.5,
            padding: "16px",
            shape: "roundrectangle",
            color: "#475569",
            "text-wrap": "none",
            width: undefined as unknown as number,
            height: undefined as unknown as number,
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
            "font-size": "7px",
            "font-family": "ui-monospace, Consolas, monospace",
            "text-rotation": "autorotate",
            color: "#64748b",
            "text-background-color": "#ffffff",
            "text-background-opacity": 0.8,
            "text-background-padding": "2px",
            "arrow-scale": 0.6,
          },
        },
        {
          selector: "edge.selfloop",
          style: {
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            "curve-style": "loop" as any,
            "loop-direction": "0deg",
            "loop-sweep": "-90deg",
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
        name: "elk",
        nodeDimensionsIncludeLabels: true,
        elk: {
          algorithm: elkParams.algorithm,
          "elk.direction": elkParams.direction,
          "spacing.nodeNode": String(elkParams.nodeSpacing),
          "layered.spacing.nodeNodeBetweenLayers": String(
            elkParams.layerSpacing,
          ),
          "spacing.edgeEdge": String(elkParams.edgeEdgeSpacing),
          "spacing.edgeNode": String(elkParams.edgeNodeSpacing),
          "layered.compaction.postCompaction.strategy":
            elkParams.compactionStrategy,
          hierarchyHandling: elkParams.hierarchyHandling,
        },
        padding: 20,
      } as cytoscape.LayoutOptions,
      minZoom: 0.05,
      maxZoom: 5,
      wheelSensitivity: 0.3,
    });

    cyRef.current = cy;

    return () => {
      cy.destroy();
      cyRef.current = null;
    };
  }, [graph, elkParams]);

  // Update highlighting
  useEffect(() => {
    const cy = cyRef.current;
    if (!cy) return;

    cy.elements(".active-state").removeClass("active-state");
    cy.elements(".active-edge").removeClass("active-edge");

    const stateNode = cy.getElementById(String(currentState));
    if (stateNode.length) {
      stateNode.addClass("active-state");
    }

    if (currentSymbol !== undefined) {
      const sym = String(currentSymbol);
      const st = String(currentState);
      cy.edges().forEach((edge) => {
        const syms: string[] = edge.data("symbols") ?? [];
        if (edge.data("source") === st && syms.includes(sym)) {
          edge.addClass("active-edge");
        }
      });
    }
  }, [currentState, currentSymbol]);

  return (
    <div>
      <ElkControls params={elkParams} onChange={setElkParams} />
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
    </div>
  );
}
