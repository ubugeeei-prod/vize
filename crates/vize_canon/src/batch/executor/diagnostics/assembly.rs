//! `vize check`'s side of the one diagnostic post-pass (P4-5a).
//!
//! Backends decode their output into [`RawDiagnostic`]s; this module resolves
//! the file each one names, hands every authored file's diagnostics to
//! [`crate::projection::assemble_diagnostics`] once, and renders the result
//! as batch [`Diagnostic`]s (backend line/column encoding, authored paths).
//! Nothing here decides whether a diagnostic exists or where it lands.

use std::path::{Path, PathBuf};

use vize_carton::{FxHashMap, String};

use super::{DiagnosticMapper, OriginalPosition, VirtualFile};
use crate::batch::{Diagnostic, SfcBlockType};
use crate::projection::{
    AssembledOrigin, AssemblyPolicy, AuthoredSource, FinishedDiagnostic, ProjectedDocument,
    assemble::is_reportable, assemble_diagnostics,
};

/// A backend diagnostic in the coordinates of the file the checker saw.
#[derive(Debug, Clone)]
pub(in crate::batch::executor) struct RawDiagnostic {
    pub(in crate::batch::executor) virtual_path: PathBuf,
    pub(in crate::batch::executor) line: u32,
    pub(in crate::batch::executor) column: u32,
    pub(in crate::batch::executor) end: Option<(u32, u32)>,
    pub(in crate::batch::executor) code: Option<u32>,
    pub(in crate::batch::executor) severity: u8,
    pub(in crate::batch::executor) message: String,
}

/// Diagnostics of one authored file, over the projected documents they name.
#[derive(Default)]
struct AuthoredGroup<'a> {
    documents: Vec<&'a VirtualFile>,
    finished: Vec<FinishedDiagnostic<()>>,
}

impl<'a> AuthoredGroup<'a> {
    fn document(&mut self, file: &'a VirtualFile) -> usize {
        match self
            .documents
            .iter()
            .position(|document| document.virtual_path == file.virtual_path)
        {
            Some(index) => index,
            None => {
                self.documents.push(file);
                self.documents.len() - 1
            }
        }
    }
}

