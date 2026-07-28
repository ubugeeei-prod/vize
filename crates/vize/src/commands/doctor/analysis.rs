//! Whole-project graph construction with authored SFC coordinate recovery.

use std::path::Path;
use vize_armature::Parser;
use vize_atelier_sfc::{
    SfcParseOptions,
    croquis::{SfcCroquisOptions, analyze_sfc_descriptor_with_context},
    parse_sfc,
};
use vize_carton::{Allocator, FxHashMap};
use vize_croquis::{EffectGraphScript, build_effect_graph_from_sfc_scripts};
use vize_croquis_cf::{CrossFileAnalyzer, CrossFileDiagnosticKind, CrossFileOptions, FileId};
use vize_doctor::{
    DoctorFinding, DoctorReport, application_analysis::report_from_application_graph,
};

use super::{DoctorError, DoctorSource, canonical_sfc};

#[derive(Clone, Copy, Debug, Default)]
struct SourceBlock {
    authored_start: u32,
    length: u32,
}

#[derive(Clone, Copy, Debug, Default)]
struct SfcSourceMap {
    source_length: u32,
    script: Option<SourceBlock>,
    setup: Option<SourceBlock>,
    template: Option<SourceBlock>,
}

pub(super) fn analyze_application(
    root: &Path,
    sources: &[DoctorSource],
    public_sfc: bool,
) -> Result<DoctorReport, DoctorError> {
    let mut analyzer = CrossFileAnalyzer::with_project_root(doctor_options(), root);
    let mut source_maps = FxHashMap::default();
    let mut public_sfc_findings = Vec::new();
    let mut ordered_sources = sources.iter().collect::<Vec<_>>();
    ordered_sources.sort_unstable_by(|left, right| left.path.cmp(&right.path));

    for source in ordered_sources {
        if source
            .path
            .extension()
            .is_some_and(|extension| extension == "vue")
        {
            add_sfc(
                &mut analyzer,
                &mut source_maps,
                &mut public_sfc_findings,
                source,
                public_sfc,
            )?;
        } else {
            analyzer.add_file(&source.path, source.source.as_str());
        }
    }

    analyzer.rebuild_import_edges();
    analyzer.rebuild_component_edges();
    let mut result = analyzer.analyze();
    normalize_sfc_diagnostics(&mut result.diagnostics, &source_maps);
    let graph_report =
        report_from_application_graph(".", &analyzer, &result).map_err(DoctorError::from)?;
    Ok(DoctorReport::new(
        graph_report.workspace(),
        graph_report
            .findings()
            .iter()
            .cloned()
            .chain(public_sfc_findings),
    ))
}

fn add_sfc(
    analyzer: &mut CrossFileAnalyzer,
    source_maps: &mut FxHashMap<FileId, SfcSourceMap>,
    public_sfc_findings: &mut Vec<DoctorFinding>,
    source: &DoctorSource,
    public_sfc: bool,
) -> Result<(), DoctorError> {
    let filename = source.path.to_string_lossy();
    let doctor_path = filename.replace('\\', "/");
    let descriptor = parse_sfc(
        source.source.as_str(),
        SfcParseOptions {
            filename: filename.as_ref().into(),
            ..Default::default()
        },
    )
    .map_err(|error| DoctorError::ParseSfc {
        path: source.path.clone(),
        message: error.message,
    })?;
    if public_sfc && let Some(finding) = canonical_sfc::finding(&doctor_path, &descriptor) {
        public_sfc_findings.push(finding);
    }
    let analysis = if let Some(template) = descriptor.template.as_ref() {
        let allocator = Allocator::with_capacity((template.content.len() * 4).max(64 * 1024));
        let parser = Parser::new(allocator.as_bump(), template.content.as_ref());
        let (root, errors) = parser.parse();
        if let Some(error) = errors.iter().find(|error| !error.is_recoverable()) {
            return Err(DoctorError::ParseSfc {
                path: source.path.clone(),
                message: error.message.clone(),
            });
        }
        analyze_sfc_descriptor_with_context(&descriptor, Some(&root), SfcCroquisOptions::full())
    } else {
        analyze_sfc_descriptor_with_context(&descriptor, None, SfcCroquisOptions::full())
    };
    let effect_summary = build_effect_graph_from_sfc_scripts(
        descriptor
            .script
            .as_ref()
            .map(|block| EffectGraphScript::new(block.content.as_ref(), block.lang.as_deref())),
        descriptor
            .script_setup
            .as_ref()
            .map(|block| EffectGraphScript::new(block.content.as_ref(), block.lang.as_deref())),
    )
    .summary();
    let file_id = analyzer.add_file_with_analysis_and_effect_summary(
        &source.path,
        source.source.as_str(),
        analysis.croquis,
        effect_summary,
    );
    source_maps.insert(file_id, SfcSourceMap::from_descriptor(&descriptor));
    Ok(())
}

