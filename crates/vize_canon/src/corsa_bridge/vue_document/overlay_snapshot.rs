//! Revision-scoped identities for one complete set of editor overlays.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use vize_carton::FxHashMap;

/// Immutable open-buffer snapshot shared by every semantic request in one
/// editor revision.
#[derive(Clone, Default)]
pub struct CorsaOverlaySnapshot {
    sources: Arc<PersistentOverlayMap>,
    fingerprint: OverlayFingerprint,
}

#[derive(Clone, Default)]
pub(in crate::corsa_bridge) struct OverlayFingerprint {
    pub(in crate::corsa_bridge) identity: u64,
    pub(in crate::corsa_bridge) inputs: Arc<PersistentOverlayMap>,
}

impl CorsaOverlaySnapshot {
    /// Capture content identities once for one owner-provided revision.
    pub fn new(identity: u64, entries: Vec<(PathBuf, Arc<String>)>) -> Self {
        let sources = PersistentOverlayMap::from_entries(entries);
        let sources = Arc::new(sources);
        Self {
            sources: Arc::clone(&sources),
            fingerprint: OverlayFingerprint {
                identity,
                inputs: sources,
            },
        }
    }

    /// Derive the next immutable overlay set by path-copying only changed
    /// search-tree nodes. Unchanged paths and texts stay pointer-identical.
    pub fn revise(
        &self,
        identity: u64,
        upserts: Vec<(PathBuf, Arc<String>)>,
        removals: Vec<PathBuf>,
    ) -> Self {
        let mut sources = (*self.sources).clone();
        for path in removals {
            sources.remove(&vize_carton::path::canonicalize_non_verbatim(&path));
        }
        for (path, text) in upserts {
            sources.insert(vize_carton::path::canonicalize_non_verbatim(&path), text);
        }
        let sources = Arc::new(sources);
        Self {
            sources: Arc::clone(&sources),
            fingerprint: OverlayFingerprint {
                identity,
                inputs: sources,
            },
        }
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&PathBuf, &Arc<String>)> {
        self.sources.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    pub(in crate::corsa_bridge) fn sources(&self) -> &dyn OverlaySources {
        &*self.sources
    }

    pub(in crate::corsa_bridge) fn fingerprint(&self) -> &OverlayFingerprint {
        &self.fingerprint
    }
}

/// Persistent deterministic treap. One edit allocates only the search path;
/// immutable snapshots already held by concurrent requests remain valid.
#[derive(Clone, Default)]
pub(in crate::corsa_bridge) struct PersistentOverlayMap {
    root: Option<Arc<OverlayNode>>,
}

struct OverlayNode {
    path: PathBuf,
    text: Arc<String>,
    content_identity: u64,
    priority: u64,
    left: Option<Arc<Self>>,
    right: Option<Arc<Self>>,
    len: usize,
}

impl PersistentOverlayMap {
    /// Build a cold revision with one node allocation per retained source.
    ///
    /// Calling persistent `insert` N times path-copies every search route and
    /// therefore allocates O(N log N) nodes before the first request. Build the
    /// deterministic treap as a Cartesian tree instead; sorting establishes
    /// key order and the monotonic stack links the same priorities in O(N)
    /// additional work and O(N) node allocations.
    fn from_entries(entries: Vec<(PathBuf, Arc<String>)>) -> Self {
        let mut entries = entries
            .into_iter()
            .enumerate()
            .map(|(order, (path, text))| {
                (
                    vize_carton::path::canonicalize_non_verbatim(&path),
                    text,
                    order,
                )
            })
            .collect::<Vec<_>>();
        entries.sort_unstable_by(|left, right| {
            left.0.cmp(&right.0).then_with(|| left.2.cmp(&right.2))
        });

        // Match repeated map insertion: the last value for an equal canonical
        // path wins. DocumentStore paths are unique, but the public constructor
        // keeps this deterministic compatibility for other callers.
        let mut unique: Vec<(PathBuf, Arc<String>)> = Vec::with_capacity(entries.len());
        for (path, text, _) in entries {
            if let Some((previous, value)) = unique.last_mut()
                && previous == &path
            {
                *value = text;
            } else {
                unique.push((path, text));
            }
        }

        let mut nodes = unique
            .into_iter()
            .map(|(path, text)| {
                let content_identity = overlay_content_identity(&text);
                let priority = path_priority(&path);
                OwnedOverlayNode {
                    path,
                    text,
                    content_identity,
                    priority,
                    left: None,
                    right: None,
                }
            })
            .collect::<Vec<_>>();
        let mut stack = Vec::<usize>::with_capacity(nodes.len());
        for index in 0..nodes.len() {
            let Some(priority) = nodes.get(index).map(|node| node.priority) else {
                continue;
            };
            let mut left = None;
            while stack.last().is_some_and(|parent| {
                nodes
                    .get(*parent)
                    .is_some_and(|node| node.priority > priority)
            }) {
                left = stack.pop();
            }
            if let Some(parent) = stack.last().copied()
                && let Some(node) = nodes.get_mut(parent)
            {
                node.right = Some(index);
            }
            if let Some(node) = nodes.get_mut(index) {
                node.left = left;
            }
            stack.push(index);
        }
        let mut nodes = nodes.into_iter().map(Some).collect::<Vec<_>>();
        let root = stack
            .first()
            .copied()
            .and_then(|root| freeze_bulk_node(&mut nodes, root));
        Self { root }
    }

