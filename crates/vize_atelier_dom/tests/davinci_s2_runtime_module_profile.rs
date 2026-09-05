//! P2-11 witness: custom runtime module names stay on the profiled S2 DOM
//! production selector. Direct emitter tests cover the spelling; this keeps
//! the production routing from silently falling back to compatibility codegen.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use vize_atelier_core::options::{CodegenMode, CodegenOptions, TemplateSyntaxMode};
use vize_atelier_dom::{
    DomCompilerOptions, compile_template_with_template_syntax_and_codegen_options,
};
use vize_s0::Allocator;
use vize_s0::String;
use vize_s0::profiler::{CounterSummary, global_profiler};

#[test]
fn source_map_disabled_runtime_module_name_stays_on_s2_codegen() {
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();
    let source = r#"<button @click="go">{{ label }}</button>"#;
    let codegen = CodegenOptions {
        runtime_module_name: String::from("@scope/vue-runtime"),
        ..Default::default()
    };
    let compat_allocator = Allocator::new();
    let (_, compat_errors, compat) = compile_template_with_template_syntax_and_codegen_options(
        &compat_allocator,
        source,
        DomCompilerOptions {
            mode: CodegenMode::Module,
            source_map: true,
            ..Default::default()
        },
        TemplateSyntaxMode::Standard,
        codegen.clone(),
    );
    assert!(compat_errors.is_empty());

    let profile = ProfileScope::enable();
    let allocator = Allocator::new();
    let (_, errors, result) = compile_template_with_template_syntax_and_codegen_options(
        &allocator,
        source,
        DomCompilerOptions {
            mode: CodegenMode::Module,
            ..Default::default()
        },
        TemplateSyntaxMode::Standard,
        codegen,
    );
    let counters = profile.finish();

    assert!(errors.is_empty());
    assert_eq!(result.preamble, compat.preamble);
    assert_eq!(result.code, compat.code);
    assert_eq!(
        counter(&counters, "davinci.s2_dom.files"),
        1,
        "custom runtime-module compiles are covered by the S2 production option surface"
    );
}

fn counter(counters: &CounterSummary, name: &str) -> u64 {
    counters
        .entries
        .iter()
        .find(|entry| entry.name == name)
        .unwrap_or_else(|| panic!("missing {name} profile counter"))
        .total
}

struct ProfileScope;

impl ProfileScope {
    fn enable() -> Self {
        let profiler = global_profiler();
        profiler.clear();
        profiler.enable();
        Self
    }

    fn finish(self) -> CounterSummary {
        let profiler = global_profiler();
        profiler.disable();
        profiler.counter_summary()
    }
}

impl Drop for ProfileScope {
    fn drop(&mut self) {
        let profiler = global_profiler();
        profiler.disable();
        profiler.clear();
    }
}
