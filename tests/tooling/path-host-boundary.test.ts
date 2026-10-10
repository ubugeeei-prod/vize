import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  integrationPlan,
  rewritePathImports,
} from "../../tools/support/compat/levels/move-path-host.ts";
import { pathHostCallers } from "../../tools/support/compat/levels/path-host-callers.ts";
import { withoutHostRuntimeReferences } from "./support/davinci-host-imports.ts";
import { withoutPathHostReferences } from "./support/davinci-path-host-imports.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const read = (file: string) => fs.readFileSync(path.join(root, file), "utf8");
const caller = "crates/vize/src/commands/check/path_cache.rs";
const source = read(caller);
const forbidden = /\bvize_carton::|use vize_carton\b/u;

test("path ownership replay is integrated and keeps literal/comment data", () => {
  assert.equal(integrationPlan(read).size, 0);
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

test("changed caller shape refuses a complete integration plan before writes", () => {
  assert.throws(
    () =>
      integrationPlan((file) =>
        file === caller ? source + "\nvize_carton::path::is_git_metadata_path(path);" : read(file),
      ),
    /Changed path host references/u,
  );
  assert.throws(
    () =>
      integrationPlan((file) =>
        file === "crates/vize_carton/src/lib.rs"
          ? read(file).replace("pub mod path;\n", "")
          : read(file),
      ),
    /Missing or duplicate/u,
  );
});

const pendingRead = (file: string) => {
  const current = read(file);
  if (file === "davinci/vize_l0/src/lib.rs") return current + "pub mod path;\n";
  if (file === "crates/vize_carton/src/lib.rs") return current.replace("\npub mod path;\n", "");
  if (file === "crates/vize_carton/src/path.rs")
    return current.replace(
      /#\[expect\(\n    clippy::disallowed_types,\n    reason = [^\n]+\n\)\]\n/u,
      "",
    );
  if (pathHostCallers[file])
    return current
      .replace(
        "use {vize_carton::path::canonicalize_non_verbatim, vize_l0::cstr};",
        "use vize_l0::{cstr, path::canonicalize_non_verbatim};",
      )
      .replace(
        "use vize_carton::path::canonicalize_non_verbatim;\nuse vize_l0::{String as CompactString, cstr};",
        "use vize_l0::{String as CompactString, cstr, path::canonicalize_non_verbatim};",
      )
      .replaceAll("vize_carton::path", "vize_l0::path");
  return current;
};

test("pending integration rejects mixed owners before any writes", () => {
  assert.equal(integrationPlan(pendingRead).size, 59);
  assert.throws(
    () =>
      integrationPlan((file) =>
        file === caller
          ? pendingRead(file) + "\nvize_carton :: path :: canonicalize_non_verbatim(path);"
          : pendingRead(file),
      ),
    /Partial path caller/u,
  );
});

test("both replay states retain the complete original path laws", () => {
  const file = "crates/vize_carton/src/path.rs";
  for (const readState of [read, pendingRead])
    for (const changed of [
      readState(file).split("#[cfg(test)]\nmod tests {")[0],
      readState(file).replace('Some("D:', 'Some("changed-D:'),
    ])
      assert.throws(
        () => integrationPlan((name) => (name === file ? changed : readState(name))),
        /Missing or duplicate|Changed original path laws/u,
      );
});
