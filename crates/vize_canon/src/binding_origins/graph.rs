//! Iterative dependency closure and bounded evidence propagation.
use super::*;
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Key {
    Binding(BindingRef, Usage),
    Global(String, Namespace, Usage),
}
#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Evidence {
    first: Option<SymbolIdentity>,
    second: Option<SymbolIdentity>,
    missing: bool,
    ambiguous: bool,
}
impl Evidence {
    fn add_symbol(&mut self, symbol: SymbolIdentity) {
        if self.first == Some(symbol) || self.second == Some(symbol) {
            return;
        }
        if self.first.is_none() {
            self.first = Some(symbol);
        } else if self.second.is_none() {
            self.second = Some(symbol);
        }
    }
    fn merge(&mut self, other: Self) -> bool {
        let before = *self;
        for symbol in [other.first, other.second].into_iter().flatten() {
            self.add_symbol(symbol);
        }
        self.missing |= other.missing;
        self.ambiguous |= other.ambiguous;
        *self != before
    }
    fn origin(self, reference: SymbolIdentity) -> BindingOrigin {
        if self.ambiguous || self.second.is_some() || self.missing && self.first.is_some() {
            BindingOrigin::Ambiguous
        } else if let Some(symbol) = self.first {
            if symbol == reference {
                BindingOrigin::Vue
            } else {
                BindingOrigin::Custom(symbol)
            }
        } else {
            BindingOrigin::Unresolved
        }
    }
}
#[derive(Default)]
struct Node {
    evidence: Evidence,
    dependents: Vec<usize>,
}
struct Graph<'a, R> {
    facts: &'a ProjectFacts,
    resolver: &'a R,
    keys: Vec<Key>,
    indices: FxHashMap<Key, usize>,
    nodes: Vec<Node>,
    globals: FxHashMap<(String, Namespace), Vec<BindingRef>>,
}
impl<'a, R: Resolver> Graph<'a, R> {
    fn intern(&mut self, key: Key) -> usize {
        if let Some(index) = self.indices.get(&key) {
            return *index;
        }
        let index = self.nodes.len();
        self.indices.insert(key.clone(), index);
        self.keys.push(key);
        self.nodes.push(Node::default());
        index
    }
    fn edge(&mut self, parent: usize, key: Key) {
        let child = self.intern(key);
        self.nodes[child].dependents.push(parent);
    }
    fn external(&mut self, node: usize, identity: SymbolIdentity) {
        if self.facts.versions.get(&identity.source.source) == Some(&identity.source) {
            self.nodes[node].evidence.add_symbol(identity);
        } else {
            self.nodes[node].evidence.missing = true;
        }
    }
    fn target(&mut self, node: usize, owner: BindingRef, usage: Usage, target: &BindingTarget) {
        let mut pending = Vec::new();
        let mut first = Some(target);
        while let Some(target) = first.take().or_else(|| pending.pop()) {
            match target {
                BindingTarget::Opaque => {
                    let source = self.facts.versions[&owner.source];
                    self.nodes[node].evidence.add_symbol(SymbolIdentity {
                        source,
                        local: owner.local,
                    });
                }
                BindingTarget::Alias { binding, usage } => {
                    self.edge(node, Key::Binding(*binding, *usage))
                }
                BindingTarget::Global {
                    name,
                    namespace,
                    usage,
                } => self.edge(node, Key::Global(name.clone(), *namespace, *usage)),
                BindingTarget::Import {
                    module,
                    exported,
                    namespace,
                    type_only,
                    usage: override_usage,
                } => {
                    let usage = override_usage.unwrap_or(usage);
                    if *type_only && usage == Usage::Runtime {
                        self.nodes[node].evidence.missing = true;
                        continue;
                    }
                    match self.resolver.resolve_import(
                        self.facts.versions[&owner.source],
                        module,
                        exported,
                        *namespace,
                        usage,
                    ) {
                        ImportResolution::Binding(identity)
                            if identity.local.namespace == *namespace
                                && self.facts.versions.get(&identity.source.source)
                                    == Some(&identity.source) =>
                        {
                            self.edge(
                                node,
                                Key::Binding(
                                    BindingRef {
                                        source: identity.source.source,
                                        local: identity.local,
                                    },
                                    usage,
                                ),
                            )
                        }
                        ImportResolution::External(identity)
                            if identity.local.namespace == *namespace =>
                        {
                            self.external(node, identity)
                        }
                        ImportResolution::Binding(_) | ImportResolution::External(_) => {
                            self.nodes[node].evidence.missing = true
                        }
                        ImportResolution::Missing => self.nodes[node].evidence.missing = true,
                        ImportResolution::Ambiguous => self.nodes[node].evidence.ambiguous = true,
                    }
                }
                BindingTarget::Candidates(targets) => pending.extend(targets),
                BindingTarget::Ambiguous => self.nodes[node].evidence.ambiguous = true,
            }
        }
    }
    fn expand(&mut self, node: usize) {
        match self.keys[node].clone() {
            Key::Binding(binding, usage) => {
                let facts = self.facts;
                match facts
                    .sources
                    .get(&binding.source)
                    .and_then(|source| source.bindings.get(&binding.local))
                {
                    Some(target) => self.target(node, binding, usage, target),
                    None => self.nodes[node].evidence.missing = true,
                }
            }
            Key::Global(name, namespace, usage) => {
                if let Some(bindings) = self.globals.get(&(name, namespace)).cloned() {
                    for binding in bindings {
                        self.edge(node, Key::Binding(binding, usage));
                    }
                } else {
                    self.nodes[node].evidence.missing = true;
                }
            }
        }
    }
}