impl DiagnosticMapper<'_> {
    fn policy(&self) -> AssemblyPolicy {
        AssemblyPolicy {
            report_unused: self.preserve_unused_diagnostics,
        }
    }

    /// Whether a diagnostic the checker reported with no file position is
    /// reportable (the project-level half of the same rules).
    pub(in crate::batch::executor) fn is_reportable_project_diagnostic(
        &self,
        code: Option<u32>,
        message: &str,
    ) -> bool {
        is_reportable(code, None, message, self.policy())
    }

    /// Assemble decoded diagnostics. `owns` selects the authored files whose
    /// template diagnostic directives this call evaluates even when the
    /// checker reported nothing for them (a sharded run evaluates each file in
    /// the shard that owns it).
    pub(in crate::batch::executor) fn assemble(
        &mut self,
        raws: Vec<RawDiagnostic>,
        owns: &dyn Fn(&Path) -> bool,
    ) -> Vec<Diagnostic> {
        let project = self.project;
        let policy = self.policy();
        let mut output = Vec::new();
        let mut groups: FxHashMap<PathBuf, AuthoredGroup<'_>> = FxHashMap::default();
        for raw in raws {
            if project.skips_typescript_diagnostics(&raw.virtual_path) {
                continue;
            }
            let Some(file) = project.find_by_diagnostic_virtual(&raw.virtual_path) else {
                if let Some(diagnostic) = self.unprojected(raw, policy) {
                    output.push(diagnostic);
                }
                continue;
            };
            let Some(start) = self.virtual_offset(file, raw.line, raw.column) else {
                continue;
            };
            let end = raw
                .end
                .and_then(|(line, column)| self.virtual_offset(file, line, column))
                .unwrap_or(start)
                .max(start);
            let group = groups.entry(file.original_path.clone()).or_default();
            let document = group.document(file);
            group.finished.push(FinishedDiagnostic {
                document,
                start: start as usize,
                end: end as usize,
                code: raw.code,
                severity: Some(raw.severity),
                message: raw.message,
                payload: (),
            });
        }
        for file in project.virtual_files_sorted() {
            if is_vue(&file.original_path)
                && owns(&file.original_path)
                && !groups.contains_key(&file.original_path)
                && project
                    .original_content_for_virtual(&file.virtual_path)
                    .is_some_and(|source| source.contains("@vue-"))
            {
                groups
                    .entry(file.original_path.clone())
                    .or_default()
                    .document(file);
            }
        }
        let mut groups: Vec<_> = groups.into_iter().collect();
        groups.sort_by(|left, right| left.0.cmp(&right.0));
        for (original_path, group) in groups {
            self.assemble_file(&original_path, group, policy, &mut output);
        }
        output
    }

    fn assemble_file(
        &mut self,
        original_path: &Path,
        group: AuthoredGroup<'_>,
        policy: AssemblyPolicy,
        output: &mut Vec<Diagnostic>,
    ) {
        let project = self.project;
        let Some(&canonical) = group.documents.first() else {
            return;
        };
        let Some(content) = project.original_content_for_virtual(&canonical.virtual_path) else {
            return;
        };
        let sfc_map = canonical.source_map.sfc_map.as_ref();
        let authored = if is_vue(original_path) {
            match sfc_map {
                Some(map) => AuthoredSource::vue_with_template(content, map.template_block()),
                None => AuthoredSource::vue(content),
            }
        } else {
            AuthoredSource::script(content)
        };
        let documents: Vec<_> = group
            .documents
            .iter()
            .map(|file| ProjectedDocument {
                generated: &file.content,
                import_map: &file.source_map.import_map,
                mapping: file.source_map.sfc_map.as_ref().map(|map| map.projection()),
                tsx: file
                    .virtual_path
                    .extension()
                    .is_some_and(|extension| extension == "tsx"),
            })
            .collect();
        for assembled in assemble_diagnostics(&authored, &documents, group.finished, policy) {
            let Some(cached) = self.original_source(original_path) else {
                continue;
            };
            let Some((line, column)) = cached
                .line_index
                .offset_to_line_col(&cached.content, assembled.start as u32)
            else {
                continue;
            };
            let block_type = match assembled.origin {
                AssembledOrigin::Checker(()) => {
                    sfc_map.map(|map| map.block_type_at(assembled.start as u32))
                }
                AssembledOrigin::MissingVueImport(()) => None,
                AssembledOrigin::UnusedExpectation => Some(SfcBlockType::Template),
            };
            let original = OriginalPosition {
                path: original_path.to_path_buf(),
                line,
                column,
                block_type,
            };
            output.push(Diagnostic {
                message: self.authored_message(&original, assembled.message),
                file: original.path,
                line,
                column,
                code: assembled.code,
                severity: assembled.severity.unwrap_or(1),
                block_type,
            });
        }
    }

    /// A diagnostic in a file the project reads but does not project (a
    /// registered diagnostic path outside the virtual files).
    fn unprojected(&mut self, raw: RawDiagnostic, policy: AssemblyPolicy) -> Option<Diagnostic> {
        if !is_reportable(raw.code, Some(raw.severity), &raw.message, policy) {
            return None;
        }
        let original = self
            .project
            .diagnostic_position(&raw.virtual_path, raw.line, raw.column)?;
        if raw.code == Some(6133) && is_vue(&original.path) {
            return None;
        }
        Some(Diagnostic {
            message: self.authored_message(&original, raw.message),
            line: original.line,
            column: original.column,
            file: original.path,
            code: raw.code,
            severity: raw.severity,
            block_type: original.block_type,
        })
    }
}

fn is_vue(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "vue")
}
