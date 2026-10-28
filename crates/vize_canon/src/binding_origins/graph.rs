//! Iterative dependency closure and bounded evidence propagation.
use super::{
    BindingOrigin, BindingRef, BindingTarget, ImportResolution, Namespace, OriginError,
    OriginSnapshot, ProjectFacts, Resolver, SourceRevision, SymbolIdentity, Usage,
};
use std::collections::VecDeque;
use vize_carton::{FxHashMap, String};

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
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ImportKey<'a> {
    from: SourceRevision,
    module: &'a str,
    exported: &'a str,
    namespace: Namespace,
    usage: Usage,
}

#[derive(Default)]
struct Node {
    evidence: Evidence,
    dependents: Vec<usize>,
}
struct Graph<'f, 's, R> {
    facts: &'f ProjectFacts<'s>,
    resolver: &'f R,
    keys: Vec<Key>,
    indices: FxHashMap<Key, usize>,
    nodes: Vec<Node>,
    globals: FxHashMap<(String, Namespace), Vec<BindingRef>>,
    resolutions: FxHashMap<ImportKey<'s>, ImportResolution>,
}
impl<'s, R: Resolver> Graph<'_, 's, R> {
    fn node(&self, index: usize) -> Result<&Node, OriginError> {
        self.nodes.get(index).ok_or(OriginError::InvalidGraphIndex)
    }
    fn node_mut(&mut self, index: usize) -> Result<&mut Node, OriginError> {
        self.nodes
            .get_mut(index)
            .ok_or(OriginError::InvalidGraphIndex)
    }
    fn source(&self, id: u32) -> Result<SourceRevision, OriginError> {
        self.facts
            .versions
            .get(&id)
            .copied()
            .ok_or(OriginError::UnregisteredSource)
    }
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
    fn edge(&mut self, parent: usize, key: Key) -> Result<(), OriginError> {
        let child = self.intern(key);
        self.node_mut(child)?.dependents.push(parent);
        Ok(())
    }
    fn external(&mut self, node: usize, identity: SymbolIdentity) -> Result<(), OriginError> {
        if self.facts.versions.get(&identity.source.source) == Some(&identity.source) {
            self.node_mut(node)?.evidence.add_symbol(identity);
        } else {
            self.node_mut(node)?.evidence.missing = true;
        }
        Ok(())
    }
    fn resolve_import(&mut self, key: ImportKey<'s>) -> ImportResolution {
        let resolver = self.resolver;
        *self.resolutions.entry(key).or_insert_with(|| {
            resolver.resolve_import(key.from, key.module, key.exported, key.namespace, key.usage)
        })
    }
    fn target(
        &mut self,
        node: usize,
        owner: BindingRef,
        usage: Usage,
        target: &'s BindingTarget,
    ) -> Result<(), OriginError> {
        let mut pending = Vec::new();
        let mut first = Some(target);
        while let Some(target) = first.take().or_else(|| pending.pop()) {
            match target {
                BindingTarget::Opaque => {
                    let source = self.source(owner.source)?;
                    self.node_mut(node)?.evidence.add_symbol(SymbolIdentity {
                        source,
                        local: owner.local,
                    });
                }
                BindingTarget::Alias { binding, usage } => {
                    self.edge(node, Key::Binding(*binding, *usage))?
                }
                BindingTarget::Global {
                    name,
                    namespace,
                    usage,
                } => self.edge(node, Key::Global(name.clone(), *namespace, *usage))?,
                BindingTarget::Import {
                    module,
                    exported,
                    namespace,
                    type_only,
                    usage: override_usage,
                } => {
                    let usage = override_usage.unwrap_or(usage);
                    if *type_only && usage == Usage::Runtime {
                        self.node_mut(node)?.evidence.missing = true;
                        continue;
                    }
                    let from = self.source(owner.source)?;
                    match self.resolve_import(ImportKey {
                        from,
                        module,
                        exported,
                        namespace: *namespace,
                        usage,
                    }) {
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
                            )?
                        }
                        ImportResolution::External(identity)
                            if identity.local.namespace == *namespace =>
                        {
                            self.external(node, identity)?
                        }
                        ImportResolution::Binding(_)
                        | ImportResolution::External(_)
                        | ImportResolution::Missing => self.node_mut(node)?.evidence.missing = true,
                        ImportResolution::Ambiguous => {
                            self.node_mut(node)?.evidence.ambiguous = true
                        }
                    }
                }
                BindingTarget::Candidates(targets) => pending.extend(targets),
                BindingTarget::Ambiguous => self.node_mut(node)?.evidence.ambiguous = true,
            }
        }
        Ok(())
    }
    fn expand(&mut self, index: usize) -> Result<(), OriginError> {
        match self
            .keys
            .get(index)
            .cloned()
            .ok_or(OriginError::InvalidGraphIndex)?
        {
            Key::Binding(binding, usage) => {
                let facts = self.facts;
                match facts
                    .sources
                    .get(&binding.source)
                    .copied()
                    .and_then(|source| source.bindings.get(&binding.local))
                {
                    Some(target) => self.target(index, binding, usage, target)?,
                    None => self.node_mut(index)?.evidence.missing = true,
                }
            }
            Key::Global(name, namespace, usage) => {
                if let Some(bindings) = self.globals.get(&(name, namespace)).cloned() {
                    for binding in bindings {
                        self.edge(index, Key::Binding(binding, usage))?;
                    }
                } else {
                    self.node_mut(index)?.evidence.missing = true;
                }
            }
        }
        Ok(())
    }
}

/// Expand requested roots once and propagate at most two identities/refusal bits.
/// This neither traverses unrelated global dependencies nor retains source ASTs.
pub fn seal<R: Resolver>(
    facts: ProjectFacts<'_>,
    resolver: &R,
    reference: SymbolIdentity,
    requested_globals: &[&str],
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
        resolutions: FxHashMap::default(),
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
    let mut names = requested_globals
        .iter()
        .map(|name| String::from(*name))
        .collect::<Vec<_>>();
    names.sort();
    names.dedup();
    let roots = names
        .iter()
        .map(|name| graph.intern(Key::Global(name.clone(), Namespace::Value, Usage::Runtime)))
        .collect::<Vec<_>>();
    let mut cursor = 0;
    while cursor < graph.nodes.len() {
        graph.expand(cursor)?;
        cursor += 1;
    }
    let mut work = graph
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| (node.evidence != Evidence::default()).then_some(index))
        .collect::<VecDeque<_>>();
    while let Some(child) = work.pop_front() {
        let node = graph.node_mut(child)?;
        let evidence = node.evidence;
        let dependents = std::mem::take(&mut node.dependents);
        for &parent in &dependents {
            if graph.node_mut(parent)?.evidence.merge(evidence) {
                work.push_back(parent);
            }
        }
        graph.node_mut(child)?.dependents = dependents;
    }
    let origins = names
        .into_iter()
        .zip(roots)
        .map(|(name, index)| Ok((name, graph.node(index)?.evidence.origin(reference))))
        .collect::<Result<FxHashMap<_, _>, OriginError>>()?;
    Ok(OriginSnapshot {
        revision: facts.revision,
        reference,
        origins,
    })
}
