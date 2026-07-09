use super::*;
use vize_carton::{CompactString, smallvec};
use vize_croquis::analysis::{ComponentUsage, EventListener, PassedProp};
use vize_croquis::macros::{EmitDefinition, PropDefinition};
use vize_croquis::{Croquis, ScopeId};

fn prop(name: &str, start: u32, end: u32, dynamic: bool) -> PassedProp {
    PassedProp {
        name: CompactString::new(name),
        value: None,
        start,
        end,
        is_dynamic: dynamic,
    }
}

fn event(name: &str, start: u32, end: u32) -> EventListener {
    EventListener {
        name: CompactString::new(name),
        handler: None,
        modifiers: smallvec![],
        start,
        end,
    }
}

fn usage(
    name: &str,
    start: u32,
    end: u32,
    props: Vec<PassedProp>,
    events: Vec<EventListener>,
    has_spread_attrs: bool,
) -> ComponentUsage {
    ComponentUsage {
        name: CompactString::new(name),
        start,
        end,
        props: props.into_iter().collect(),
        events: events.into_iter().collect(),
        slots: smallvec![],
        has_spread_attrs,
        scope_id: ScopeId::ROOT,
        vif_guard: None,
    }
}

fn declare_prop(analysis: &mut Croquis, name: &str) {
    analysis.macros.add_prop(PropDefinition {
        name: CompactString::new(name),
        prop_type: None,
        required: false,
        default_value: None,
    });
}

fn declare_event(analysis: &mut Croquis, name: &str) {
    analysis.macros.add_emit(EmitDefinition {
        name: CompactString::new(name),
        payload_type: None,
    });
}

fn sorted_diagnostics(result: &crate::analyzer::CrossFileResult) -> Vec<String> {
    let mut diagnostics = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let mut related = diagnostic.related_files.clone();
            related.sort_by_key(|related| (related.0.as_u32(), related.1, related.2.clone()));
            format!(
                "{:?}@{}:{}:{} related={:?}",
                diagnostic.kind,
                diagnostic.primary_file.as_u32(),
                diagnostic.primary_offset,
                diagnostic.primary_end_offset,
                related
            )
        })
        .collect::<Vec<_>>();
    diagnostics.sort();
    diagnostics
}

#[test]
fn test_snapshot_fallthrough_usage_facts() {
    let mut analyzer =
        CrossFileAnalyzer::new(CrossFileOptions::default().with_fallthrough_attrs(true));

    let mut panel = Croquis::new();
    panel.template_info.root_element_count = 2;
    panel.template_info.content_end = 120;
    declare_prop(&mut panel, "kind");
    declare_event(&mut panel, "close");

    let mut forwarder = Croquis::new();
    forwarder.template_info.root_element_count = 1;
    forwarder.template_info.uses_attrs = true;
    forwarder.template_info.binds_attrs_explicitly = true;
    forwarder.template_info.content_end = 90;

    let mut app = Croquis::new();
    app.used_components.insert(CompactString::new("Panel"));
    app.used_components.insert(CompactString::new("Forwarder"));
    app.component_usages.push(usage(
        "Panel",
        10,
        90,
        vec![
            prop("kind", 18, 29, false),
            prop("trackingId", 34, 58, true),
            prop("class", 59, 72, false),
        ],
        vec![event("close", 73, 88)],
        true,
    ));
    app.component_usages.push(usage(
        "Forwarder",
        100,
        150,
        vec![prop("data-testid", 110, 135, false)],
        vec![event("focus", 136, 148)],
        false,
    ));

    let mut shell = Croquis::new();
    shell.used_components.insert(CompactString::new("Panel"));
    shell.component_usages.push(usage(
        "Panel",
        20,
        60,
        vec![prop("telemetryKey", 28, 48, false)],
        vec![],
        false,
    ));

    analyzer.add_file_with_analysis(Path::new("Panel.vue"), "", panel);
    analyzer.add_file_with_analysis(Path::new("Forwarder.vue"), "", forwarder);
    analyzer.add_file_with_analysis(Path::new("App.vue"), "", app);
    analyzer.add_file_with_analysis(Path::new("Shell.vue"), "", shell);
    analyzer.rebuild_component_edges();

    let result = analyzer.analyze();

    assert_eq!(result.fallthrough_usage_facts.len(), 3);
    assert_eq!(result.fallthrough_component_facts.len(), 4);
    assert_eq!(
        result
            .fallthrough_summary
            .expect("fallthrough summary should be populated")
            .risky_unconsumed_fallthrough_attr_count,
        2
    );

    let mut output = String::new();
    output.push_str("=== Fallthrough Usage Facts ===\n\n");
    output.push_str("== Summary ==\n");
    append!(
        output,
        "{}\n",
        serde_json::to_string_pretty(&result.fallthrough_summary).unwrap()
    );
    output.push_str("\n== Usage Facts ==\n");
    append!(
        output,
        "{}\n",
        serde_json::to_string_pretty(&result.fallthrough_usage_facts).unwrap()
    );
    output.push_str("\n== Component Facts ==\n");
    append!(
        output,
        "{}\n",
        serde_json::to_string_pretty(&result.fallthrough_component_facts).unwrap()
    );
    output.push_str("\n== Diagnostics ==\n");
    append!(
        output,
        "{}\n",
        serde_json::to_string_pretty(&sorted_diagnostics(&result)).unwrap()
    );

    assert_snapshot!(output);
}
