import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { test } from "node:test";

import {
  cleanup,
  root,
  script,
  setup,
  writeJson,
} from "./support/typecheck-dependency-prepare-fixture.ts";

test("dependency prepare requires exact SHA evidence for selected targets", () => {
  const fixture = setup();
  try {
    writeJson(fixture.registryPath, { projects: [fixture.project] });
    const malformedSha = spawnSync(
      process.execPath,
      [
        "--experimental-strip-types",
        script,
        "--registry",
        fixture.registryPath,
        "--output-dir",
        fixture.outputDir,
      ],
      { cwd: root, encoding: "utf8", env: { ...process.env, GITHUB_SHA: "main" } },
    );
    assert.equal(malformedSha.status, 1);
    assert.match(malformedSha.stderr, /GITHUB_SHA must be a full lowercase commit SHA/);
  } finally {
    cleanup(fixture);
  }
});
