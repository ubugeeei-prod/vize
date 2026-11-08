//! A `<script setup lang="ts">` compiled to JavaScript erases TypeScript per
//! authored piece, so the compiler itself decides which imports survive. It
//! elides exactly as oxc's TypeScript transform with
//! `only_remove_type_imports: false` (and compiler-sfc with esbuild's
//! defaults): an import the finished module never uses as a value goes,
//! `import type` and inline `type` specifiers go, side-effect imports stay,
//! and tsconfig's `verbatimModuleSyntax` / `importsNotUsedAsValues` are not
//! read. Every case runs in every script output shape, because each shape
//! references its imports from different generated code.

use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode,
    TemplateCompileOptions, compile_script::typescript::is_plain_javascript,
    compile_sfc_for_adapter, parse_sfc,
};
use vize_carton::{String, ToCompactString, cstr};

#[derive(Clone, Copy, Debug)]
enum Shape {
    /// Production: the render function is inlined into `setup`.
    Inline,
    /// Development: `setup` returns the bindings a separate render reads.
    Separate,
    Vapor,
    Ssr,
}

const SHAPES: [Shape; 4] = [Shape::Inline, Shape::Separate, Shape::Vapor, Shape::Ssr];

fn compile(source: &str, shape: Shape) -> String {
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: "Elision.vue".into(),
            ..Default::default()
        },
    )
    .expect("parse");
    let prod = matches!(shape, Shape::Inline);
    let result = compile_sfc_for_adapter(
        &descriptor,
        SfcCompileOptions {
            vapor: matches!(shape, Shape::Vapor),
            script: ScriptCompileOptions {
                id: Some("Elision.vue".into()),
                ..Default::default()
            },
            template: TemplateCompileOptions {
                ssr: matches!(shape, Shape::Ssr),
                is_prod: prod,
                ..Default::default()
            },
            ..Default::default()
        },
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        CodegenOptions::default(),
        if prod {
            SfcScriptOutputMode::InlineTemplate
        } else {
            SfcScriptOutputMode::SeparateTemplate
        },
    )
    .expect("compile");
    assert_eq!(result.errors.len(), 0, "{shape:?}\n{}", result.code);
    assert!(
        is_plain_javascript(&result.code),
        "{shape:?}\n{}",
        result.code
    );
    result.code
}

/// The module's user imports, in order (the runtime's own imports excluded).
fn user_imports(code: &str) -> Vec<String> {
    code.lines()
        .filter(|line| line.starts_with("import "))
        .filter(|line| {
            ["vue", "@vue/server-renderer"].iter().all(|runtime| {
                !line.contains(cstr!("from '{runtime}'").as_str())
                    && !line.contains(cstr!("from \"{runtime}\"").as_str())
            })
        })
        .map(ToCompactString::to_compact_string)
        .collect()
}

fn assert_imports(source: &str, expected: &[&str]) {
    for shape in SHAPES {
        let code = compile(source, shape);
        assert_eq!(user_imports(&code), expected, "{shape:?}\n{code}");
    }
}

fn sfc(script: &str, template: &str) -> String {
    cstr!(
        "<script setup lang=\"ts\">\n{script}\n</script>\n\n<template>\n  {template}\n</template>\n"
    )
}

#[test]
fn compiler_helper_names_in_non_reference_positions_do_not_keep_imports() {
    const PLAIN: &str = "import { toDisplayString as _toDisplayString, openBlock as _openBlock, createElementBlock as _createElementBlock } from \"vue\"";
    const UNREF: &str = "import { toDisplayString as _toDisplayString, openBlock as _openBlock, createElementBlock as _createElementBlock, unref as _unref } from \"vue\"";
    for (script, imports) in [
        (
            "const label = '_createCommentVNode'; const show = true",
            PLAIN,
        ),
        (
            "const label = { '_createCommentVNode': true }; const show = true",
            PLAIN,
        ),
        (
            "const label = { _createCommentVNode: true }; const show = true",
            PLAIN,
        ),
        (
            "const label = /_createCommentVNode/; const show = true",
            UNREF,
        ),
        (
            "const label = `_createCommentVNode`; const show = true",
            PLAIN,
        ),
        (
            "const label = true; const show = true; // _createCommentVNode",
            PLAIN,
        ),
    ] {
        let source = sfc(
            script,
            "<div v-if=\"show\">{{ label }}</div><div v-else>off</div>",
        );
        let code = compile(&source, Shape::Inline);
        assert_eq!(code.lines().next(), Some(imports), "{script}\n{code}",);
    }
    let code = compile(
        &sfc("const show = true", "<div v-if=\"show\">on</div>"),
        Shape::Inline,
    );
    assert_eq!(
        code.lines().next(),
        Some(
            "import { openBlock as _openBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode } from \"vue\""
        ),
    );
}

