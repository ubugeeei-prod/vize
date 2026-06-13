import { describe, it, before } from "node:test";

import { requireVizeBin } from "../../_helpers/apps.ts";
import { naiveUiApp } from "../../_helpers/ui-library-apps.ts";
import { runCrashFreeVizeLint } from "../_helpers/realworld.ts";

const app = naiveUiApp;

describe(`${app.name} lint (linter)`, () => {
  before(requireVizeBin);

  it("vize lint covers the upstream component library without crashing or hanging", () => {
    const summary = runCrashFreeVizeLint(app, { timeoutMs: 180_000 });
    console.log(`fileCount=${summary.fileCount}, durationMs=${summary.durationMs.toFixed(0)}`);
  });
});
