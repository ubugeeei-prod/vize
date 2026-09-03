import assert from "node:assert/strict";
import { test } from "node:test";

import { metadata, readRepoFile, workspacePackage } from "./support/davinci-stage-dependencies.ts";

const domStageDeps = new Set(["vize_davinci", "vize_s1_to_s2", "vize_s2"]);

test("DOM compiler depends on the published S2 renderer", () => {
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

test("DOM compile entry emits through S2 and retains only explicit compatibility paths", () => {
  const source = readRepoFile("crates", "vize_atelier_dom", "src", "compile.rs");
  const options = readRepoFile("crates", "vize_atelier_dom", "src", "compile", "stage_options.rs");
  assert.match(
    source,
    /transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id\(/u,
  );
  assert.match(source, /atelier\.dom\.template\.s2_codegen/u);
  assert.match(source, /stage_options::emit_s2\(allocator, source, s2_option_source\.dialect/u);
  assert.match(options, /vize_s1_to_s2::emit_dom_source_with_options/u);
  assert.match(options, /LegacyCaps::for_version\(dialect\)/u);
  assert.match(source, /atelier\.dom\.template\.codegen_compat/u);
  assert.doesNotMatch(source, /\b(?:VIZE_DAVINCI_DOM|DOM_LANE_FLAG|DAVINCI_DOM)\b/u);
});
