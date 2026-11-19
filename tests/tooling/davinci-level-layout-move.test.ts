// The scripted level-layout moves rewrite paths only as the module tree changes.
import assert from "node:assert/strict";
import { test } from "node:test";

import {
  declaredNames,
  identifierRenames,
  modulePathOf,
  namesFrom,
  relocate,
  renameIdentifiers,
  rewritePaths,
  rewriteRelativeFiles,
  rewriteRust,
  type Step,
  steps,
} from "../../tools/commands/davinci/level-layout-move.ts";

const step: Step = steps["l1-markup"]!;

test("module paths of files and their relocation", () => {
  assert.deepEqual(modulePathOf("davinci/vize_l1", "davinci/vize_l1/src/lib.rs"), []);
  assert.deepEqual(modulePathOf("davinci/vize_l1", "davinci/vize_l1/src/markup/lex.rs"), [
    "markup",
    "lex",
  ]);
  assert.deepEqual(relocate(step.moves, ["markup", "lex", "native"]), [
    "markup",
    "htmlish",
    "lex",
    "native",
  ]);
  assert.deepEqual(relocate(step.moves, ["pug", "tree"]), ["markup", "pug", "tree"]);
  assert.deepEqual(relocate(step.moves, ["markup", "token"]), ["markup", "token"]);
});

test("crate, relative and grouped paths follow the moved modules", () => {
  const lex = ["markup", "lex"];
  // A moved file reaching a module that stays: the spelling becomes absolute.
  assert.equal(
    rewriteRust("use super::token::Sink;", step, lex),
    "use crate::markup::token::Sink;",
  );
  assert.equal(
    rewriteRust("use super::{Document, Lexer};", step, ["markup", "document"]),
    "use crate::markup::{Document, Lexer};",
  );
  // Moved together: relative spellings stay.
  assert.equal(
    rewriteRust("use super::native::Lexer;", step, ["markup", "lex", "compat"]),
    "use super::native::Lexer;",
  );
  // Inline test modules resolve against their own module.
  assert.equal(
    rewriteRust("mod tests { use super::Lexer; }", step, lex),
    "mod tests { use super::Lexer; }",
  );
  assert.equal(
    rewriteRust("use crate::pug::tree::PugNode; use crate::build::scope;", step, ["dialect"]),
    "use crate::markup::pug::tree::PugNode; use crate::markup::htmlish::build::scope;",
  );
  assert.equal(
    rewriteRust("use crate::markup::{lex::Lexer, token::Sink};", step, ["dialect"]),
    "use crate::markup::{htmlish::lex::Lexer, token::Sink};",
  );
  // `markup::parse` is also a re-exported function: a terminal path stays.
  assert.equal(
    rewriteRust("crate::markup::parse(a, b)", step, ["css"]),
    "crate::markup::parse(a, b)",
  );
  assert.equal(
    rewriteRust("crate::markup::parse::SurfaceError", step, ["css"]),
    "crate::markup::htmlish::parse::SurfaceError",
  );
  // The parent's bare child paths.
  assert.equal(
    rewriteRust("mod lex;\npub use lex::{Lexer};", step, ["markup"]),
    "mod lex;\npub use htmlish::lex::{Lexer};",
  );
  // Other crates.
  assert.equal(
    rewriteRust("use vize_l1::markup::{document::Doc, parse};", step, null),
    "use vize_l1::markup::{htmlish::document::Doc, parse};",
  );
  assert.equal(
    rewriteRust("vize_l1::pug::parse_pug(x)", step, null),
    "vize_l1::markup::pug::parse_pug(x)",
  );
});

