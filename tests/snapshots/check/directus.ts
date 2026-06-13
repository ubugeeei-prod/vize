import { describe, it, before } from "node:test";

import { directusApp } from "../../_helpers/directus-app.ts";
import { requireVizeAndCorsaBins } from "../../_helpers/apps.ts";
import { runCrashFreeVizeCheck } from "../_helpers/realworld.ts";

const app = directusApp;

describe(`${app.name} check (type checker)`, () => {
  before(requireVizeAndCorsaBins);

  it("vize check covers the upstream Directus app without crashing or hanging", () => {
    const summary = runCrashFreeVizeCheck(app, { timeoutMs: 300_000 });
    console.log(
      `fileCount=${summary.fileCount}, errorCount=${summary.errorCount}, durationMs=${summary.durationMs.toFixed(0)}`,
    );
  });
});
