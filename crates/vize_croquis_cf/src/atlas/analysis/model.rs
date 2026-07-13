//! Owned full-analysis result and source-coordinate projection.

use std::path::PathBuf;

use vize_atlas::{CompilationInput, Product, SourceId};
use vize_carton::{CompactString, String};

use crate::{
    CrossFileDiagnostic, CrossFileDiagnosticKind, CrossFileOptions, CrossFileResult, FileId,
};

/// Project-wide options for one cross-file query closure.
#[derive(Debug, Clone, Default)]
pub struct CrossFileAnalysisRequest {
    pub options: CrossFileOptions,
    pub project_root: Option<PathBuf>,
}

impl CrossFileAnalysisRequest {
    pub fn new(options: CrossFileOptions) -> Self {
        Self {
            options,
            project_root: None,
        }
    }

    pub fn with_project_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.project_root = Some(root.into());
        self
    }
}

/// Open Atlas input used only by the full cross-file analysis provider.
pub struct CrossFileAnalysisInput;

impl CompilationInput for CrossFileAnalysisInput {
    type Value = CrossFileAnalysisRequest;

    const NAME: &'static str = "croquis.cross-file-request";
}

/// The coordinate space used when projecting an analyzer offset to its source.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CrossFileOffsetRegion {
    Script,
    Template,
    TemplateTag,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(super) struct OffsetSegment {
    pub(super) generated_start: u32,
    pub(super) generated_end: u32,
    pub(super) source_start: u32,
}

/// Stable mapping between one analyzer file and one Atlas source.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CrossFileSourceLayout {
    pub(super) file: FileId,
    pub(super) source: SourceId,
    pub(super) path: CompactString,
    pub(super) script: Vec<OffsetSegment>,
    pub(super) template_tag: (u32, u32),
    pub(super) template_content_start: u32,
}

impl CrossFileSourceLayout {
    pub const fn file(&self) -> FileId {
        self.file
    }

    pub const fn source(&self) -> SourceId {
        self.source
    }

    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    pub fn map_offset(&self, region: CrossFileOffsetRegion, offset: u32) -> u32 {
        match region {
            CrossFileOffsetRegion::Script => self.map_script_offset(offset),
            CrossFileOffsetRegion::Template => self.template_content_start.saturating_add(offset),
            CrossFileOffsetRegion::TemplateTag => self.template_tag.0,
        }
    }

    pub fn map_range(&self, region: CrossFileOffsetRegion, start: u32, end: u32) -> (u32, u32) {
        if region == CrossFileOffsetRegion::TemplateTag {
            return self.template_tag;
        }
        (self.map_offset(region, start), self.map_offset(region, end))
    }

    fn map_script_offset(&self, offset: u32) -> u32 {
        let segment = self
            .script
            .iter()
            .find(|segment| offset <= segment.generated_end)
            .or_else(|| self.script.last());
        segment.map_or(offset, |segment| {
            segment
                .source_start
                .saturating_add(offset.saturating_sub(segment.generated_start))
        })
    }
}

/// Full analyzer result plus source provenance needed by every host.
#[derive(Debug)]
pub struct CrossFileAnalysisArtifact {
    pub(super) result: CrossFileResult,
    pub(super) layouts: Vec<CrossFileSourceLayout>,
    pub(super) provide_inject_tree: Option<String>,
}

impl CrossFileAnalysisArtifact {
    pub const fn result(&self) -> &CrossFileResult {
        &self.result
    }

    pub fn layouts(&self) -> &[CrossFileSourceLayout] {
        &self.layouts
    }

    pub fn layout(&self, file: FileId) -> Option<&CrossFileSourceLayout> {
        self.layouts.iter().find(|layout| layout.file == file)
    }

    pub fn layout_for_source(&self, source: SourceId) -> Option<&CrossFileSourceLayout> {
        self.layouts.iter().find(|layout| layout.source == source)
    }

    pub fn provide_inject_tree(&self) -> Option<&str> {
        self.provide_inject_tree.as_deref()
    }

    pub fn diagnostic_range(
        &self,
        diagnostic: &CrossFileDiagnostic,
    ) -> Option<(SourceId, u32, u32)> {
        let layout = self.layout(diagnostic.primary_file)?;
        let region = diagnostic_region(&diagnostic.kind);
        let (start, end) = layout.map_range(
            region,
            diagnostic.primary_offset,
            diagnostic.primary_end_offset,
        );
        Some((layout.source(), start, end))
    }

    pub fn related_offset(&self, file: FileId, offset: u32) -> Option<(SourceId, u32)> {
        let layout = self.layout(file)?;
        Some((
            layout.source(),
            layout.map_offset(CrossFileOffsetRegion::Script, offset),
        ))
    }

    /// Project a related location using the coordinate space implied by the
    /// diagnostic. Most related locations point to declarations in script;
    /// duplicate IDs are the exception and point back into another template.
    pub fn diagnostic_related_offset(
        &self,
        diagnostic: &CrossFileDiagnostic,
        file: FileId,
        offset: u32,
    ) -> Option<(SourceId, u32)> {
        let layout = self.layout(file)?;
        let region = if matches!(
            &diagnostic.kind,
            CrossFileDiagnosticKind::DuplicateElementId { .. }
        ) {
            CrossFileOffsetRegion::Template
        } else {
            CrossFileOffsetRegion::Script
        };
        Some((layout.source(), layout.map_offset(region, offset)))
    }
}

fn diagnostic_region(kind: &CrossFileDiagnosticKind) -> CrossFileOffsetRegion {
    use CrossFileDiagnosticKind::*;
    if matches!(
        kind,
        MultiRootMissingAttrs | InheritAttrsDisabledUnused | UnusedFallthroughAttrs { .. }
    ) {
        return CrossFileOffsetRegion::TemplateTag;
    }
    if matches!(
        kind,
        UnmatchedEventListener { .. }
            | UndeclaredProp { .. }
            | MissingRequiredProp { .. }
            | PropTypeMismatch { .. }
            | UndefinedSlot { .. }
            | UnregisteredComponent { .. }
            | DuplicateElementId { .. }
            | NonUniqueIdInLoop { .. }
            | EventModifierIssue { .. }
            | BrowserApiInSsr { .. }
            | UncaughtErrorBoundary
    ) {
        CrossFileOffsetRegion::Template
    } else {
        CrossFileOffsetRegion::Script
    }
}

/// Opt-in execution of the complete cross-file analyzer.
pub struct CrossFileAnalysisProduct;

impl Product for CrossFileAnalysisProduct {
    type Value = CrossFileAnalysisArtifact;

    const NAME: &'static str = "croquis.cross-file-analysis";
}
