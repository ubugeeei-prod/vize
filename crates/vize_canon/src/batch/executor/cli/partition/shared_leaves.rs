//! Bounded duplication of cheap, multiply imported plain script leaves.

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_span::SourceType;

use super::{VirtualFile, normalize_join, resolve_virtual_import};
use vize_carton::{FxHashMap, FxHashSet, profiler::global_profiler};

// The measured 62-byte shared.ts costs 372 duplicate bytes at seven shards.
// Keep the generalization small: at most 1 KiB per leaf, 32 KiB total extra
// generated content and 1% of existing partitioned content. These fixed limits
// do not change the existing 25% minimum predicted shard saving.
const MAX_LEAF_BYTES: usize = 1024;
const MAX_DUPLICATE_BYTES: usize = 32 * 1024;

pub(super) fn select(
    files: &[&VirtualFile],
    imports: &[Vec<&str>],
    index_by_virtual: &FxHashMap<&Path, usize>,
    servers: usize,
) -> FxHashSet<usize> {
    let mut importers = vec![FxHashSet::default(); files.len()];
    for (index, (file, specifiers)) in files.iter().zip(imports).enumerate() {
        let Some(base) = file.virtual_path.parent() else {
            continue;
        };
        for &specifier in specifiers {
            if !specifier.starts_with("./") && !specifier.starts_with("../") {
                continue;
            }
            let target = normalize_join(base, specifier);
            if let Some(target_index) = resolve_virtual_import(&target, index_by_virtual)
                && let Some(importers) = importers.get_mut(target_index)
            {
                importers.insert(index);
            }
        }
    }

    let budget = files
        .iter()
        .map(|file| file.content.len())
        .sum::<usize>()
        .saturating_div(100)
        .min(MAX_DUPLICATE_BYTES);
    let mut duplicate_bytes = 0;
    let mut selected = FxHashSet::default();
    // Registered virtual files are sorted, so budget selection stays stable.
    for (index, (file, importers)) in files.iter().zip(&importers).enumerate() {
        if importers.len() < 2 || !is_leaf(file) {
            continue;
        }
        let extra = file.content.len().saturating_mul(servers.saturating_sub(1));
        if extra > budget.saturating_sub(duplicate_bytes) {
            continue;
        }
        duplicate_bytes += extra;
        selected.insert(index);
    }
    selected
}

fn is_leaf(file: &VirtualFile) -> bool {
    file.original_path
        .extension()
        .is_some_and(|extension| extension == "ts" || extension == "js")
        && file.content.len() <= MAX_LEAF_BYTES
        // Conservative rejection avoids accepting any
        // transitive dependency syntax the graph's quoted scan might miss:
        // multiline exports, dynamic/import-type calls, require, references
        // and escaped identifiers. Comments/strings may reject a safe leaf;
        // that only keeps its existing connected-component behavior.
        && !["import", "from", "require", "<reference", "\\"]
            .iter()
            .any(|token| file.content.contains(token))
}

// A formerly dominant component may have kept all global script roots
// together. Run this screen only for a plan that would actually split it.
pub(super) fn has_explicit_modules(files: &[&VirtualFile]) -> bool {
    files.iter().all(|file| {
        file.original_path
            .extension()
            .is_some_and(|extension| extension == "vue")
            || is_explicit_module(file)
    })
}

fn is_explicit_module(file: &VirtualFile) -> bool {
    let Ok(source_type) = SourceType::from_path(&file.original_path) else {
        return false;
    };
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &file.content, source_type).parse();
    !parsed.panicked
        && parsed.diagnostics.is_empty()
        && parsed.program.body.iter().any(|statement| {
            matches!(
                statement,
                Statement::ImportDeclaration(_)
                    | Statement::ExportNamedDeclaration(_)
                    | Statement::ExportAllDeclaration(_)
                    | Statement::ExportDefaultDeclaration(_)
                    | Statement::TSImportEqualsDeclaration(_)
            )
        })
}

pub(super) fn record_selection(
    files: &[&VirtualFile],
    selected: &FxHashSet<usize>,
    servers: usize,
) {
    let duplicate_bytes = selected
        .iter()
        .filter_map(|&index| files.get(index))
        .map(|file| file.content.len())
        .sum::<usize>()
        .saturating_mul(servers.saturating_sub(1));
    global_profiler().record_counter("canon.corsa.cli.shared_leaf.files", selected.len() as u64);
    global_profiler().record_counter(
        "canon.corsa.cli.shared_leaf.duplicate_bytes",
        duplicate_bytes as u64,
    );
}

#[cfg(test)]
mod tests;
