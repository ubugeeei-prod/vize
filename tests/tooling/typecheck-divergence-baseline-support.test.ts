import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  cleanup,
  readJson,
  run,
  setup,
} from "./_helpers/typecheck-divergence-report-fixture.ts";

test("typecheck divergence report includes dot-directory support roots", () => {
  const fixture = setup();
  try {
    fs.mkdirSync(path.join(fixture.fixtureRoot, "src/.vitepress/vitepress/utils"), {
      recursive: true,
    });
    fs.writeFileSync(
      path.join(fixture.fixtureRoot, "src/.vitepress/vitepress/utils/index.ts"),
      "export const label = 'ok';\n",
    );
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);

    const config = readJson(path.join(fixture.reportDir, "fixture-vue-tsc.tsconfig.json"));
    assert.equal(config.include.includes("../src/.vitepress/**/*.d.ts"), true);
    assert.equal(config.include.includes("../src/.vitepress/**/*.ts"), true);
    assert.equal(config.include.includes("../src/.vitepress/**/*.js"), true);
    assert.equal(config.include.includes("../src/.vitepress/**/*.json"), true);
    assert.equal(config.include.includes("../src/.vitepress/**/*.vue"), true);
  } finally {
    cleanup(fixture);
  }
});
