// The scripted level-layout moves rewrite paths only as the module tree changes.
import assert from "node:assert/strict";
import { test } from "node:test";

import {
  modulePathOf,
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
