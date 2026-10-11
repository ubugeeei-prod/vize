import { describe, it, before, after } from "node:test";

import { requireVizeBin, voicevoxApp } from "../../_helpers/apps.ts";
import { prepareLintFixtureDependencies } from "../../_helpers/lint-fixture-dependencies.ts";
import { runCrashFreeVizeLint } from "../_helpers/realworld.ts";

const app = voicevoxApp;
let fixture: ReturnType<typeof prepareLintFixtureDependencies>;

describe(`${app.name} lint (linter)`, () => {
  before(() => {
    requireVizeBin();
    fixture = prepareLintFixtureDependencies(app);
  });
  after(() => fixture?.verify());

  it("vize lint covers the large app without crashing or hanging", () => {
    const summary = runCrashFreeVizeLint(fixture.app, { timeoutMs: 180_000, env: fixture.env });
    console.log(`fileCount=${summary.fileCount}, durationMs=${summary.durationMs.toFixed(0)}`);
  });
});