fn unresolved_generated_helpers(code: &str) -> Vec<String> {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, code, oxc_span::SourceType::mjs()).parse();
    assert_eq!(parsed.diagnostics.len(), 0);
    let semantic = oxc_semantic::SemanticBuilder::new().build(&parsed.program);
    assert_eq!(semantic.diagnostics.len(), 0);
    let scoping = semantic.semantic.into_scoping();
    let mut names: Vec<String> = scoping
        .root_unresolved_references()
        .keys()
        .filter(|name| name.as_str().starts_with('_'))
        .map(|name| name.as_str().to_compact_string())
        .collect();
    names.sort();
    names
}

#[test]
fn compiler_glue_keeps_the_runtime_helpers_it_actually_calls() {
    for source in [
        sfc("const slots = defineSlots()", "<slot/>"),
        sfc("const count = defineModel<number>()", "<p>{{ count }}</p>"),
        sfc(
            "const value: number = await Promise.resolve(1)",
            "<p>{{ value }}</p>",
        ),
        sfc("defineOptions({ inheritAttrs: false })", "<div/>"),
        sfc(
            "const { title = 'x', ...rest } = defineProps(['title', 'other'])",
            "<p>{{ title }}{{ rest.other }}</p>",
        ),
        sfc(
            "const props = withDefaults(defineProps<{ label?: string }>(), { label: 'x' })",
            "<p>{{ props.label }}</p>",
        ),
        cstr!(
            "{}<style>.x {{ color: v-bind(color) }}</style>",
            sfc("const color: string = 'red'", "<div/>")
        ),
        cstr!(
            "{}<style module>.x {{ color: red }}</style>",
            sfc(
                "const label: string = 'x'",
                "<div :class=\"$style.x\">{{ label }}</div>"
            )
        ),
    ] {
        for shape in SHAPES {
            let code = compile(&source, shape);
            assert_eq!(
                unresolved_generated_helpers(&code),
                Vec::<String>::new(),
                "{shape:?}\n{source}\n{code}"
            );
        }
    }
    let code = compile(
        &sfc("const slots = defineSlots()", "<slot/>"),
        Shape::Inline,
    );
    let changed = code.replace("useSlots as _useSlots", "useSlots as _missingUseSlots");
    assert_ne!(changed, code);
    assert_eq!(unresolved_generated_helpers(&changed), vec!["_useSlots"]);
}

#[test]
fn an_import_used_only_in_the_template_is_kept() {
    assert_imports(
        &sfc(
            "import { formatLabel } from './format'\nimport Card from './Card.vue'\nconst label: string = 'one'",
            "<Card>{{ formatLabel(label) }}</Card>",
        ),
        &[
            "import { formatLabel } from './format'",
            "import Card from './Card.vue'",
        ],
    );
}

#[test]
fn an_import_used_only_as_a_type_is_elided() {
    assert_imports(
        &sfc(
            "import { Label } from './label'\nconst label: Label = 'one'",
            "<p>{{ label }}</p>",
        ),
        &[],
    );
}

/// Pinned on purpose: a type imported WITHOUT `type` and used only as a
/// resolved prop value type is elided, as oxc's whole-module transform did.
/// Changing this needs a deliberate decision.
#[test]
fn an_untyped_import_used_only_by_define_props_is_elided() {
    assert_imports(
        &sfc(
            "import { Props } from './types'\nconst props = defineProps<{ value?: Props }>()",
            "<p>{{ props.value }}</p>",
        ),
        &[],
    );
}

#[test]
fn an_import_used_as_a_type_and_in_the_template_is_kept() {
    assert_imports(
        &sfc(
            "import { Status } from './status'\nconst current: Status = 'open'",
            "<p :class=\"Status.className\">{{ current }}</p>",
        ),
        &["import { Status } from './status'"],
    );
}

