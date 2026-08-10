//! Breadth-first traversal shared by Vue and script virtual-document hosts.

use std::collections::VecDeque;
use std::path::PathBuf;

use vize_carton::{FxHashMap, FxHashSet, String};

use super::{DependencyScan, ImportQueue, parent_dir, queue_imports};
use crate::batch::ImportRewriter;
use crate::corsa_bridge::vue_document::{
    CorsaVueVirtualDependency, CorsaVueVirtualDocumentOptions,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn collect_queued_documents(
    documents: &mut Vec<(String, String)>,
    mut dependencies: Option<&mut Vec<CorsaVueVirtualDependency>>,
    options: CorsaVueVirtualDocumentOptions,
    rewriter: &ImportRewriter,
    alias_context: &crate::corsa_bridge::vue_dependencies_alias::AliasContext,
    overlays: &FxHashMap<PathBuf, &str>,
    visited_vue: &mut FxHashSet<PathBuf>,
    visited_ts: &mut FxHashSet<PathBuf>,
    mut queue: VecDeque<DependencyScan>,
) {
    while let Some(scan) = queue.pop_front() {
        let (dir, source_type, code) = match scan {
            DependencyScan::Vue {
                dir,
                source_type,
                pre_rewrite_code,
            } => (dir, source_type, pre_rewrite_code),
            DependencyScan::Script {
                path,
                source_type,
                content,
            } => (parent_dir(&path), source_type, content),
        };
        queue_imports(
            ImportQueue {
                documents,
                dependencies: dependencies.as_deref_mut(),
                queue: &mut queue,
                visited_vue,
                visited_ts,
                overlays,
            },
            options,
            rewriter,
            alias_context,
            &dir,
            &code,
            source_type,
        );
    }
}