test("repository paths, regex literals, test filters and relative includes", () => {
  assert.equal(
    rewritePaths("see davinci/vize_l1/src/markup/lex.rs and davinci/vize_l1/src/pug/tree.rs", step),
    "see davinci/vize_l1/src/markup/htmlish/lex.rs and davinci/vize_l1/src/markup/pug/tree.rs",
  );
  assert.equal(
    rewritePaths("/^davinci\\/vize_l1\\/src\\/markup\\/native\\/operand(?:\\.rs$|\\/)/", step),
    "/^davinci\\/vize_l1\\/src\\/markup\\/htmlish\\/native\\/operand(?:\\.rs$|\\/)/",
  );
  assert.equal(
    rewritePaths("cargo test -p vize_l1 --lib markup::native::operand::model::tests::", step),
    "cargo test -p vize_l1 --lib markup::htmlish::native::operand::model::tests::",
  );
  assert.equal(
    rewritePaths("davinci/vize_l1/src/markup/lexer.rs", step),
    "davinci/vize_l1/src/markup/lexer.rs",
  );
  assert.equal(
    rewriteRelativeFiles(
      'include_str!("../../../tests/fixtures/a.json")',
      "davinci/vize_l1/src/markup/document/tests.rs",
      "davinci/vize_l1/src/markup/htmlish/document/tests.rs",
      new Map(),
    ),
    'include_str!("../../../../tests/fixtures/a.json")',
  );
});

test("identifier renames are whole words", () => {
  assert.equal(
    renameIdentifiers("x.lint_tag(); mod native_lint_tag; NativeLintTagKind::Element", {
      lint_tag: "tag_facts",
      NativeLintTagKind: "NativeTagFactsKind",
    }),
    "x.tag_facts(); mod native_lint_tag; NativeTagFactsKind::Element",
  );
});

test("identifier renames keep literals and emitted TypeScript names", () => {
  const renames = { NativeElement: "ElementView", LINT: "TAG_FACTS" };
  const source = [
    "/// [`NativeElement`] and the `$NativeElement` helper.",
    'ts.push_str(": $NativeElement<"); f.debug_struct("NativeElement");',
    "let raw = r#\"NativeElement\"#; let c = '\"'; fn f<'a>(_: &'a NativeElement) {}",
    'let b = b"LINT"; __vizeNativeElementProp; <const LINT: bool>',
  ].join("\n");
  assert.equal(
    renameIdentifiers(source, renames),
    [
      "/// [`ElementView`] and the `$NativeElement` helper.",
      'ts.push_str(": $NativeElement<"); f.debug_struct("NativeElement");',
      "let raw = r#\"NativeElement\"#; let c = '\"'; fn f<'a>(_: &'a ElementView) {}",
      'let b = b"LINT"; __vizeNativeElementProp; <const TAG_FACTS: bool>',
    ].join("\n"),
  );
  assert.equal(
    renameIdentifiers('f.debug_struct("NativeElement"); "NativeElement {"', renames, {
      debugNames: true,
    }),
    'f.debug_struct("ElementView"); "NativeElement {"',
  );
  assert.equal(
    renameIdentifiers("LINT; NativeElement", renames, { only: new Set(["LINT"]) }),
    "TAG_FACTS; NativeElement",
  );
});

