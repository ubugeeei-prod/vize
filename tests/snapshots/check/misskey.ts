import { describe, it, before } from "node:test";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { misskeyApp, MISSKEY_WORK_DIR, requireVizeAndCorsaBins } from "../../_helpers/apps.ts";
import { assertSnapshot } from "../../_helpers/snapshot.ts";
import { stringifyDiagnosticSnapshot } from "../../_helpers/vize-check.ts";
import { runBudgetedBatchVizeCheck } from "../_helpers/batch-check-performance.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SNAPSHOT_DIR = path.join(__dirname, "__snapshots__");
const app = misskeyApp;

describe(`${app.name} check (type checker)`, () => {
  before(() => {
    requireVizeAndCorsaBins();
    if (app.setup) app.setup();
  });

  it("vize check keeps budgeted cold and warm output exact", () => {
    const checkConfig = app.check!;
    const { cold, warm } = runBudgetedBatchVizeCheck(app);
    console.log(
      `fileCount=${cold.fileCount}, errorCount=${cold.errorCount}, ` +
        `coldMs=${cold.durationMs.toFixed(0)}, warmMs=${warm.durationMs.toFixed(0)}`,
    );

    const prettyOutput = stringifyDiagnosticSnapshot(cold.result, checkConfig.cwd).replaceAll(
      MISSKEY_WORK_DIR,
      "<project>",
    );
    assertSnapshot(SNAPSHOT_DIR, `${app.name}-check`, prettyOutput);
  });
});
