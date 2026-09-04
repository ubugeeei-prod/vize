import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { metadata, repoRoot, workspacePackage } from "./support/davinci-stage-dependencies.ts";

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
