import { z } from "zod";
import { type Dir, type TuringMachineSpec, State, Symbol } from "./types";
import rawSpecs from "./machine-specs.json";

const DirSchema = z.literal("L").or(z.literal("R"));
const RuleTriple = z.tuple([z.string(), z.string(), DirSchema]);
const SymbolName = z.string().brand<"SymbolName">();
type SymbolName = z.infer<typeof SymbolName>;

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
});
type JsonSpec = z.infer<typeof JsonSpecSchema>;

export type ParsedSpec = {
  name: string;
  description: string;
  spec: TuringMachineSpec;
  symbolChars: Record<SymbolName, Symbol>;
  blank: SymbolName;
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
    blank: json.blank,
  };
}

export const machineSpecs: ParsedSpec[] = z
  .array(JsonSpecSchema)
  .parse(rawSpecs)
  .map(parseSpec);
