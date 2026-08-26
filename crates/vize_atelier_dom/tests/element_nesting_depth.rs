//! Element nesting depth through the whole DOM pipeline (#3480).
//!
//! Vize used to refuse element nesting past 256 levels, and the constant was not
//! free to raise: transform, static hoisting, codegen, the parser's whitespace
//! pass and even the AST's own teardown each descended one machine-stack frame
//! per nesting level, so one level past the constant the failure mode was
//! `fatal runtime error: stack overflow` — `SIGABRT`, which takes the process
//! down and cannot be turned into a diagnostic. 256 was simply the depth a debug
//! build survived on Rust's default 2 MiB thread stack, which put Vize below
//! `@vue/compiler-dom`, whose own recursion reaches ~1092 levels of `<div>` on a
//! default Node stack and then raises a *catchable* `RangeError`. Those descents
//! now grow onto the heap when the stack runs low (`vize_s0::recursion`), so
//! the limit is chosen for output size instead.
//!
//! Every case here runs on a thread with a deliberately small fixed stack. That
//! is the whole point: on a stack this size the pre-fix compiler could not reach
//! even the old limit, so "it compiled" is only possible if depth costs heap.
//! It also keeps the test honest in a release build, where frames are ~8x
//! smaller and a default-sized stack would hide the bug.
//!
//! Test-only: `std::string::String` builds the sources and expected output.
#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

use vize_atelier_core::{ErrorCode, errors::CompilerError};
use vize_atelier_dom::{DomCompilerOptions, compile_template, compile_template_with_options};
use vize_s0::Allocator;

/// Mirrors `MAX_ELEMENT_NESTING_DEPTH` in `vize_armature::parser::element::nesting`.
const NESTING_LIMIT: usize = 4096;

/// Slightly above the current upstream compiler's practical default-stack
/// boundary (~1092), while keeping the strict full-output assertion fast.
const UPSTREAM_PRACTICAL_DEPTH: usize = 1100;

/// Deeper than the old 256-level limit, shallow enough to keep the full-output
/// assertion readable when it fails.
const PAST_OLD_LIMIT: usize = 512;

/// Exercises structural-node recursion beyond the previous parser limit.
const STRUCTURAL_DEPTH: usize = 1100;

/// Smaller than a debug build needed for even 128 levels before this fix
/// (measured: 1 MiB survived 128 and aborted at 256).
const SMALL_STACK: usize = 256 * 1024;

const EXPECTED_PREAMBLE: &str = "const { createElementVNode: _createElementVNode, openBlock: _openBlock, createElementBlock: _createElementBlock } = Vue\n";

const EXPECTED_STRUCTURAL_PREAMBLE: &str = "const { resolveComponent: _resolveComponent, renderSlot: _renderSlot, openBlock: _openBlock, createBlock: _createBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, createCommentVNode: _createCommentVNode, withCtx: _withCtx } = Vue\n";

const EXPECTED_V_FOR_PREAMBLE: &str = "const { resolveComponent: _resolveComponent, renderSlot: _renderSlot, openBlock: _openBlock, createBlock: _createBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, renderList: _renderList, withCtx: _withCtx } = Vue\n";

/// `(code, message, start, end, source)` where the positions are byte
/// offsets.
type Diagnostic = (ErrorCode, String, u32, u32, Option<String>);

/// Everything a compile produced, in a form that survives the worker thread.
struct Compiled {
    diagnostics: Vec<Diagnostic>,
    code: String,
    preamble: String,
    has_source_map: bool,
}

fn nested_divs(depth: usize) -> String {
    let mut source = String::with_capacity(depth * 11 + 1);
    for _ in 0..depth {
        source.push_str("<div>");
    }
    source.push('x');
    for _ in 0..depth {
        source.push_str("</div>");
    }
    source
}

/// `<Comp>` wrapping `depth` levels of `<template v-if>` around a `<slot />`.
///
/// Structural directives take a different route through the compiler than plain
/// elements — `v-if` branches, slot detection, the patterned-template pre-pass —
/// and every one of those walks is depth-proportional too.
fn nested_v_if_templates(depth: usize) -> String {
    let mut source = String::from("<Comp>");
    for _ in 0..depth {
        source.push_str("<template v-if=\"ok\">");
    }
    source.push_str("<slot />");
    for _ in 0..depth {
        source.push_str("</template>");
    }
    source.push_str("</Comp>");
    source
}

/// `<Comp>` wrapping `depth` levels of `<template v-for>` around a `<slot />`.
fn nested_v_for_templates(depth: usize) -> String {
    let mut source = String::from("<Comp>");
    for _ in 0..depth {
        source.push_str("<template v-for=\"item in items\">");
    }
    source.push_str("<slot />");
    for _ in 0..depth {
        source.push_str("</template>");
    }
    source.push_str("</Comp>");
    source
}

