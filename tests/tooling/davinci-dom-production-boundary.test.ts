import assert from "node:assert/strict";
import { test } from "node:test";

import { metadata, readRepoFile, workspacePackage } from "./support/davinci-stage-dependencies.ts";

const domStageDeps = new Set(["vize_davinci", "vize_s1_to_s2", "vize_s2"]);

test("DOM compiler keeps Davinci S2 dependencies in test space only", () => {
  const dependencies = workspacePackage(metadata, "vize_atelier_dom").dependencies;
  const productionStageDeps = dependencies
    .filter((dependency) => dependency.kind === null && domStageDeps.has(dependency.name))
    .map((dependency) => dependency.name)
    .sort();
  assert.deepEqual(productionStageDeps, []);

  const witnessDeps = dependencies
    .filter((dependency) => dependency.kind === "dev" && domStageDeps.has(dependency.name))
    .map((dependency) => dependency.name)
    .sort();
  assert.deepEqual(witnessDeps, ["vize_davinci", "vize_s1_to_s2"]);
});

test("DOM compile entry remains on the shipped transform and codegen lane", () => {
  const source = readRepoFile("crates", "vize_atelier_dom", "src", "compile.rs");
  assert.match(
    source,
    /transform_with_custom_elements_and_template_syntax_quirks_and_hoisted_scope_id\(/u,
  );
  assert.match(source, /generate_with_sections\(&root, codegen_opts\)/u);
  assert.doesNotMatch(source, /\b(?:vize_s1_to_s2|vize_s2)::|use\s+vize_s(?:1_to_s2|2)\b/u);
  assert.doesNotMatch(source, /\b(?:VIZE_DAVINCI_DOM|DOM_LANE_FLAG|DAVINCI_DOM)\b/u);
});
