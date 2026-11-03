//! Authored template text enters render code through one TypeScript erasure.
//! The JS-module gate that lets a retained AST skip it must reject every
//! TypeScript-only construct, including declarations in a function body:
//! `@click="() => { void Child; type T = typeof Other }"` reached CLI and wasm
//! output as TypeScript wherever no later whole-module strip ran.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode, TemplateCompileOptions, compile_sfc,
    compile_sfc_for_adapter, parse_sfc,
};
use vize_carton::{String, ToCompactString, cstr};

fn compile(script: &str, handler: &str, ssr: bool) -> String {
    let source = cstr!("{script}<template><Control @click=\"{handler}\" /></template>");
    let descriptor = parse_sfc(&source, SfcParseOptions::default()).expect("parse");
    let options = SfcCompileOptions {
        template: TemplateCompileOptions {
            ssr,
            ..Default::default()
        },
        ..Default::default()
    };
    compile_sfc(&descriptor, options).expect("compile").code
}

const TS_OPTIONS: &str =
    "<script lang=\"ts\">\nexport default { data: () => ({ Child: 1, Other: 2 }) }\n</script>\n";
const JS_OPTIONS: &str = "<script>\nexport default { data: () => ({ Child: 1 }) }\n</script>\n";

#[test]
fn typescript_declarations_in_a_handler_body_are_erased_in_every_lane() {
    let handler = "() => { void Child; type T = typeof Other; }";
    assert_eq!(
        compile(TS_OPTIONS, handler, false),
        r##"const _sfc_main = { data: () => ({
  Child: 1,
  Other: 2
}) };

import { resolveComponent as _resolveComponent, openBlock as _openBlock, createBlock as _createBlock } from "vue"

function _sfc_render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Control = _resolveComponent("Control")
  
  return (_openBlock(), _createBlock(_component_Control, {
    onClick: () => {
	void $data.Child;
}
  }, null, 8 /* PROPS */, ["onClick"]))
}
_sfc_main.render = _sfc_render
export default _sfc_main"##
    );
    assert_eq!(
        compile(TS_OPTIONS, handler, true),
        r##"import { ssrRenderComponent as _ssrRenderComponent } from "@vue/server-renderer"
import { resolveComponent as _resolveComponent, mergeProps as _mergeProps } from "vue"

function ssrRender(_ctx, _push, _parent, _attrs, $props, $setup, $data, $options) {
  _push(_ssrRenderComponent(_resolveComponent("Control"), _mergeProps({ onClick: () => {
	void $data.Child;
} }, _attrs), null, _parent))
}

const _sfc_main = { data: () => ({
  Child: 1,
  Other: 2
}) };

_sfc_main.ssrRender = ssrRender
export default _sfc_main"##
    );
}

/// Negative control: a JavaScript handler body keeps its authored bytes.
#[test]
fn javascript_handler_bodies_keep_their_bytes() {
    let handler = "() => { void Child; }";
    assert_eq!(
        compile(JS_OPTIONS, handler, false),
        r##"
const _sfc_main = { data: () => ({ Child: 1 }) }

import { resolveComponent as _resolveComponent, openBlock as _openBlock, createBlock as _createBlock } from "vue"

function _sfc_render(_ctx, _cache, $props, $setup, $data, $options) {
  const _component_Control = _resolveComponent("Control")
  
  return (_openBlock(), _createBlock(_component_Control, {
    onClick: () => { void $data.Child; }
  }, null, 8 /* PROPS */, ["onClick"]))
}
_sfc_main.render = _sfc_render
export default _sfc_main"##
    );
    assert_eq!(
        compile(JS_OPTIONS, handler, true),
        r##"import { ssrRenderComponent as _ssrRenderComponent } from "@vue/server-renderer"
import { resolveComponent as _resolveComponent, mergeProps as _mergeProps } from "vue"

function ssrRender(_ctx, _push, _parent, _attrs, $props, $setup, $data, $options) {
  _push(_ssrRenderComponent(_resolveComponent("Control"), _mergeProps({ onClick: () => { void $data.Child; } }, _attrs), null, _parent))
}


const _sfc_main = { data: () => ({ Child: 1 }) }

_sfc_main.ssrRender = ssrRender
export default _sfc_main"##
    );
}

#[derive(Clone, Copy, Debug)]
enum Lane {
    DomInline,
    DomSeparate,
    Ssr,
    Vapor,
}

/// Compiler fixture SFCs whose authored scripts and templates are valid
/// compile to JavaScript in every lane without a later strip (CLI and wasm add none; napi's
/// `finalize` must not be load-bearing). `lang="tsx"` output keeps JSX.
#[test]
fn fixture_sfcs_emit_javascript_in_every_lane() {
    let root = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential"
    );
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    {
        let mut stack = vec![std::path::Path::new(root).join("compiler")];
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).expect("fixture dir").flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.to_string_lossy().ends_with(".vue.txt") {
                    files.push(path);
                }
            }
        }
    }
    files.sort();
    let mut covered = 0usize;
    let mut not_javascript: Vec<String> = Vec::new();
    for path in &files {
        let source = std::fs::read_to_string(path).expect("fixture");
        let Ok(descriptor) = parse_sfc(&source, SfcParseOptions::default()) else {
            continue;
        };
        let scripts = [descriptor.script.as_ref(), descriptor.script_setup.as_ref()];
        let jsx = scripts
            .iter()
            .flatten()
            .any(|block| matches!(block.lang.as_deref(), Some("tsx" | "jsx")));
        let scripts_parse = scripts
            .iter()
            .flatten()
            .all(|block| parses(&block.content, SourceType::tsx()));
        if !scripts_parse {
            continue;
        }
        let lanes = [Lane::DomInline, Lane::DomSeparate, Lane::Ssr, Lane::Vapor];
        // An input any lane reports as invalid has no JavaScript to emit.
        let Some(codes) = lanes
            .iter()
            .map(|&lane| compile_lane(&descriptor, lane).map(|code| (lane, code)))
            .collect::<Option<Vec<_>>>()
        else {
            continue;
        };
        for (lane, code) in codes {
            covered += 1;
            let module = SourceType::mjs().with_jsx(jsx);
            if !parses(&code, module) {
                not_javascript.push(cstr!("{} {lane:?}", path.display()));
            }
        }
    }
    assert!(covered >= 200, "{covered} lane compiles");
    assert_eq!(not_javascript, Vec::<String>::new());
}

fn compile_lane(descriptor: &vize_atelier_sfc::SfcDescriptor<'_>, lane: Lane) -> Option<String> {
    let inline = matches!(lane, Lane::DomInline);
    let result = compile_sfc_for_adapter(
        descriptor,
        SfcCompileOptions {
            vapor: matches!(lane, Lane::Vapor),
            template: TemplateCompileOptions {
                ssr: matches!(lane, Lane::Ssr),
                is_prod: inline,
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
    .ok()?;
    result
        .errors
        .is_empty()
        .then(|| result.code.to_compact_string())
}

fn parses(code: &str, source_type: SourceType) -> bool {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, code, source_type).parse();
    !parsed.panicked && parsed.diagnostics.is_empty()
}
