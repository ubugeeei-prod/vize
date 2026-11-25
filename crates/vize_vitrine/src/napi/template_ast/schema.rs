//! The TypeScript declarations of `parseTemplateAst`'s result.
//!
//! These objects are never built in Rust (the tree is written as JSON text);
//! they exist so the generated `index.d.ts` declares the documented shape.

use napi_derive::napi;
use serde_json::Value;

/// A point in the template. `offset` and `column` count UTF-16 code units,
/// as JavaScript strings index; `line` and `column` are 1-based. This is
/// `@vue/compiler-core`'s `Position`.
#[napi(object)]
pub struct TemplatePosition {
    pub offset: u32,
    pub line: u32,
    pub column: u32,
}

/// The half-open range `[start, end)` of a node:
/// `source.slice(loc.start.offset, loc.end.offset)` is its authored text.
#[napi(object)]
pub struct TemplateSourceLocation {
    pub start: TemplatePosition,
    pub end: TemplatePosition,
}

/// The template itself; `loc` spans the whole source.
#[napi(object)]
pub struct TemplateRootNode {
    #[napi(js_name = "type", ts_type = "'ROOT'")]
    pub kind: Value,
    #[napi(
        ts_type = "Array<TemplateElementNode | TemplateTextNode | TemplateCommentNode | TemplateInterpolationNode>"
    )]
    pub children: Value,
    pub loc: TemplateSourceLocation,
}

/// An element as authored. A self-closing element (`<div />`) is one element
/// with `isSelfClosing` and no children, as Vue's compiler parses it.
#[napi(object)]
pub struct TemplateElementNode {
    #[napi(js_name = "type", ts_type = "'ELEMENT'")]
    pub kind: Value,
    /// The tag name as authored.
    pub tag: String,
    /// How Vue's SFC compiler treats the element: a component (a tag that is
    /// not a platform element, `<component>`, a built-in or a `vue:` cast),
    /// a `<slot>` outlet, a `<template>` with a structural directive, or
    /// plain markup (including everything inside `v-pre`).
    #[napi(ts_type = "'ELEMENT' | 'COMPONENT' | 'SLOT' | 'TEMPLATE'")]
    pub tag_type: Value,
    pub is_self_closing: bool,
    #[napi(ts_type = "Array<TemplateAttributeNode | TemplateDirectiveNode>")]
    pub props: Value,
    #[napi(
        ts_type = "Array<TemplateElementNode | TemplateTextNode | TemplateCommentNode | TemplateInterpolationNode>"
    )]
    pub children: Value,
    /// From `<` to the end tag's `>`, or to the end of the last child when
    /// the end tag is omitted.
    pub loc: TemplateSourceLocation,
}

/// Text with its character references decoded (raw in `<script>` and
/// `<style>`). Whitespace is kept as authored, never condensed.
#[napi(object)]
pub struct TemplateTextNode {
    #[napi(js_name = "type", ts_type = "'TEXT'")]
    pub kind: Value,
    pub content: String,
    pub loc: TemplateSourceLocation,
}

/// A comment in place; `content` is the text between `<!--` and `-->`.
#[napi(object)]
pub struct TemplateCommentNode {
    #[napi(js_name = "type", ts_type = "'COMMENT'")]
    pub kind: Value,
    pub content: String,
    pub loc: TemplateSourceLocation,
}

/// `{{ expression }}`; `loc` includes the delimiters.
#[napi(object)]
pub struct TemplateInterpolationNode {
    #[napi(js_name = "type", ts_type = "'INTERPOLATION'")]
    pub kind: Value,
    /// The trimmed expression.
    pub content: TemplateSimpleExpressionNode,
    pub loc: TemplateSourceLocation,
}

/// An expression, directive argument or modifier as authored source.
#[napi(object)]
pub struct TemplateSimpleExpressionNode {
    #[napi(js_name = "type", ts_type = "'SIMPLE_EXPRESSION'")]
    pub kind: Value,
    pub content: String,
    /// `true` for a static argument or a modifier, `false` for JavaScript.
    pub is_static: bool,
    pub loc: TemplateSourceLocation,
}

/// A plain attribute.
#[napi(object)]
pub struct TemplateAttributeNode {
    #[napi(js_name = "type", ts_type = "'ATTRIBUTE'")]
    pub kind: Value,
    pub name: String,
    pub name_loc: TemplateSourceLocation,
    /// The decoded value; its `loc` includes the quotes. `null` when the
    /// attribute has no value.
    #[napi(ts_type = "TemplateTextNode | null")]
    pub value: Value,
    pub loc: TemplateSourceLocation,
}

/// A directive, shorthand or not: `:a` and `.a` are `bind`, `@a` is `on`
/// and `#a` is `slot`. `.a` carries the implicit `prop` modifier first.
#[napi(object)]
pub struct TemplateDirectiveNode {
    #[napi(js_name = "type", ts_type = "'DIRECTIVE'")]
    pub kind: Value,
    /// The directive name without `v-` (`for`, `if`, `model`, `bind`, ...).
    pub name: String,
    /// The attribute name as authored (`@click.stop`, `v-slot:item`).
    pub raw_name: String,
    /// The argument; a dynamic `[arg]` is not static and its `loc` includes
    /// the brackets.
    #[napi(ts_type = "TemplateSimpleExpressionNode | null")]
    pub arg: Value,
    pub modifiers: Vec<TemplateSimpleExpressionNode>,
    /// The decoded value between the quotes; `null` when there is none.
    #[napi(ts_type = "TemplateSimpleExpressionNode | null")]
    pub exp: Value,
    pub loc: TemplateSourceLocation,
}

/// A parser diagnostic. `code` is the stable vize compiler code.
#[napi(object)]
pub struct TemplateAstDiagnostic {
    pub code: String,
    pub message: String,
    pub loc: TemplateSourceLocation,
}

/// The result of `parseTemplateAst`: the tree is always returned. `errors`
/// are syntax errors; `warnings` are recovery notes on valid input.
#[napi(object)]
pub struct TemplateAstResult {
    pub ast: TemplateRootNode,
    pub errors: Vec<TemplateAstDiagnostic>,
    pub warnings: Vec<TemplateAstDiagnostic>,
}
