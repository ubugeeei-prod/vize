//! Complete public Vapor outputs for the next five authored history inputs.

use serde_json::{Value, json};
use vize_atelier_vapor::{
    VaporCompileResult, VaporCompilerExperimentalOptions, VaporCompilerOptions, compile_vapor,
};
use vize_carton::Allocator;

pub const CASES: &[(&str, &str, bool)] = &[
    (
        "once-directive",
        include_str!("../fixtures/fix-history-next/once-directive.input.txt"),
        false,
    ),
    (
        "insertion-placeholder",
        include_str!("../fixtures/fix-history-next/insertion-placeholder.input.txt"),
        false,
    ),
    (
        "root-document-order",
        include_str!("../fixtures/fix-history-next/root-document-order.input.txt"),
        false,
    ),
    (
        "hydration-sibling-element",
        include_str!("../fixtures/fix-history-next/hydration-sibling-element.input.txt"),
        false,
    ),
    (
        "mounted-control-slot",
        include_str!("../fixtures/fix-history-next/mounted-control-slot.input.txt"),
        true,
    ),
];

fn options(prefix_identifiers: bool) -> VaporCompilerOptions {
    VaporCompilerOptions {
        prefix_identifiers,
        ..VaporCompilerOptions::default()
    }
}

pub fn option_payload(prefix: bool) -> Value {
    let VaporCompilerOptions {
        prefix_identifiers,
        ssr,
        binding_metadata,
        inline,
        custom_renderer,
        experimental_in_tag_comments,
        experimental_patterned_template,
        davinci_retained_lane,
    } = options(prefix);
    let VaporCompilerExperimentalOptions {
        component_name,
        self_component,
        source_map,
        source_map_filename,
    } = VaporCompilerExperimentalOptions::default();
    json!({
        "entrypoint": "compile_vapor", "templateSyntax": "Standard",
        "customElements": "default", "sfcScope": null,
        "prefixIdentifiers": prefix_identifiers, "ssr": ssr,
        "bindingMetadata": binding_metadata, "inline": inline,
        "customRenderer": custom_renderer,
        "experimentalInTagComments": experimental_in_tag_comments,
        "experimentalPatternedTemplate": experimental_patterned_template,
        "retainedLane": davinci_retained_lane,
        "experimental": {
            "componentName": component_name, "selfComponent": self_component,
            "sourceMap": source_map, "sourceMapFilename": source_map_filename,
        },
    })
}

pub fn observe(source: &str, prefix: bool) -> Value {
    let allocator = Allocator::new();
    let VaporCompileResult {
        code,
        templates,
        map,
        error_messages,
    } = compile_vapor(&allocator, source, options(prefix));
    json!({
        "code": code.as_str(), "templates": templates,
        "map": map.as_ref().map(|value| value.as_str()),
        "errorMessages": error_messages,
    })
}