    fn insert(&mut self, path: PathBuf, text: Arc<String>) {
        let content_identity = overlay_content_identity(&text);
        self.root = Some(insert(self.root.take(), path, text, content_identity));
    }

    fn remove(&mut self, path: &Path) {
        self.root = remove(self.root.take(), path);
    }

    fn node(&self, path: &Path) -> Option<&OverlayNode> {
        let mut current = self.root.as_deref();
        while let Some(node) = current {
            current = match path.cmp(&node.path) {
                std::cmp::Ordering::Less => node.left.as_deref(),
                std::cmp::Ordering::Greater => node.right.as_deref(),
                std::cmp::Ordering::Equal => return Some(node),
            };
        }
        None
    }

    pub(in crate::corsa_bridge) fn content_identity(&self, path: &Path) -> Option<u64> {
        self.node(path).map(|node| node.content_identity)
    }

    pub(in crate::corsa_bridge) fn iter(&self) -> PersistentOverlayIter<'_> {
        PersistentOverlayIter::new(self.root.as_deref(), self.len())
    }

    pub(in crate::corsa_bridge) fn len(&self) -> usize {
        self.root.as_ref().map_or(0, |root| root.len)
    }
}

struct OwnedOverlayNode {
    path: PathBuf,
    text: Arc<String>,
    content_identity: u64,
    priority: u64,
    left: Option<usize>,
    right: Option<usize>,
}

fn freeze_bulk_node(
    nodes: &mut [Option<OwnedOverlayNode>],
    index: usize,
) -> Option<Arc<OverlayNode>> {
    let node = nodes.get_mut(index)?.take()?;
    let left = match node.left {
        Some(child) => Some(freeze_bulk_node(nodes, child)?),
        None => None,
    };
    let right = match node.right {
        Some(child) => Some(freeze_bulk_node(nodes, child)?),
        None => None,
    };
    Some(make_node(
        node.path,
        node.text,
        node.content_identity,
        node.priority,
        left,
        right,
    ))
}

pub(in crate::corsa_bridge) struct PersistentOverlayIter<'a> {
    stack: Vec<&'a OverlayNode>,
    remaining: usize,
}

impl<'a> PersistentOverlayIter<'a> {
    fn new(root: Option<&'a OverlayNode>, remaining: usize) -> Self {
        let mut iter = Self {
            stack: Vec::new(),
            remaining,
        };
        iter.push_left(root);
        iter
    }

    fn push_left(&mut self, mut node: Option<&'a OverlayNode>) {
        while let Some(current) = node {
            self.stack.push(current);
            node = current.left.as_deref();
        }
    }
}

