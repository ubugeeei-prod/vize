import { describe, it, before, after } from "node:test";

import { requireVizeAndCorsaBins, voicevoxApp } from "../../_helpers/apps.ts";
import { prepareCheckFixtureDependencies } from "../../_helpers/lint-fixture-dependencies.ts";
import { runCrashFreeVizeCheck } from "../_helpers/realworld.ts";

const app = voicevoxApp;
let fixture: ReturnType<typeof prepareCheckFixtureDependencies>;

describe(`${app.name} check (type checker)`, () => {
  before(() => {
    requireVizeAndCorsaBins();
    fixture = prepareCheckFixtureDependencies(app);
  });
  after(() => fixture?.verify());

  it("vize check covers the large app without crashing or hanging", () => {
    const summary = runCrashFreeVizeCheck(fixture.app, { timeoutMs: 300_000, env: fixture.env });
    console.log(
      `fileCount=${summary.fileCount}, errorCount=${summary.errorCount}, durationMs=${summary.durationMs.toFixed(0)}`,
    );
  });
});
