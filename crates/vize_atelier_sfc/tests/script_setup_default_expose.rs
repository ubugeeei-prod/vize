//! Non-inline setup state is private unless defineExpose explicitly publishes it (#10891).

#![expect(
    clippy::disallowed_macros,
    reason = "format! assembles authored SFC fixtures"
)]

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
    TemplateCompileOptions, compile_sfc_for_adapter, parse_sfc,
};

fn compile(source: &str, inline: bool, ssr: bool, prod: bool) -> String {
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: "Closed.vue".into(),
            ..Default::default()
        },
    )
    .expect("parse setup fixture");
    let result = compile_sfc_for_adapter(
        &descriptor,
        SfcCompileOptions {
            script: ScriptCompileOptions {
                inline_template: inline,
                ..Default::default()
            },
            template: TemplateCompileOptions {
                ssr,
                is_prod: prod,
                ..Default::default()
            },
            ..Default::default()
        },
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        if inline {
            SfcScriptOutputMode::InlineTemplate
        } else {
            SfcScriptOutputMode::SeparateTemplate
        },
    )
    .expect("compile setup fixture");
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        result.code.as_str(),
        SourceType::default().with_module(true),
    )
    .parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "generated module must parse: {:?}\n{}",
        parsed.diagnostics,
        result.code
    );
    result.code.to_string()
}

#[test]
fn separate_setup_closes_the_public_instance_in_dev_production_and_ssr() {
    for source in [
        "<script setup>const msg = 1</script><template><div>{{ msg }}</div></template>",
        "<script setup lang=\"ts\">const msg: number = 1</script><template><div>{{ msg }}</div></template>",
        "<script setup>defineEmits(['save']); const msg = 1</script><template><button @click=\"$emit('save')\">{{ msg }}</button></template>",
    ] {
        for ssr in [false, true] {
            for prod in [false, true] {
                let code = compile(source, false, ssr, prod);
                assert!(code.contains("expose: __expose"), "{code}");
                assert_eq!(code.matches("__expose();").count(), 1, "{code}");
                assert!(
                    code.find("__expose();") < code.find("const __returned__"),
                    "{code}"
                );
            }
        }
    }
}

#[test]
fn explicit_exposure_is_not_replaced_or_called_twice() {
    for exposure in ["defineExpose({ msg })", "defineExpose()"] {
        let source = format!(
            "<script setup>const msg = 1; {exposure}</script><template><div>{{{{ msg }}}}</div></template>"
        );
        for inline in [false, true] {
            let code = compile(&source, inline, false, false);
            assert!(code.contains("expose: __expose"), "{code}");
            assert_eq!(code.matches("__expose(").count(), 1, "{code}");
            if exposure.contains("msg") {
                assert!(code.contains("__expose({ msg })"), "{code}");
            }
        }
    }
}

#[test]
fn inline_setup_remains_closed_without_an_extra_expose_call() {
    let code = compile(
        "<script setup>const msg = 1</script><template><div>{{ msg }}</div></template>",
        true,
        false,
        true,
    );
    assert!(!code.contains("__expose"), "{code}");
    assert!(code.contains("return (_ctx"), "{code}");
}

#[test]
fn default_exposure_precedes_async_setup_execution() {
    let code = compile(
        "<script setup>const msg = await Promise.resolve(1)</script><template><div>{{ msg }}</div></template>",
        false,
        false,
        false,
    );
    assert!(code.contains("async setup("), "{code}");
    assert_eq!(code.matches("__expose();").count(), 1, "{code}");
    let exposed = code.find("__expose();").expect("default exposure");
    let temporary = code.find("let __temp").expect("async temporaries");
    assert!(
        exposed < temporary,
        "exposure must be registered before async setup: {code}"
    );
}
