import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { test } from "node:test";
import { withoutHostRuntimeReferences } from "./support/davinci-host-imports.ts";
import {
  profileHostImports,
  previousCuratorProfileImport,
} from "./support/davinci-profile-host-imports.ts";
import { hashes } from "../../tools/support/levels/profile-export-host-contract.ts";

// The profile-export host move is complete. These tests keep the host import
// boundary; the move replay that pinned live caller bytes is retired.
const read = (file: string) => fs.readFileSync(new URL("../../" + file, import.meta.url), "utf8");
const forbidden = /\bvize_carton::|use vize_carton\b/u;

test("actual JSON hosts retain L0 storage and admit only their exact export symbols", () => {
  for (const [file, declaration] of Object.entries(profileHostImports)) {
    const source = read(file);
    for (const path of [file, file.replaceAll("/", "\\")])
      assert.doesNotMatch(withoutHostRuntimeReferences(source, path), forbidden);
    for (const extra of [
      "use vize_carton::String;",
      "use vize_carton::profiler::Profiler;",
      "use vize_carton::profile_export::*;",
      "use vize_carton::{profile_export::export_report, String};",
      "use vize_carton::profile_export::unreviewed_function;",
    ])
      assert.match(withoutHostRuntimeReferences(source + "\n" + extra, file), forbidden);
    assert.match(
      withoutHostRuntimeReferences(declaration, "davinci/vize_l2/src/lib.rs"),
      forbidden,
    );
  }
});

test("every tracked Rust caller retires the L0 wire API and inherent export method", () => {
  const root = new URL("../../", import.meta.url);
  const files = execFileSync("git", ["ls-files", "-z", "--", "*.rs"], {
    cwd: root,
    encoding: "utf8",
  })
    .split("\0")
    .filter(Boolean);
  for (const file of files)
    assert.doesNotMatch(
      read(file),
      /\.export_report\s*\(|vize_l0\s*::\s*profiler\s*::\s*(?:ProfileExport|PROFILE_EXPORT_SCHEMA_VERSION|\{[^}]*\bProfileExport)/u,
      file,
    );
  assert.doesNotMatch(read("davinci/vize_l0/src/profiler.rs"), /ProfileExport|mod export;/u);
});

test("both exact Curator input APIs remain limited to the original host path", () => {
  const file = hashes.snapshotCaller[0];
  for (const declaration of [profileHostImports[file], previousCuratorProfileImport]) {
    assert.doesNotMatch(withoutHostRuntimeReferences(declaration, file), forbidden);
    for (const other of ["davinci/vize_l0/src/lib.rs", "crates/vize_curator/src/other.rs"])
      assert.match(withoutHostRuntimeReferences(declaration, other), forbidden);
    for (const changed of [
      declaration.replace("export_report", "unreviewed_report"),
      declaration.replace("ProfileExportOptions", "String"),
      declaration.replace("ProfileExportBudget", "*"),
    ])
      assert.match(withoutHostRuntimeReferences(changed, file), forbidden);
  }
});