/// Seal one declaration snapshot. Every graph node expands once. Evidence has
/// at most two distinct symbols plus refusal bits, so reverse propagation is
/// bounded even for diamonds, long chains and strongly connected aliases.
/// A cycle alone is unresolved; a cycle with an independently proved provider
/// reaches that provider without a recursion limit or an "any" fallback.
pub fn seal<R: Resolver>(
    facts: ProjectFacts,
    resolver: &R,
    reference: SymbolIdentity,
) -> Result<OriginSnapshot, OriginError> {
    if reference.source.owner != facts.revision.owner {
        return Err(OriginError::ForeignOwner);
    }
    if facts.versions.get(&reference.source.source) != Some(&reference.source) {
        return Err(OriginError::UnregisteredReferenceProvider);
    }
    let mut graph = Graph {
        facts: &facts,
        resolver,
        keys: Vec::new(),
        indices: FxHashMap::default(),
        nodes: Vec::new(),
        globals: FxHashMap::default(),
    };
    for source in facts.sources.values() {
        for (name, local) in &source.globals {
            graph
                .globals
                .entry((name.clone(), local.namespace))
                .or_default()
                .push(BindingRef {
                    source: source.revision.source,
                    local: *local,
                });
        }
    }
    let mut names = graph
        .globals
        .keys()
        .filter_map(|(name, space)| (*space == Namespace::Value).then_some(name.clone()))
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    let roots = names
        .iter()
        .map(|name| graph.intern(Key::Global(name.clone(), Namespace::Value, Usage::Runtime)))
        .collect::<Vec<_>>();
    let mut cursor = 0;
    while cursor < graph.nodes.len() {
        graph.expand(cursor);
        cursor += 1;
    }
    let mut work = (0..graph.nodes.len())
        .filter(|index| graph.nodes[*index].evidence != Evidence::default())
        .collect::<VecDeque<_>>();
    while let Some(child) = work.pop_front() {
        let evidence = graph.nodes[child].evidence;
        let dependents = std::mem::take(&mut graph.nodes[child].dependents);
        for &parent in &dependents {
            if graph.nodes[parent].evidence.merge(evidence) {
                work.push_back(parent);
            }
        }
        graph.nodes[child].dependents = dependents;
    }
    let origins = names
        .into_iter()
        .zip(roots)
        .map(|(name, index)| (name, graph.nodes[index].evidence.origin(reference)))
        .collect();
    Ok(OriginSnapshot {
        revision: facts.revision,
        reference,
        origins,
    })
}
