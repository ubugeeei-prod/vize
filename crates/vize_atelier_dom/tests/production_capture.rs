//! The opt-in stage sidecar describes the emission that returned the module.

use vize_atelier_core::options::{
    BindingMetadata, BindingType, CodegenExperimentalOptions, CodegenMode, CodegenOptions,
    CustomElementMatcher, TemplateSyntaxMode,
};
use vize_atelier_dom::{
    DomCompilerOptions,
    compile_template_with_custom_elements_template_syntax_codegen_and_experimental_options,
    compile_template_with_custom_elements_template_syntax_codegen_and_experimental_options_with_stage_capture,
};
use vize_l0::dump::capture::{CaptureOutcome, StageCapture};
use vize_l0::level::Level;
use vize_l0::{Allocator, String, cstr};

fn compare(source: &str, options: DomCompilerOptions) -> StageCapture {
    let allocator = Allocator::new();
    let (_, plain_errors, plain) =
        compile_template_with_custom_elements_template_syntax_codegen_and_experimental_options(
            &allocator,
            source,
            options.clone(),
            TemplateSyntaxMode::Standard,
            CustomElementMatcher::default(),
            CodegenOptions::default(),
            CodegenExperimentalOptions::default(),
        );
    let mut capture = StageCapture::new("dom");
    let (_, observed_errors, observed) =
        compile_template_with_custom_elements_template_syntax_codegen_and_experimental_options_with_stage_capture(
            &allocator,
            source,
            options,
            TemplateSyntaxMode::Standard,
            CustomElementMatcher::default(),
            CodegenOptions::default(),
            CodegenExperimentalOptions::default(),
            &mut capture,
        );
    assert_eq!(plain_errors.len(), observed_errors.len());
    assert_eq!(plain.code, observed.code);
    assert_eq!(plain.preamble, observed.preamble);
    assert_eq!(plain.map, observed.map);
    if capture.outcome == CaptureOutcome::Accepted {
        let emitted = capture.pages.last();
        let rendered = cstr!("{}\n{}", observed.preamble, observed.code);
        assert_eq!(
            emitted.map(|page| (page.level, page.step, page.text.as_str())),
            Some((Level::L4, "emit", rendered.as_str()))
        );
    }
    capture
}

#[test]
fn accepted_dom_capture_is_from_the_emitting_run() {
    let capture = compare("<div>{{ msg }}</div>", DomCompilerOptions::default());
    assert_eq!(capture.outcome, CaptureOutcome::Accepted);
    assert_eq!(
        capture
            .pages
            .iter()
            .map(|page| page.level)
            .collect::<Vec<_>>(),
        [Level::L1, Level::L2, Level::L2, Level::L4],
    );
    assert_eq!(capture.pages[2].step, "facts");
    assert_eq!(
        capture.pages[2].text.as_str(),
        r#"L2Facts {
    legacy: LegacyFacts {
        filters: [],
        filter_helper_precedes_components: false,
    },
    if_facts: SideTable {
        entries: {},
    },
    for_facts: SideTable {
        entries: {},
    },
    slot_facts: SideTable {
        entries: {},
    },
    text_facts: SideTable {
        entries: {},
    },
    model_faults: SideTable {
        entries: {},
    },
    static_facts: SideTable {
        entries: {
            NodeId(0): StaticFacts {
                level: HasDynamicText,
                props_hoistable: false,
                nested_static: true,
                native_descendants: true,
                foreign: false,
            },
        },
    },
    complexity: None,
}"#
    );
    assert_ne!(capture.pages[1].text, capture.pages[2].text);
}

#[test]
fn accepted_dom_capture_records_effective_compile_options() {
    let mut bindings = BindingMetadata {
        is_script_setup: true,
        ..BindingMetadata::default()
    };
    bindings
        .bindings
        .insert(String::from("msg"), BindingType::SetupRef);
    bindings
        .props_aliases
        .insert(String::from("local"), String::from("prop"));
    let options = DomCompilerOptions {
        mode: CodegenMode::Module,
        prefix_identifiers: true,
        cache_handlers: true,
        comments: true,
        scope_id: Some(String::from("data-v-captured")),
        component_name: Some(String::from("OptionName")),
        binding_metadata: Some(bindings),
        ..DomCompilerOptions::default()
    };
    let defaults = CodegenOptions {
        runtime_module_name: String::from("@test/runtime"),
        runtime_global_name: String::from("TestRuntime"),
        ..CodegenOptions::default()
    };
    let experimental = CodegenExperimentalOptions {
        component_name: Some(String::from("EffectiveName")),
        ..CodegenExperimentalOptions::default()
    };
    let allocator = Allocator::new();
    let (_, plain_errors, plain) =
        compile_template_with_custom_elements_template_syntax_codegen_and_experimental_options(
            &allocator,
            "<div>{{ msg }}</div>",
            options.clone(),
            TemplateSyntaxMode::Standard,
            CustomElementMatcher::default(),
            defaults.clone(),
            experimental.clone(),
        );
    let mut capture = StageCapture::new("dom");
    let (_, observed_errors, observed) =
        compile_template_with_custom_elements_template_syntax_codegen_and_experimental_options_with_stage_capture(
            &allocator, "<div>{{ msg }}</div>", options, TemplateSyntaxMode::Standard,
            CustomElementMatcher::default(), defaults, experimental, &mut capture,
        );
    assert_eq!(capture.outcome, CaptureOutcome::Accepted);
    assert_eq!(plain_errors.len(), observed_errors.len());
    assert_eq!(plain.code, observed.code);
    assert_eq!(plain.preamble, observed.preamble);
    let option = |name| {
        capture
            .options
            .iter()
            .find(|option| option.name == name)
            .unwrap()
            .value
            .as_str()
    };
    assert_eq!(option("mode"), "module");
    assert_eq!(option("prefix-identifiers"), "true");
    assert_eq!(option("cache-handlers"), "true");
    assert_eq!(option("comments"), "true");
    assert_eq!(option("scope-id"), "Some(\"data-v-captured\")");
    assert_eq!(option("component-name"), "Some(\"EffectiveName\")");
    assert_eq!(option("runtime-module-name"), "@test/runtime");
    assert_eq!(option("runtime-global-name"), "TestRuntime");
    assert_eq!(option("dialect"), "3");
    assert_eq!(option("template-syntax"), "standard");
    assert_eq!(option("bindings"), "[(\"msg\", SetupRef)]");
    assert_eq!(option("props-aliases"), "[(\"local\", \"prop\")]");
    assert_eq!(option("script-setup"), "true");
}

#[test]
fn compatibility_selection_discards_native_pages() {
    let capture = compare(
        "<div>{{ msg }}</div>",
        DomCompilerOptions {
            experimental_patterned_template: true,
            ..DomCompilerOptions::default()
        },
    );
    assert_eq!(
        capture.outcome,
        CaptureOutcome::Legacy("patterned-template".into())
    );
    assert!(capture.pages.is_empty());
}

#[test]
fn parse_failure_does_not_claim_a_production_stage() {
    let capture = compare("<div", DomCompilerOptions::default());
    assert_eq!(
        capture.outcome,
        CaptureOutcome::Rejected("parse-error".into())
    );
    assert!(capture.pages.is_empty());
}