impl<'a> Iterator for PersistentOverlayIter<'a> {
    type Item = (&'a PathBuf, &'a Arc<String>);

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        self.push_left(node.right.as_deref());
        self.remaining -= 1;
        Some((&node.path, &node.text))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl ExactSizeIterator for PersistentOverlayIter<'_> {}

fn make_node(
    path: PathBuf,
    text: Arc<String>,
    content_identity: u64,
    priority: u64,
    left: Option<Arc<OverlayNode>>,
    right: Option<Arc<OverlayNode>>,
) -> Arc<OverlayNode> {
    Arc::new(OverlayNode {
        path,
        text,
        content_identity,
        priority,
        len: 1
            + left.as_ref().map_or(0, |node| node.len)
            + right.as_ref().map_or(0, |node| node.len),
        left,
        right,
    })
}

fn path_priority(path: &Path) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hasher);
    hasher.finish()
}

fn insert(
    root: Option<Arc<OverlayNode>>,
    path: PathBuf,
    text: Arc<String>,
    content_identity: u64,
) -> Arc<OverlayNode> {
    let Some(root) = root else {
        let priority = path_priority(&path);
        return make_node(path, text, content_identity, priority, None, None);
    };
    match path.cmp(&root.path) {
        std::cmp::Ordering::Equal => make_node(
            path,
            text,
            content_identity,
            root.priority,
            root.left.clone(),
            root.right.clone(),
        ),
        std::cmp::Ordering::Less => {
            let left = insert(root.left.clone(), path, text, content_identity);
            let rebuilt = make_node(
                root.path.clone(),
                Arc::clone(&root.text),
                root.content_identity,
                root.priority,
                Some(Arc::clone(&left)),
                root.right.clone(),
            );
            if left.priority < rebuilt.priority {
                make_node(
                    left.path.clone(),
                    Arc::clone(&left.text),
                    left.content_identity,
                    left.priority,
                    left.left.clone(),
                    Some(make_node(
                        rebuilt.path.clone(),
                        Arc::clone(&rebuilt.text),
                        rebuilt.content_identity,
                        rebuilt.priority,
                        left.right.clone(),
                        rebuilt.right.clone(),
                    )),
                )
            } else {
                rebuilt
            }
        }
        std::cmp::Ordering::Greater => {
            let right = insert(root.right.clone(), path, text, content_identity);
            let rebuilt = make_node(
                root.path.clone(),
                Arc::clone(&root.text),
                root.content_identity,
                root.priority,
                root.left.clone(),
                Some(Arc::clone(&right)),
            );
            if right.priority < rebuilt.priority {
                make_node(
                    right.path.clone(),
                    Arc::clone(&right.text),
                    right.content_identity,
                    right.priority,
                    Some(make_node(
                        rebuilt.path.clone(),
                        Arc::clone(&rebuilt.text),
                        rebuilt.content_identity,
                        rebuilt.priority,
                        rebuilt.left.clone(),
                        right.left.clone(),
                    )),
                    right.right.clone(),
                )
            } else {
                rebuilt
            }
        }
    }
}

fn merge(
    left: Option<Arc<OverlayNode>>,
    right: Option<Arc<OverlayNode>>,
) -> Option<Arc<OverlayNode>> {
    match (left, right) {
        (None, right) => right,
        (left, None) => left,
        (Some(left), Some(right)) if left.priority <= right.priority => Some(make_node(
            left.path.clone(),
            Arc::clone(&left.text),
            left.content_identity,
            left.priority,
            left.left.clone(),
            merge(left.right.clone(), Some(right)),
        )),
        (Some(left), Some(right)) => Some(make_node(
            right.path.clone(),
            Arc::clone(&right.text),
            right.content_identity,
            right.priority,
            merge(Some(left), right.left.clone()),
            right.right.clone(),
        )),
    }
}

fn remove(root: Option<Arc<OverlayNode>>, path: &Path) -> Option<Arc<OverlayNode>> {
    let root = root?;
    match path.cmp(&root.path) {
        std::cmp::Ordering::Equal => merge(root.left.clone(), root.right.clone()),
        std::cmp::Ordering::Less => Some(make_node(
            root.path.clone(),
            Arc::clone(&root.text),
            root.content_identity,
            root.priority,
            remove(root.left.clone(), path),
            root.right.clone(),
        )),
        std::cmp::Ordering::Greater => Some(make_node(
            root.path.clone(),
            Arc::clone(&root.text),
            root.content_identity,
            root.priority,
            root.left.clone(),
            remove(root.right.clone(), path),
        )),
    }
}

