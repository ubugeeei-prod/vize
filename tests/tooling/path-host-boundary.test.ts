import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { rewritePathImports } from "../../tools/support/compat/levels/move-path-host.ts";
import { pathHostCallers } from "../../tools/support/compat/levels/path-host-callers.ts";
import { withoutHostRuntimeReferences } from "./support/davinci-host-imports.ts";
import { withoutPathHostReferences } from "./support/davinci-path-host-imports.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const read = (file: string) => fs.readFileSync(path.join(root, file), "utf8");
const caller = "crates/vize/src/commands/check/path_cache.rs";
const source = read(caller);
const forbidden = /\bvize_carton::|use vize_carton\b/u;
// The path host move is complete. These tests keep the host import boundary;
// the move replay that pinned live caller bytes is retired.

test("path import rewriting keeps literal and comment data", () => {
  assert.equal(
    rewritePathImports('let name = "vize_l0::path::is_git_metadata_path"; // vize_l0::path\n'),
    'let name = "vize_l0::path::is_git_metadata_path"; // vize_l0::path\n',
  );
});

test("only the complete finite host references are admitted", () => {
  for (const file of Object.keys(pathHostCallers)) {
    assert.doesNotMatch(
      withoutPathHostReferences(read(file), file),
      /\bvize_carton::path\b/u,
      file,
    );
  }
  assert.match(withoutPathHostReferences(source, "davinci/vize_l1/src/fixture.rs"), forbidden);
});

test("additional path APIs and identifier suffixes retain rejection", () => {
  for (const extra of [
    "vize_carton::path::canonicalize_non_verbatim(path);",
    "vize_carton :: path :: canonicalize_non_verbatim(path);",
    "vize_carton::path::is_git_metadata_path(path);",
    "vize_carton::path::normalize_windows_verbatim_path(path);",
    "vize_carton::path::canonicalize_non_verbatim_extra(path);",
    "vize_carton::path::canonicalize_non_verbatimα(path);",
  ]) {
    assert.match(withoutHostRuntimeReferences(source + "\n" + extra, caller), forbidden, extra);
  }
});

test("module aliases, grouped uses, wildcards, storage and literal references stay visible", () => {
  for (const extra of [
    "use vize_carton::path as host;",
    "use vize_carton::{path};",
    "use vize_carton::path::*;",
    "use vize_carton::String;",
    'let fixture = "vize_carton::path::canonicalize_non_verbatim";',
  ]) {
    assert.match(withoutHostRuntimeReferences(source + "\n" + extra, caller), forbidden, extra);
  }
});

test("Maestro path and source IO compose without admitting extra source IO", () => {
  const file = "crates/vize_maestro/src/server/state/global_tag_names.rs";
  const actual = read(file);
  assert.doesNotMatch(withoutHostRuntimeReferences(actual, file), forbidden);
  for (const extra of [
    "vize_carton::source_io::read_to_string(path);",
    "vize_carton::path::canonicalize_non_verbatim(path);",
    "vize_carton :: path :: canonicalize_non_verbatim(path);",
    "vize_carton::path::normalize_windows_verbatim_path(path);",
    "use vize_carton::{source_io};",
    "use vize_carton::source_io::*;",
  ])
    assert.match(withoutHostRuntimeReferences(actual + "\n" + extra, file), forbidden, extra);
});

test("editor admission regression hosts retain exact real canonicalizer boundaries", () => {
  for (const file of [
    "crates/vize_canon/src/corsa_bridge/vue_document/build/tests/native_union.rs",
    "crates/vize_maestro/src/server/native_requests/tests.rs",
    "crates/vize_maestro/src/server/project_contexts/operation_admission_tests.rs",
  ]) {
    const actual = read(file);
    const pathReference = /\bvize_carton::path\b/u;
    assert.doesNotMatch(withoutPathHostReferences(actual, file), pathReference, file);
    for (const extra of [
      "vize_carton::path::canonicalize_non_verbatim(path);",
      "vize_carton::path::normalize_windows_verbatim_path(path);",
      "use vize_carton::path::*;",
    ]) {
      assert.match(withoutPathHostReferences(`${actual}\n${extra}`, file), pathReference);
    }
    assert.match(withoutPathHostReferences(actual, `${file}.other`), pathReference);
  }
});
