import assert from "node:assert/strict";
import { test } from "node:test";

import {
  dependency,
  metadata,
  readRepoFile,
  workspacePackage,
} from "./support/davinci-stage-dependencies.ts";

test("Davinci SSR compile path imports the S4 string-plan bridge", () => {
  for (const [dependencyName, rename] of [
    ["vize_s1", null],
    ["vize_s1_to_s2", null],
    ["vize_s2", null],
    ["vize_s2_to_s3", null],
    ["vize_impeto", "vize_s3"],
  ] as const) {
    const dep = dependency(metadata, "vize_atelier_ssr", dependencyName, null);
    assert.equal(dep.rename, rename);
    assert.equal(dep.req, `=${workspacePackage(metadata, dependencyName).version}`);
  }

  const compile = readRepoFile("crates", "vize_atelier_ssr", "src", "compile.rs");
  const bridge = readRepoFile("crates", "vize_atelier_ssr", "src", "s4.rs");
  const plan = readRepoFile("crates", "vize_atelier_ssr", "src", "s4", "string_plan.rs");
  assert.match(compile, /lower_source_for_ssr/u);
  assert.match(bridge, /vize_s1::parse_with_options/u);
  assert.match(bridge, /vize_s1_to_s2::lower/u);
  assert.match(bridge, /vize_s2_to_s3::lower/u);
  assert.match(bridge, /vize_s3::verify::verify/u);
  assert.match(plan, /PartitionFacts/u);
});