test("other packages rename only the names they use from the crate", () => {
  const step: Step = {
    crate: "l1",
    name: "vize_l1",
    moves: [],
    items: [],
    parents: {},
    identifiers: { NativeElement: "ElementView", NativeChild: "ChildView", lint_tag: "tag_facts" },
  };
  const sources: Record<string, string> = {
    "l1/src/lib.rs": "pub struct NativeElement; pub struct NativeChild;",
    "l1/tests/a.rs": "use vize_l1::NativeElement;",
    "user/src/lib.rs": "use vize_l1::{markup::NativeElement, x};\nfn f(e: &E) { e.lint_tag(); }",
    "tool/src/lib.rs": 'use vize_l1::x; struct NativeChild; const S: &str = "$NativeElement";',
    "probe/src/lib.rs": "struct NativeChild(Child);",
    "tests/_fixtures/a/b.rs": "use vize_l1::NativeElement;",
  };
  const plan = identifierRenames(
    step,
    Object.keys(sources),
    (file) => sources[file]!,
    (dir) => ["l1", "user", "tool", "probe"].includes(dir),
  );
  assert.deepEqual(
    [...plan].map(([file, { only, debugNames }]) => [file, only && [...only].sort(), debugNames]),
    [
      ["l1/src/lib.rs", undefined, true],
      ["l1/tests/a.rs", undefined, true],
      ["user/src/lib.rs", ["NativeElement", "lint_tag"], undefined],
    ],
  );
  // A package with a same-named item of its own renames the crate's only by path.
  const mixed =
    "use vize_l1::{NativeChild as Source, NativeElement};\nenum E { NativeChild }\nE::NativeChild; NativeElement;";
  const scope = identifierRenames(
    step,
    ["mixed/src/lib.rs"],
    () => mixed,
    (dir) => dir === "mixed",
  ).get("mixed/src/lib.rs")!;
  assert.equal(
    renameIdentifiers(mixed, step.identifiers!, scope),
    "use vize_l1::{ChildView as Source, ElementView};\nenum E { NativeChild }\nE::NativeChild; ElementView;",
  );
});

test("use trees and paths name what a package reaches", () => {
  assert.deepEqual(
    [
      ...namesFrom("use vize_l1::{a::{B, c as D}, E};\nlet x = ::vize_l1::f::G::new();", "vize_l1"),
    ].sort(),
    ["B", "D", "E", "G", "a", "as", "c", "f", "new", "vize_l1"],
  );
  assert.deepEqual(
    [
      ...declaredNames(
        "enum K { #[x] A, B(u8), C { d: u8 } } struct S<T: Fn() -> u8> { pub e: Vec<(T, u8)>, f: u8 } fn g() {}",
      ),
    ].sort(),
    ["A", "B", "C", "K", "S", "e", "f", "g"],
  );
});

test("the style and Vue steps move the most specific module first", () => {
  const style = steps["l1-style"]!.moves;
  assert.deepEqual(relocate(style, ["css", "lang", "nested"]), ["style", "nesting"]);
  assert.deepEqual(relocate(style, ["css", "bind"]), ["framework", "vue", "style_bind"]);
  assert.deepEqual(relocate(style, ["css", "parser"]), ["style", "css", "parser"]);
  const vue = steps["l1-vue"]!.moves;
  assert.deepEqual(relocate(vue, ["dialect", "vue", "feature", "event"]), [
    "framework",
    "vue",
    "feature",
    "event",
  ]);
  assert.deepEqual(relocate(vue, ["dialect", "vue"]), ["framework", "vue", "quirks"]);
  assert.deepEqual(relocate(vue, ["dialect", "vue2", "text"]), [
    "framework",
    "vue",
    "version",
    "v2",
    "text",
  ]);
});

test("group members that leave their group are named from the root", () => {
  const vue = steps["l1-vue"]!;
  assert.equal(
    rewriteRust("use vize_l1::markup::{directive::DirectiveName, token::Token};", vue, null),
    "use vize_l1::{markup::{token::Token}, framework::vue::directive::DirectiveName};",
  );
  assert.equal(
    rewriteRust("use super::{directive::DirectiveName, token::Token};", vue, [
      "markup",
      "grammar",
    ]),
    "use crate::{markup::{token::Token}, framework::vue::directive::DirectiveName};",
  );
  assert.equal(
    rewriteRust("use vize_l1::markup::{directive::DirectiveName};", vue, null),
    "use vize_l1::{framework::vue::directive::DirectiveName};",
  );
  assert.equal(
    rewriteRust(
      "use vize_l1::{dialect::vue3::VueDirectives, markup::{token::Token, directive::DirectivePrefix,},};",
      vue,
      null,
    ),
    "use vize_l1::{framework::vue::version::v3::VueDirectives, markup::{token::Token,}, framework::vue::directive::DirectivePrefix};",
  );
});
