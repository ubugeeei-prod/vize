//! #7881: genuine selected L2 output against the independent compatibility entry.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::expect_used,
    reason = "complete source/module/map/selected-backend regression evidence"
)]

use vize_atelier_core::options::{
    BindingMetadata, BindingType, CodegenExperimentalOptions, CodegenMode, CodegenOptions,
    CustomElementMatcher, TemplateSyntaxMode,
};
use vize_atelier_dom::{
    DomCompilerOptions,
    compile_sfc_template_with_custom_elements_template_syntax_hoisted_scope_id_sections_codegen_and_experimental_options_with_stage_capture as selected,
    compile_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_and_codegen_options as compatibility,
};
use vize_l0::{
    Allocator, FxHashMap, cstr,
    dump::capture::{CaptureOutcome, StageCapture},
};

macro_rules! source {
    ($name:literal) => {
        include_str!(concat!(
            "../../../tests/_fixtures/differential/compiler/setup-dynamic-args-7881/",
            $name,
            ".vue.txt"
        ))
    };
}

fn metadata(name: &str, kind: BindingType) -> BindingMetadata {
    let (event, attr) = if name == "Names" {
        ("_evt", "$attr")
    } else {
        ("evt", "attr")
    };
    let mut bindings = FxHashMap::default();
    bindings.insert(event.into(), kind);
    bindings.insert(attr.into(), kind);
    if name != "Child" {
        bindings.insert("ref".into(), BindingType::SetupMaybeRef);
        bindings.insert("n".into(), BindingType::SetupRef);
    }
    if name == "Maybe" {
        bindings.insert("make".into(), BindingType::SetupConst);
    }
    if name == "Slot" {
        bindings.insert("Child".into(), BindingType::SetupConst);
    }
    BindingMetadata {
        bindings,
        props_aliases: FxHashMap::default(),
        is_script_setup: true,
    }
}

#[test]
fn original_dynamic_setup_modules_require_selected_emission_and_full_compatibility_bytes() {
    for (name, original, kind) in [
        ("Child", source!("Child"), BindingType::LiteralConst),
        ("App", source!("App"), BindingType::SetupRef),
        ("Const", source!("Const"), BindingType::LiteralConst),
        ("Maybe", source!("Maybe"), BindingType::SetupMaybeRef),
        ("Mutable", source!("Mutable"), BindingType::SetupLet),
        ("Names", source!("Names"), BindingType::SetupRef),
        ("For", source!("For"), BindingType::SetupRef),
        ("Slot", source!("Slot"), BindingType::SetupRef),
        ("Static", source!("Static"), BindingType::SetupRef),
    ] {
        let source = original
            .split_once("<template>")
            .expect("original template start")
            .1
            .split_once("</template>")
            .expect("original template end")
            .0;
        for inline in [true, false] {
            for source_map in [false, true] {
                let allocator = Allocator::new();
                let options = DomCompilerOptions {
                    mode: CodegenMode::Module,
                    prefix_identifiers: true,
                    inline,
                    cache_handlers: inline,
                    source_map,
                    binding_metadata: Some(metadata(name, kind)),
                    ..Default::default()
                };
                let codegen = CodegenOptions {
                    source_map,
                    filename: cstr!("{}.vue", name),
                    ..Default::default()
                };
                let (_, errors, old) = compatibility(
                    &allocator,
                    source,
                    options.clone(),
                    TemplateSyntaxMode::Standard,
                    None,
                    CustomElementMatcher::default(),
                    codegen.clone(),
                );
                assert!(errors.is_empty(), "{name}: {errors:?}");
                let mut capture = StageCapture::new("dom");
                let (errors, current) = selected(
                    &allocator,
                    source,
                    options,
                    TemplateSyntaxMode::Standard,
                    None,
                    CustomElementMatcher::default(),
                    codegen,
                    CodegenExperimentalOptions::default(),
                    &mut capture,
                );
                assert!(errors.is_empty(), "{name}: {errors:?}");
                assert_eq!(
                    capture.outcome,
                    CaptureOutcome::Accepted,
                    "{name}: actual selected producer"
                );
                assert_eq!(
                    current.result.preamble, old.preamble,
                    "{name}: entire helper/hoist section"
                );
                assert_eq!(
                    current.result.code, old.code,
                    "{name}: entire original render module"
                );
                assert_eq!(current.result.map, old.map, "{name}: complete original map");
                assert_eq!(current.result.map.is_some(), source_map);
                let sections = current.sections.expect("actual selected section ranges");
                assert!(sections.imports_len <= current.result.preamble.len());
                assert!(
                    sections.assets_start <= sections.assets_end
                        && sections.assets_end <= current.result.code.len()
                );
                assert!(
                    sections.return_expr_start <= sections.return_expr_end
                        && sections.return_expr_end <= current.result.code.len()
                );
                assert_eq!(
                    capture.pages.last().map(|page| page.text.as_str()),
                    Some(cstr!("{}\n{}", current.result.preamble, current.result.code).as_str())
                );
            }
        }
    }
}