#[test]
fn a_reexported_import_is_kept() {
    let source = "<script lang=\"ts\">\nimport { formatLabel } from './format'\nimport { Label } from './label'\nexport { formatLabel }\nexport type { Label }\n</script>\n\n<script setup lang=\"ts\">\nconst label: Label = 'one'\n</script>\n\n<template>\n  <p>{{ label }}</p>\n</template>\n";
    assert_imports(source, &["import { formatLabel } from './format'"]);
}

#[test]
fn side_effect_imports_are_kept() {
    assert_imports(
        &sfc(
            "import './polyfill'\nimport 'virtual:theme.css'\nconst label: string = 'one'",
            "<p>{{ label }}</p>",
        ),
        &["import './polyfill'", "import 'virtual:theme.css'"],
    );
}

#[test]
fn import_type_declarations_are_removed() {
    assert_imports(
        &sfc(
            "import type { Label } from './label'\nimport { shout } from './shout'\nconst label: Label = shout('one')",
            "<p>{{ label }}</p>",
        ),
        &["import { shout } from './shout'"],
    );
}

#[test]
fn inline_type_specifiers_are_removed() {
    assert_imports(
        &sfc(
            "import { shout, type Shout } from './shout'\nconst loud: Shout = shout",
            "<p>{{ loud('one') }}</p>",
        ),
        &["import { shout } from './shout'"],
    );
}

#[test]
fn an_unused_value_import_is_elided_but_a_used_one_stays() {
    assert_imports(
        &sfc(
            "import { used, unused } from './values'\nconst label: string = used()",
            "<p>{{ label }}</p>",
        ),
        &["import { used } from \"./values\";"],
    );
}

/// Imports read only by authored code the compiler moves into component
/// options or `setup` glue are values in the finished module.
#[test]
fn imports_read_by_macro_arguments_are_kept() {
    for (script, kept) in [
        (
            "import { sizes } from './sizes'\ndefineProps({ size: { type: String, validator: (v: string) => sizes.includes(v) } })",
            "import { sizes } from './sizes'",
        ),
        (
            "import { fallback } from './fallback'\nconst props = withDefaults(defineProps<{ label?: string }>(), { label: () => fallback as string })",
            "import { fallback } from './fallback'",
        ),
        (
            "import { fallback } from './fallback'\nconst { label = fallback } = defineProps<{ label?: string }>()",
            "import { fallback } from './fallback'",
        ),
        (
            "import { onChange } from './events'\ndefineEmits({ change: (id: number) => onChange(id) })",
            "import { onChange } from './events'",
        ),
        (
            "import { name } from './meta'\ndefineOptions({ name: name as string })",
            "import { name } from './meta'",
        ),
        (
            "import { api } from './api'\ndefineExpose({ api: api as unknown })",
            "import { api } from './api'",
        ),
        (
            "import { initial } from './initial'\nconst model = defineModel<string>({ default: initial as string })",
            "import { initial } from './initial'",
        ),
    ] {
        assert_imports(&sfc(script, "<p>x</p>"), &[kept]);
    }
}

/// A name the template binds itself is not the import, so the import goes.
#[test]
fn a_template_scope_name_does_not_keep_a_type_import() {
    assert_imports(
        &sfc(
            "import { Row } from './row'\nconst rows: Row[] = []",
            "<p v-for=\"Row in rows\">{{ Row }}</p>",
        ),
        &[],
    );
}

/// A setup body may `await` at its top level, where `await {` takes an
/// object operand: the piece is erased under the module goal.
#[test]
fn a_setup_body_awaiting_an_object_literal_is_erased() {
    assert_imports(
        &sfc(
            "import { shout } from './shout'\nconst value: number = 1\nawait { value }\nshout(String(value))",
            "<p>x</p>",
        ),
        &["import { shout } from './shout'"],
    );
}

/// JavaScript sources are emitted as written: nothing is elided.
#[test]
fn javascript_sources_keep_every_import() {
    let source = "<script setup>\nimport { unused } from './values'\nconst label = 'one'\n</script>\n\n<template>\n  <p>{{ label }}</p>\n</template>\n";
    assert_imports(source, &["import { unused } from './values'"]);
}
