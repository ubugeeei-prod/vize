import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { test } from "node:test";
import { withoutHostRuntimeReferences } from "./support/level-host-imports.ts";
import { profileHostImports } from "./support/level-profile-host-imports.ts";

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
    assert.match(withoutHostRuntimeReferences(declaration, "levels/vize_l2/src/lib.rs"), forbidden);
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
  assert.doesNotMatch(read("levels/vize_l0/src/profiler.rs"), /ProfileExport|mod export;/u);
});

test("the retired Curator ladder profile host admits no export import", () => {
  const file = "crates/vize_curator/src/inspector/stages/profile.rs";
  for (const declaration of [
    `use vize_carton::profile_export::{
    ProfileExportBudget, ProfileExportOptions, export_report_from_snapshots,
};`,
    "use vize_carton::profile_export::{ProfileExportBudget, ProfileExportOptions, export_report};",
  ])
    for (const path of [file, "crates/vize_curator/src/other.rs"])
      assert.match(withoutHostRuntimeReferences(declaration, path), forbidden);
});
