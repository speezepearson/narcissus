import { z } from "zod";
import { type Dir, type TuringMachineSpec } from "./types";
import rawSpecs from "./machine-specs.json";

const DirSchema = z.literal("L").or(z.literal("R"));
const RuleTriple = z.tuple([z.string(), z.string(), DirSchema]);

const JsonSpecSchema = z.object({
  name: z.string(),
  description: z.string(),
  allStates: z.array(z.string()),
  allSymbols: z.array(z.string()),
  initial: z.string(),
  acceptingStates: z.array(z.string()),
  blank: z.string(),
  rules: z.record(z.string(), z.record(z.string(), RuleTriple)),
  symbolChars: z.record(z.string(), z.string()),
});
type JsonSpec = z.infer<typeof JsonSpecSchema>;

export type ParsedSpec = {
  name: string;
  description: string;
  spec: TuringMachineSpec<string, string>;
  symbolChars: Record<string, string>;
  blank: string;
};

function parseSpec(json: JsonSpec): ParsedSpec {
  const sc = json.symbolChars; // rustName -> displayChar

  const rules = new Map<string, Map<string, [string, string, Dir]>>();
  for (const [state, symbolMap] of Object.entries(json.rules)) {
    const inner = new Map<string, [string, string, Dir]>();
    for (const [symbol, [ns, nsym, dir]] of Object.entries(symbolMap)) {
      inner.set(sc[symbol], [ns, sc[nsym], dir]);
    }
    rules.set(state, inner);
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
