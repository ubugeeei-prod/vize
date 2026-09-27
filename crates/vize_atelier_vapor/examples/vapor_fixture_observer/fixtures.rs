//! Complete public Vapor outputs for authored compiler history inputs.

use serde_json::{Value, json};
use vize_atelier_vapor::{
    VaporCompileResult, VaporCompilerExperimentalOptions, VaporCompilerOptions, compile_vapor,
};
use vize_carton::Allocator;

pub const CASES: &[(&str, &str, bool)] = &[
    (
        "destructured-aliases",
        include_str!("../../tests/fixtures/fix-history/destructured-aliases.input.txt"),
        false,
    ),
    (
        "destructured-key",
        include_str!("../../tests/fixtures/fix-history/destructured-key.input.txt"),
        false,
    ),
    (
        "object-index",
        include_str!("../../tests/fixtures/fix-history/object-index.input.txt"),
        false,
    ),
    (
        "delegated-if",
        include_str!("../../tests/fixtures/fix-history/delegated-if.input.txt"),
        true,
    ),
    (
        "delegated-loop",
        include_str!("../../tests/fixtures/fix-history/delegated-loop.input.txt"),
        true,
    ),
    (
        "delegated-slot-fallback",
        include_str!("../../tests/fixtures/fix-history/delegated-slot-fallback.input.txt"),
        true,
    ),
    (
        "named-slot-fallback",
        include_str!("../../tests/fixtures/fix-history/named-slot-fallback.input.txt"),
        false,
    ),
    (
        "nested-slot-outlet",
        include_str!("../../tests/fixtures/fix-history/nested-slot-outlet.input.txt"),
        false,
    ),
    (
        "directive-dynamic-argument",
        include_str!("../../tests/fixtures/fix-history/directive-dynamic-argument.input.txt"),
        false,
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
