import { z } from "zod";
import { type Dir, type TuringMachineSpec, State, Symbol } from "./types";
import rawRustExport from "./rust-export.json";

const DirSchema = z.literal("L").or(z.literal("R"));
const RuleTriple = z.tuple([z.string(), z.string(), DirSchema]);
const SymbolName = z.string().brand<"SymbolName">();
type SymbolName = z.infer<typeof SymbolName>;

const GraphNodeSchema = z.object({
  id: z.string(),
  label: z.string(),
  cluster: z.string().optional(),
});

const GraphEdgeSchema = z.object({
  id: z.string(),
  source: z.string(),
  target: z.string(),
  label: z.string(),
  symbol: z.string(),
});

const GraphClusterSchema = z.object({
  id: z.string(),
  label: z.string(),
});

const GraphSpecSchema = z.object({
  nodes: z.array(GraphNodeSchema),
  edges: z.array(GraphEdgeSchema),
  clusters: z.array(GraphClusterSchema),
});

export type GraphSpec = z.infer<typeof GraphSpecSchema>;
export type GraphNode = z.infer<typeof GraphNodeSchema>;
export type GraphEdge = z.infer<typeof GraphEdgeSchema>;
export type GraphCluster = z.infer<typeof GraphClusterSchema>;

const JsonSpecSchema = z.object({
  name: z.string(),
  description: z.string(),
  allStates: z.array(State),
  allSymbols: z.array(Symbol),
  initial: State,
  acceptingStates: z.array(State),
  blank: SymbolName,
  rules: z.record(State, z.record(SymbolName, RuleTriple)),
  symbolChars: z.record(z.string(), Symbol),
  stateDescriptions: z.record(State, z.string()),
  graph: GraphSpecSchema,
});
type JsonSpec = z.infer<typeof JsonSpecSchema>;

export type ParsedSpec = {
  name: string;
  description: string;
  spec: TuringMachineSpec;
  symbolChars: Record<SymbolName, Symbol>;
  stateDescriptions: Record<State, string>;
  blank: SymbolName;
  graph: GraphSpec;
};

function parseSpec(json: JsonSpec): ParsedSpec {
  const sc = json.symbolChars; // rustName -> displayChar

  const rules = new Map<State, Map<Symbol, [State, Symbol, Dir]>>();
  for (const [state, symbolMap] of Object.entries(json.rules)) {
    const inner = new Map<Symbol, [State, Symbol, Dir]>();
    for (const [symbol, [ns, nsym, dir]] of Object.entries(symbolMap)) {
      inner.set(sc[symbol], [State.parse(ns), sc[nsym], dir]);
    }
    rules.set(State.parse(state), inner);
  }

  return {
    name: json.name,
    description: json.description,
    spec: {
      allStates: json.allStates,
      allSymbols: json.allSymbols.map((s) => sc[s]),
      initial: json.initial,
      acceptingStates: new Set(json.acceptingStates),
      blank: sc[json.blank],
      rules,
    },
    symbolChars: json.symbolChars,
    stateDescriptions: json.stateDescriptions,
    blank: json.blank,
    graph: json.graph,
  };
}

export const RustExportSchema = z.object({
  machineSpecs: z.array(JsonSpecSchema),
  welcomeModalExample: z.object({
    bitFlipperSpec: JsonSpecSchema,
    utmSpec: JsonSpecSchema,
    bitFlipperInput: z.array(Symbol),
    utmInput: z.array(Symbol),
    doubleUtmInput: z.array(Symbol),
  }),
});

export const rustExport = RustExportSchema.parse(rawRustExport);

export const machineSpecs: ParsedSpec[] =
  rustExport.machineSpecs.map(parseSpec);

export const utmSpec = machineSpecs.find(
  (s) => s.name === "Universal Turing Machine",
)!;