impl SfcSourceMap {
    fn from_descriptor(descriptor: &vize_atelier_sfc::SfcDescriptor<'_>) -> Self {
        Self {
            source_length: descriptor.source.len() as u32,
            script: descriptor.script.as_ref().map(|block| SourceBlock {
                authored_start: block.loc.start as u32,
                length: block.content.len() as u32,
            }),
            setup: descriptor.script_setup.as_ref().map(|block| SourceBlock {
                authored_start: block.loc.start as u32,
                length: block.content.len() as u32,
            }),
            template: descriptor.template.as_ref().map(|block| SourceBlock {
                authored_start: block.loc.start as u32,
                length: block.content.len() as u32,
            }),
        }
    }

    fn map(self, kind: &CrossFileDiagnosticKind, offset: u32) -> u32 {
        let mapped = if uses_template_coordinates(kind) {
            self.template.map_or(offset, |block| {
                block.authored_start + offset.min(block.length)
            })
        } else {
            self.map_script(offset)
        };
        mapped.min(self.source_length)
    }

    fn map_script(self, offset: u32) -> u32 {
        match (self.script, self.setup) {
            (Some(script), Some(setup)) => {
                let setup_virtual_start = script.length.saturating_add(1);
                if offset >= setup_virtual_start {
                    setup.authored_start + (offset - setup_virtual_start).min(setup.length)
                } else {
                    script.authored_start + offset.min(script.length)
                }
            }
            (Some(script), None) => script.authored_start + offset.min(script.length),
            (_, Some(setup)) => setup.authored_start + offset.min(setup.length),
            (None, None) => offset,
        }
    }
}

fn normalize_sfc_diagnostics(
    diagnostics: &mut [vize_croquis_cf::CrossFileDiagnostic],
    source_maps: &FxHashMap<FileId, SfcSourceMap>,
) {
    for diagnostic in diagnostics {
        if let Some(source_map) = source_maps.get(&diagnostic.primary_file).copied() {
            diagnostic.primary_offset = source_map.map(&diagnostic.kind, diagnostic.primary_offset);
            diagnostic.primary_end_offset =
                source_map.map(&diagnostic.kind, diagnostic.primary_end_offset);
        }
        for (file_id, offset, _) in &mut diagnostic.related_files {
            if let Some(source_map) = source_maps.get(file_id).copied() {
                *offset = source_map.map(&diagnostic.kind, *offset);
            }
        }
    }
}

fn uses_template_coordinates(kind: &CrossFileDiagnosticKind) -> bool {
    matches!(
        kind,
        CrossFileDiagnosticKind::DuplicateElementId { .. }
            | CrossFileDiagnosticKind::NonUniqueIdInLoop { .. }
            | CrossFileDiagnosticKind::BrowserApiInSsr { .. }
    )
}

fn doctor_options() -> CrossFileOptions {
    CrossFileOptions::minimal()
        .with_provide_inject(true)
        .with_unique_ids(true)
        .with_server_client_boundary(true)
        .with_reactivity_tracking(true)
        .with_race_conditions(true)
        .with_setup_context(true)
        .with_circular_dependencies(true)
}