/// The render function Vize emits for [`nested_divs`] at `depth`.
///
/// The outermost element opens the block; every level below it is a
/// `createElementVNode`; the innermost one carries the text child. Indentation
/// is two spaces per level.
fn expected_render_code(depth: usize) -> String {
    assert!(depth >= 2, "shape below assumes at least two levels");
    let mut out = String::new();
    out.push_str("function render(_ctx, _cache, $props, $setup, $data, $options) {\n");
    out.push_str("  return (_openBlock(), _createElementBlock(\"div\", null, [\n");
    for level in 2..depth {
        indent(&mut out, level);
        out.push_str("_createElementVNode(\"div\", null, [\n");
    }
    indent(&mut out, depth);
    out.push_str("_createElementVNode(\"div\", null, \"x\")\n");
    for level in (2..depth).rev() {
        indent(&mut out, level);
        out.push_str("])\n");
    }
    out.push_str("  ]))\n}");
    out
}

fn indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str("  ");
    }
}

fn diagnostics(source_text: &str, errors: &[CompilerError]) -> Vec<Diagnostic> {
    errors
        .iter()
        .map(|error| {
            let (start, end, source) = match &error.loc {
                Some(loc) => (
                    loc.span.start,
                    loc.span.end,
                    Some(String::from(loc.span.slice(source_text))),
                ),
                None => (0, 0, None),
            };
            (
                error.code,
                String::from(error.message.as_str()),
                start,
                end,
                source,
            )
        })
        .collect()
}

/// Compile `depth`-deep nesting on a thread with [`SMALL_STACK`] bytes of stack.
///
/// A stack overflow here aborts the whole test binary rather than failing the
/// assertion, so reaching the assertions at all is part of what is under test.
fn compile_nested_on_small_stack(depth: usize) -> Compiled {
    compile_source_on_small_stack(nested_divs(depth), false)
}

fn compile_source_on_small_stack(
    source: String,
    experimental_patterned_template: bool,
) -> Compiled {
    std::thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(move || {
            let allocator = Allocator::new();
            let (root, errors, result) = if experimental_patterned_template {
                compile_template_with_options(
                    &allocator,
                    &source,
                    DomCompilerOptions {
                        experimental_patterned_template: true,
                        ..DomCompilerOptions::default()
                    },
                )
            } else {
                compile_template(&allocator, &source)
            };
            let compiled = Compiled {
                diagnostics: diagnostics(&source, &errors),
                code: String::from(result.code.as_str()),
                preamble: String::from(result.preamble.as_str()),
                has_source_map: result.map.is_some(),
            };
            // Dropping the tree is a recursive walk of its own; do it here, on
            // the small stack, so the teardown is under test too.
            drop(root);
            compiled
        })
        .expect("spawn worker thread")
        .join()
        .expect("worker thread finished")
}

#[test]
fn nesting_past_the_old_limit_compiles_on_a_small_stack() {
    let compiled = compile_nested_on_small_stack(PAST_OLD_LIMIT);

    assert_eq!(compiled.diagnostics, Vec::new());
    assert_eq!(compiled.preamble, EXPECTED_PREAMBLE);
    assert!(!compiled.has_source_map);
    assert_eq!(compiled.code, expected_render_code(PAST_OLD_LIMIT));
}

#[test]
fn nesting_at_upstream_practical_depth_compiles_on_a_small_stack() {
    let compiled = compile_nested_on_small_stack(UPSTREAM_PRACTICAL_DEPTH);

    assert_eq!(compiled.diagnostics, Vec::new());
    assert_eq!(compiled.preamble, EXPECTED_PREAMBLE);
    assert!(!compiled.has_source_map);
    assert_eq!(
        compiled.code,
        expected_render_code(UPSTREAM_PRACTICAL_DEPTH)
    );
}