/// Read-only text lookup shared by borrowed cold inputs and revision-owned
/// snapshots. The editor hot path keeps the latter map by identity instead of
/// reconstructing it for every request.
pub(crate) trait OverlaySources {
    fn len(&self) -> usize;

    fn get<'a>(&'a self, path: &Path) -> Option<&'a str>;

    fn visit<'a>(&'a self, visitor: &mut dyn FnMut(&'a Path, &'a str));

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn contains_key(&self, path: &Path) -> bool {
        self.get(path).is_some()
    }
}

impl OverlaySources for FxHashMap<PathBuf, &str> {
    fn len(&self) -> usize {
        FxHashMap::len(self)
    }

    fn get<'a>(&'a self, path: &Path) -> Option<&'a str> {
        FxHashMap::get(self, path).copied()
    }

    fn visit<'a>(&'a self, visitor: &mut dyn FnMut(&'a Path, &'a str)) {
        for (path, text) in self {
            visitor(path, text);
        }
    }
}

impl OverlaySources for PersistentOverlayMap {
    fn len(&self) -> usize {
        PersistentOverlayMap::len(self)
    }

    fn get<'a>(&'a self, path: &Path) -> Option<&'a str> {
        self.node(path).map(|node| node.text.as_str())
    }

    fn visit<'a>(&'a self, visitor: &mut dyn FnMut(&'a Path, &'a str)) {
        for (path, text) in self.iter() {
            visitor(path, text);
        }
    }
}

pub(in crate::corsa_bridge) fn overlay_content_identity(text: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut input = vize_l0::micro_default_hasher!("canon.context.overlay_input");
    text.hash(&mut input);
    input.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bulk_build_matches_incremental_treap_membership() {
        const SOURCES: usize = 1_024;
        let entries = (0..SOURCES)
            .map(|position| {
                let index = position * 37 % SOURCES;
                (
                    PathBuf::from(format!("/workspace/Component{index:05}.vue")),
                    Arc::new(format!("<template>{index}</template>")),
                )
            })
            .collect::<Vec<_>>();
        let bulk = PersistentOverlayMap::from_entries(entries.clone());
        let mut incremental = PersistentOverlayMap::default();
        for (path, text) in entries {
            incremental.insert(path, text);
        }

        let bulk = bulk
            .iter()
            .map(|(path, text)| (path.clone(), text.as_str().to_owned()))
            .collect::<Vec<_>>();
        let incremental = incremental
            .iter()
            .map(|(path, text)| (path.clone(), text.as_str().to_owned()))
            .collect::<Vec<_>>();
        assert_eq!(bulk, incremental);
        assert_eq!(bulk.len(), SOURCES);
    }

    #[test]
    fn duplicate_paths_keep_the_last_source() {
        let snapshot = CorsaOverlaySnapshot::new(
            1,
            vec![
                (PathBuf::from("/workspace/App.vue"), Arc::new("old".into())),
                (PathBuf::from("/workspace/App.vue"), Arc::new("new".into())),
            ],
        );

        assert_eq!(snapshot.iter().count(), 1);
        assert_eq!(snapshot.iter().next().unwrap().1.as_str(), "new");
    }

    #[test]
    fn one_edit_reuses_unrelated_source_identity() {
        let app = Arc::new("<template>app</template>".to_owned());
        let dependency = Arc::new("export const value = 1".to_owned());
        let snapshot = CorsaOverlaySnapshot::new(
            1,
            vec![
                (PathBuf::from("/workspace/App.vue"), Arc::clone(&app)),
                (
                    PathBuf::from("/workspace/dependency.ts"),
                    Arc::clone(&dependency),
                ),
            ],
        );
        let revised = snapshot.revise(
            2,
            vec![(
                PathBuf::from("/workspace/App.vue"),
                Arc::new("<template>edited</template>".into()),
            )],
            Vec::new(),
        );

        let original_dependency = snapshot
            .iter()
            .find(|(path, _)| path.ends_with("dependency.ts"))
            .unwrap()
            .1;
        let revised_dependency = revised
            .iter()
            .find(|(path, _)| path.ends_with("dependency.ts"))
            .unwrap()
            .1;
        assert!(Arc::ptr_eq(original_dependency, revised_dependency));
        assert_eq!(snapshot.iter().count(), 2);
        assert_eq!(revised.iter().count(), 2);
    }
}
