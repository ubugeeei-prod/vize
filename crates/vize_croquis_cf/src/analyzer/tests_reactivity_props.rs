use super::{CrossFileAnalyzer, CrossFileOptions};
use crate::rules::CrossFileReactivityIssueKind;
use std::path::Path;
use vize_carton::{CompactString, SmallVec};
use vize_croquis::AnalyzerOptions;
use vize_croquis::analysis::{ComponentUsage, PassedProp};

fn script_analysis(script: &str, usages: &[(&str, &[(&str, &str)])]) -> vize_croquis::Croquis {
    let mut analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);

    for (component, props) in usages {
        analyzer
            .croquis_mut()
            .used_components
            .insert(CompactString::new(*component));
        analyzer
            .croquis_mut()
            .component_usages
            .push(component_usage(component, props));
    }

    analyzer.finish()
}

fn component_usage(component: &str, props: &[(&str, &str)]) -> ComponentUsage {
    ComponentUsage {
        name: CompactString::new(component),
        start: 0,
        end: component.len() as u32,
        props: props
            .iter()
            .enumerate()
            .map(|(index, (name, value))| PassedProp {
                name: CompactString::new(*name),
                value: Some(CompactString::new(*value)),
                start: index as u32,
                end: index as u32 + name.len() as u32,
                is_dynamic: true,
            })
            .collect(),
        events: SmallVec::new(),
        slots: SmallVec::new(),
        has_spread_attrs: false,
        scope_id: vize_croquis::ScopeId::ROOT,
        vif_guard: None,
    }
}

fn analyzer_with_parent_child(
    parent_script: &str,
    child_script: &str,
    usages: &[(&str, &[(&str, &str)])],
) -> (CrossFileAnalyzer, crate::FileId, crate::FileId) {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_reactivity_tracking(true));
    let parent_id = analyzer.add_file_with_analysis(
        Path::new("Parent.vue"),
        "",
        script_analysis(parent_script, usages),
    );
    let child_id = analyzer.add_file_with_analysis(
        Path::new("Child.vue"),
        "",
        script_analysis(child_script, &[]),
    );
    analyzer.rebuild_import_edges();
    analyzer.rebuild_component_edges();

    (analyzer, parent_id, child_id)
}

mod direct;
mod shared;
