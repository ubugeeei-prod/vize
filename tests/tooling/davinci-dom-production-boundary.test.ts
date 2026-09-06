import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  metadata,
  readRepoFile,
  repoRoot,
  workspacePackage,
} from "./support/davinci-stage-dependencies.ts";

const domStageDeps = new Set(["vize_davinci", "vize_s1_to_s2", "vize_s2"]);
const compilerOptionsProjectedToS2 = [
  "mode",
  "prefix_identifiers",
  "hoist_static",
  "cache_handlers",
  "scope_id",
  "comments",
  "component_name",
  "experimental_in_tag_comments",
  "inline",
  "binding_metadata",
  "is_ts",
];
const compilerOptionsHandledAroundS2 = ["source_map"];
const compilerOptionsHeldAtDefault = [
  "ssr",
  "experimental_patterned_template",
  "custom_renderer",
  "dialect",
  "croquis",
];
const s2EmitOptionFields = [
  "mode",
  "runtime_module_name",
  "runtime_global_name",
  "prefix_identifiers",
  "hoist_static",
  "inline",
  "component_name",
  "cache_handlers",
  "hoisted_scope_id",
  "scope_id",
  "is_ts",
  "comments",
  "experimental_in_tag_comments",
  "custom_element_patterns",
  "custom_element_predicate",
  "bindings",
];
const codegenOptionsOwnedByDomCompiler = [
  "mode",
  "prefix_identifiers",
  "source_map",
  "component_name",
  "scope_id",
  "ssr",
  "is_ts",
  "inline",
  "binding_metadata",
  "cache_handlers",
];
const codegenOptionsProjectedToS2 = ["runtime_module_name", "runtime_global_name"];
const codegenOptionsHandledAroundS2 = ["filename"];
const codegenOptionsNoopForDomS2 = ["optimize_imports"];

test("DOM compiler keeps the published S2 renderer available for profiling", () => {
  const dependencies = workspacePackage(metadata, "vize_atelier_dom").dependencies;
  const productionStageDeps = dependencies
    .filter((dependency) => dependency.kind === null && domStageDeps.has(dependency.name))
    .map((dependency) => dependency.name)
    .sort();
  assert.deepEqual(productionStageDeps, ["vize_s1_to_s2"]);

  const witnessDeps = dependencies
    .filter((dependency) => dependency.kind === "dev" && domStageDeps.has(dependency.name))
    .map((dependency) => dependency.name)
    .sort();
  assert.deepEqual(witnessDeps, ["vize_davinci"]);
});

test("source-map-disabled DOM compile records the S2 profiling counter", () => {
  const tmpDir = path.join(repoRoot, "target", "vize-tests", "tmp");
  fs.mkdirSync(tmpDir, { recursive: true });

  const result = spawnSync(
    "cargo",
    [
      "test",
      "-p",
      "vize_atelier_dom",
      "--test",
      "davinci_s2_profile",
      "profile_reports_real_s2_dom_walks",
      "--",
      "--exact",
    ],
    {
      cwd: repoRoot,
      encoding: "utf8",
      env: { ...process.env, TEMP: tmpDir, TMP: tmpDir, TMPDIR: tmpDir },
      maxBuffer: 64 * 1024 * 1024,
    },
  );

  assert.equal(result.status, 0, result.stderr || result.stdout);
  assert.match(result.stdout, /profile_reports_real_s2_dom_walks \.\.\. ok/u);
});

test("DOM S2 production switch classifies every compiler option", () => {
  const fields = publicStructFieldNames(
    readRepoFile("crates", "vize_atelier_dom", "src", "options.rs"),
    "DomCompilerOptions",
  );

  assert.deepEqual(
    [...fields].sort(),
    [
      "mode",
      "prefix_identifiers",
      "hoist_static",
      "cache_handlers",
      "scope_id",
      "ssr",
      "source_map",
      "comments",
      "experimental_in_tag_comments",
      "experimental_patterned_template",
      "component_name",
      "inline",
      "custom_renderer",
      "binding_metadata",
      "is_ts",
      "dialect",
      "croquis",
    ].sort(),
  );

  assert.deepEqual(
    sortedUnique([
      ...compilerOptionsProjectedToS2,
      ...compilerOptionsHandledAroundS2,
      ...compilerOptionsHeldAtDefault,
    ]),
    [...fields].sort(),
  );
});

test("DOM S2 emit options stay scoped to the supported switch surface", () => {
  assert.deepEqual(
    [
      ...publicStructFieldNames(
        readRepoFile("crates", "vize_s1_to_s2", "src", "emit", "options.rs"),
        "DomEmitOptions",
      ),
    ].sort(),
    [...s2EmitOptionFields].sort(),
  );
});

test("DOM S2 production switch classifies every adapter codegen option", () => {
  const fields = publicStructFieldNames(
    readRepoFile("crates", "vize_relief", "src", "options.rs"),
    "CodegenOptions",
  );

  assert.deepEqual(
    sortedUnique([
      ...codegenOptionsOwnedByDomCompiler,
      ...codegenOptionsProjectedToS2,
      ...codegenOptionsHandledAroundS2,
      ...codegenOptionsNoopForDomS2,
    ]),
    [...fields].sort(),
  );
});

test("DOM SFC parser-backed sections do not force legacy codegen", () => {
  const source = readRepoFile("crates", "vize_atelier_dom", "src", "compile", "sfc.rs");

  assert.match(
    source,
    /if use_s2_emit && fast_path_supported && !codegen_opts\.source_map/u,
    "the direct SFC fast path guard must stay explicit",
  );
  assert.doesNotMatch(
    source,
    /else\s+if\s+use_s2_emit\s*&&\s*!\s*fast_path_supported/u,
    "parser-backed SFC sections must stay eligible for S2 after recovery",
  );
});

function sortedUnique(values: string[]): string[] {
  const unique = new Set(values);
  assert.equal(unique.size, values.length, "option classification has duplicates");
  return [...unique].sort();
}

function publicStructFieldNames(source: string, structName: string): string[] {
  const declaration = new RegExp(String.raw`pub struct ${structName}(?:<[^>]+>)?\s*\{`, "u").exec(
    source,
  );
  assert.ok(declaration, `missing ${structName} declaration`);

  const bodyStart = source.indexOf("{", declaration.index);
  const bodyEnd = matchingBraceIndex(source, bodyStart);
  const body = source.slice(bodyStart + 1, bodyEnd);
  return [...body.matchAll(/^\s*pub\s+([A-Za-z_][A-Za-z0-9_]*)\s*:/gmu)].map(([, field]) => field);
}

function matchingBraceIndex(source: string, start: number): number {
  let depth = 0;
  for (let index = start; index < source.length; index += 1) {
    if (source[index] === "{") {
      depth += 1;
    } else if (source[index] === "}") {
      depth -= 1;
      if (depth === 0) {
        return index;
      }
    }
  }
  assert.fail("unterminated struct body");
}