/// The render function Vize emits for [`nested_v_if_templates`] at `depth`.
///
/// Each level is a `v-if` fragment wrapping the next; the innermost one renders
/// the slot. Indentation grows by two levels per nesting level: the condition
/// sits one level in from its fragment, and the branch one further.
fn expected_structural_code(depth: usize) -> String {
    const FRAGMENT_OPEN: &str = "? (_openBlock(), _createElementBlock(_Fragment, { key: 0 }, [\n";
    const FRAGMENT_CLOSE: &str = "], 64 /* STABLE_FRAGMENT */))\n";
    const ELSE_COMMENT: &str = ": _createCommentVNode(\"v-if\", true)\n";

    let mut out = String::new();
    out.push_str("function render(_ctx, _cache, $props, $setup, $data, $options) {\n");
    out.push_str("  const _component_Comp = _resolveComponent(\"Comp\")\n");
    out.push_str("  \n");
    out.push_str("  return (_openBlock(), _createBlock(_component_Comp, null, {\n");
    out.push_str("    default: _withCtx(() => [\n");
    for level in 1..=depth {
        indent(&mut out, 2 * level + 1);
        out.push_str("(ok)\n");
        indent(&mut out, 2 * level + 2);
        if level == depth {
            out.push_str("? _renderSlot(_ctx.$slots, \"default\", { key: 0 })\n");
            indent(&mut out, 2 * level + 2);
            out.push_str(ELSE_COMMENT);
        } else {
            out.push_str(FRAGMENT_OPEN);
        }
    }
    for level in (1..depth).rev() {
        indent(&mut out, 2 * level + 2);
        out.push_str(FRAGMENT_CLOSE);
        indent(&mut out, 2 * level + 2);
        out.push_str(ELSE_COMMENT);
    }
    out.push_str("    ]),\n    _: 3 /* FORWARDED */\n  }))\n}");
    out
}

#[test]
fn structural_recursion_paths_compile_on_a_small_stack() {
    let compiled = compile_source_on_small_stack(nested_v_if_templates(STRUCTURAL_DEPTH), true);

    assert_eq!(compiled.diagnostics, Vec::new());
    assert_eq!(compiled.preamble, EXPECTED_STRUCTURAL_PREAMBLE);
    assert!(!compiled.has_source_map);
    assert_eq!(compiled.code, expected_structural_code(STRUCTURAL_DEPTH));
}

/// The render function Vize emits for [`nested_v_for_templates`] at `depth`.
///
/// Each level is a `renderList` fragment whose callback opens another fragment
/// around the next level; the innermost one renders the slot.
fn expected_v_for_code(depth: usize) -> String {
    const LIST_OPEN: &str =
        "(_openBlock(true), _createElementBlock(_Fragment, null, _renderList(items, (item) => {\n";
    const BLOCK_OPEN: &str = "return (_openBlock(), _createElementBlock(_Fragment, null, [\n";

    let mut out = String::new();
    out.push_str("function render(_ctx, _cache, $props, $setup, $data, $options) {\n");
    out.push_str("  const _component_Comp = _resolveComponent(\"Comp\")\n");
    out.push_str("  \n");
    out.push_str("  return (_openBlock(), _createBlock(_component_Comp, null, {\n");
    out.push_str("    default: _withCtx(() => [\n");
    for level in 1..=depth {
        indent(&mut out, 2 * level + 1);
        out.push_str(LIST_OPEN);
        indent(&mut out, 2 * level + 2);
        out.push_str(BLOCK_OPEN);
    }
    indent(&mut out, 2 * depth + 3);
    out.push_str("_renderSlot(_ctx.$slots, \"default\")\n");
    for level in (1..=depth).rev() {
        indent(&mut out, 2 * level + 2);
        out.push_str("], 64 /* STABLE_FRAGMENT */))\n");
        indent(&mut out, 2 * level + 1);
        out.push_str("}), 256 /* UNKEYED_FRAGMENT */))\n");
    }
    out.push_str("    ]),\n    _: 3 /* FORWARDED */\n  }))\n}");
    out
}

#[test]
fn nested_for_nodes_compile_and_drop_on_a_small_stack() {
    let compiled = compile_source_on_small_stack(nested_v_for_templates(PAST_OLD_LIMIT), true);

    assert_eq!(compiled.diagnostics, Vec::new());
    assert!(!compiled.has_source_map);
    assert_eq!(compiled.preamble, EXPECTED_V_FOR_PREAMBLE);
    assert_eq!(compiled.code, expected_v_for_code(PAST_OLD_LIMIT));
}

#[test]
fn nesting_past_the_limit_is_a_diagnostic_and_not_an_abort() {
    let compiled = compile_nested_on_small_stack(NESTING_LIMIT + 1);

    // One diagnostic, pointing at the first element the parser refused to
    // descend into: the `<div>` that would have been level 4097, five bytes per
    // preceding start tag into the source.
    let refused_at = (NESTING_LIMIT * 5) as u32;
    assert_eq!(
        compiled.diagnostics,
        vec![(
            ErrorCode::ExtendPoint,
            "Element nesting is too deep.".to_owned(),
            refused_at,
            refused_at + 5,
            Some("<div>".to_owned()),
        )]
    );
    // Like upstream's `RangeError`, the limit stops emission rather than
    // producing code for a tree the parser deliberately did not build.
    assert_eq!(compiled.code, "");
    assert_eq!(compiled.preamble, "");
    assert!(!compiled.has_source_map);
}
