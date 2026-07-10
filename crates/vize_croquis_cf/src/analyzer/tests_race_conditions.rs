use super::{CrossFileAnalyzer, CrossFileOptions};
use crate::diagnostics::{CrossFileDiagnosticKind, DiagnosticSeverity};
use std::path::Path;
use vize_carton::{CompactString, SmallVec};
use vize_croquis::AnalyzerOptions;
use vize_croquis::analysis::ComponentUsage;

fn script_analysis(script: &str, usages: &[&str]) -> vize_croquis::Croquis {
    let mut analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);

    for component in usages {
        analyzer
            .croquis_mut()
            .used_components
            .insert(CompactString::new(*component));
        analyzer
            .croquis_mut()
            .component_usages
            .push(component_usage(component));
    }

    analyzer.finish()
}

fn component_usage(component: &str) -> ComponentUsage {
    ComponentUsage {
        name: CompactString::new(component),
        start: 0,
        end: component.len() as u32,
        props: SmallVec::new(),
        events: SmallVec::new(),
        slots: SmallVec::new(),
        has_spread_attrs: false,
        scope_id: vize_croquis::ScopeId::ROOT,
        vif_guard: None,
    }
}

fn analyzer_with_single(script: &str) -> CrossFileAnalyzer {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_race_conditions(true));
    analyzer.add_file_with_analysis(Path::new("Component.vue"), "", script_analysis(script, &[]));
    analyzer
}

mod advanced;
mod basic;
