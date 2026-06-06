import { describe, it, before } from "node:test";

import { requireVizeBin, vueVbenAdminApp } from "../../_helpers/apps.ts";
import { runCrashFreeVizeLint } from "../_helpers/realworld.ts";

const app = vueVbenAdminApp;

describe(`${app.name} lint (linter)`, () => {
  before(requireVizeBin);

  it("vize lint covers the real-world Vue app without crashing or hanging", () => {
    const summary = runCrashFreeVizeLint(app, { timeoutMs: 180_000 });
    console.log(`fileCount=${summary.fileCount}, durationMs=${summary.durationMs.toFixed(0)}`);
  });
});
