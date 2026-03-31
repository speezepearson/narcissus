/// Serializes Turing machine specs to JSON compatible with the TypeScript
/// `TuringMachineSpec<State, Symbol>` interface in `ui/src/types.ts`.
use std::collections::BTreeMap;

use serde::Serialize;

use crate::tm::{Dir, TuringMachineSpec};

#[derive(Serialize, Clone)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    /// If present, this node belongs to a cluster.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct GraphEdge {
    /// Unique id: "{source}--{symbol}"
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
    /// The triggering symbol (display char), for highlighting the active edge.
    pub symbol: String,
}

#[derive(Serialize, Clone)]
pub struct GraphCluster {
    pub id: String,
    pub label: String,
}

#[derive(Serialize, Clone)]
pub struct GraphSpec {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub clusters: Vec<GraphCluster>,
}

#[derive(Serialize)]
pub struct JsonTuringMachineSpec {
    pub name: String,
    pub description: String,
    #[serde(rename = "allStates")]
    pub all_states: Vec<String>,
    #[serde(rename = "allSymbols")]
    pub all_symbols: Vec<String>,
    pub initial: String,
    #[serde(rename = "acceptingStates")]
    pub accepting_states: Vec<String>,
    pub blank: String,
    /// Keyed by state -> symbol -> [new_state, new_symbol, dir]
    pub rules: BTreeMap<String, BTreeMap<String, (String, String, String)>>,
    /// Maps each symbol's string name to a display character.
    #[serde(rename = "symbolChars")]
    pub symbol_chars: BTreeMap<String, String>,
    /// Maps each state's string name to a human-friendly description.
    #[serde(rename = "stateDescriptions")]
    pub state_descriptions: BTreeMap<String, String>,
    /// Graph spec for visualization (nodes, edges, clusters).
    pub graph: GraphSpec,
}

pub fn export_spec<Spec: TuringMachineSpec>(
    spec: &Spec,
    name: &str,
    description: &str,
    state_name: impl Fn(Spec::State) -> String,
    state_description: impl Fn(Spec::State) -> String,
    symbol_name: impl Fn(Spec::Symbol) -> String,
    symbol_char: impl Fn(Spec::Symbol) -> char,
) -> JsonTuringMachineSpec {
    export_spec_with_clusters(
        spec,
        name,
        description,
        &state_name,
        state_description,
        &symbol_name,
        &symbol_char,
        |_| None,
    )
}

pub fn export_spec_with_clusters<Spec: TuringMachineSpec>(
    spec: &Spec,
    name: &str,
    description: &str,
    state_name: &impl Fn(Spec::State) -> String,
    state_description: impl Fn(Spec::State) -> String,
    symbol_name: &impl Fn(Spec::Symbol) -> String,
    symbol_char: &impl Fn(Spec::Symbol) -> char,
    state_cluster: impl Fn(Spec::State) -> Option<(String, String)>,
) -> JsonTuringMachineSpec {
    let all_states: Vec<String> = spec.iter_states().map(state_name).collect();
    let all_symbols: Vec<String> = spec.iter_symbols().map(symbol_name).collect();
    let initial = state_name(spec.initial());
    let blank = symbol_name(spec.blank());

    let accepting_states: Vec<String> = spec
        .iter_states()
        .filter(|s| spec.is_accepting(*s))
        .map(state_name)
        .collect();

    let mut rules: BTreeMap<String, BTreeMap<String, (String, String, String)>> = BTreeMap::new();
    for (st, sym, nst, nsym, dir) in spec.iter_rules() {
        let dir_str = match dir {
            Dir::Left => "L".to_string(),
            Dir::Right => "R".to_string(),
        };
        rules.entry(state_name(st)).or_default().insert(
            symbol_name(sym),
            (state_name(nst), symbol_name(nsym), dir_str),
        );
    }

    let symbol_chars: BTreeMap<String, String> = spec
        .iter_symbols()
        .map(|s| (symbol_name(s), symbol_char(s).to_string()))
        .collect();

    let state_descriptions: BTreeMap<String, String> = spec
        .iter_states()
        .map(|s| (state_name(s), state_description(s)))
        .collect();

    // Build graph spec
    let mut seen_clusters: BTreeMap<String, String> = BTreeMap::new();
    let nodes: Vec<GraphNode> = spec
        .iter_states()
        .map(|s| {
            let cluster = state_cluster(s);
            if let Some((ref cid, ref clabel)) = cluster {
                seen_clusters
                    .entry(cid.clone())
                    .or_insert_with(|| clabel.clone());
            }
            GraphNode {
                id: state_name(s),
                label: state_name(s),
                cluster: cluster.map(|(cid, _)| cid),
            }
        })
        .collect();

    let clusters: Vec<GraphCluster> = seen_clusters
        .into_iter()
        .map(|(id, label)| GraphCluster { id, label })
        .collect();

    let mut edges: Vec<GraphEdge> = Vec::new();
    for (st, sym, nst, nsym, dir) in spec.iter_rules() {
        // Skip self-loops that don't change the symbol (like gen_mermaid)
        if st == nst && sym == nsym {
            continue;
        }
        let dir_str = match dir {
            Dir::Left => "L",
            Dir::Right => "R",
        };
        let sc = symbol_char(sym);
        let nsc = symbol_char(nsym);
        let label = format!("{}/{},{}", sc, nsc, dir_str);
        edges.push(GraphEdge {
            id: format!("{}--{}", state_name(st), symbol_name(sym)),
            source: state_name(st),
            target: state_name(nst),
            label,
            symbol: sc.to_string(),
        });
    }

    let graph = GraphSpec {
        nodes,
        edges,
        clusters,
    };

    JsonTuringMachineSpec {
        name: name.to_string(),
        description: description.to_string(),
        all_states,
        all_symbols,
        initial,
        accepting_states,
        blank,
        rules,
        symbol_chars,
        state_descriptions,
        graph,
    }
}
