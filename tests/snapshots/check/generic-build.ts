import assert from "node:assert/strict";
import { before, describe, it } from "node:test";

import { requireVizeAndCorsaBins } from "../../_helpers/apps.ts";
import { genericBuildApp } from "../../_helpers/fixture-apps.ts";
import { runCrashFreeVizeCheck } from "../_helpers/realworld.ts";

const app = genericBuildApp;

describe(`${app.name} check (type checker)`, () => {
  before(requireVizeAndCorsaBins);

  it("vize check covers the generic build fixture cleanly", () => {
    const summary = runCrashFreeVizeCheck(app, { timeoutMs: 120_000 });
    console.log(
      `fileCount=${summary.fileCount}, errorCount=${summary.errorCount}, durationMs=${summary.durationMs.toFixed(0)}`,
    );
    assert.equal(summary.errorCount, 0);
  });
});
