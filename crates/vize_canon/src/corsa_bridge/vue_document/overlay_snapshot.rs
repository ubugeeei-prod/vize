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
        let mut sources = PersistentOverlayMap::default();
        for (path, text) in entries {
            let path = vize_carton::path::canonicalize_non_verbatim(&path);
            sources.insert(path, text);
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

fn node(
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
        return node(path, text, content_identity, priority, None, None);
    };
    match path.cmp(&root.path) {
        std::cmp::Ordering::Equal => node(
            path,
            text,
            content_identity,
            root.priority,
            root.left.clone(),
            root.right.clone(),
        ),
        std::cmp::Ordering::Less => {
            let left = insert(root.left.clone(), path, text, content_identity);
            let rebuilt = node(
                root.path.clone(),
                Arc::clone(&root.text),
                root.content_identity,
                root.priority,
                Some(Arc::clone(&left)),
                root.right.clone(),
            );
            if left.priority < rebuilt.priority {
                node(
                    left.path.clone(),
                    Arc::clone(&left.text),
                    left.content_identity,
                    left.priority,
                    left.left.clone(),
                    Some(node(
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
            let rebuilt = node(
                root.path.clone(),
                Arc::clone(&root.text),
                root.content_identity,
                root.priority,
                root.left.clone(),
                Some(Arc::clone(&right)),
            );
            if right.priority < rebuilt.priority {
                node(
                    right.path.clone(),
                    Arc::clone(&right.text),
                    right.content_identity,
                    right.priority,
                    Some(node(
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
        (Some(left), Some(right)) if left.priority <= right.priority => Some(node(
            left.path.clone(),
            Arc::clone(&left.text),
            left.content_identity,
            left.priority,
            left.left.clone(),
            merge(left.right.clone(), Some(right)),
        )),
        (Some(left), Some(right)) => Some(node(
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
        std::cmp::Ordering::Less => Some(node(
            root.path.clone(),
            Arc::clone(&root.text),
            root.content_identity,
            root.priority,
            remove(root.left.clone(), path),
            root.right.clone(),
        )),
        std::cmp::Ordering::Greater => Some(node(
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
