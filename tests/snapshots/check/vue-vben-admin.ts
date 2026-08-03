import { describe, it, before } from "node:test";

import { requireVizeAndCorsaBins, vueVbenAdminApp } from "../../_helpers/apps.ts";
import { runBudgetedBatchVizeCheck } from "../_helpers/batch-check-performance.ts";

const app = vueVbenAdminApp;

describe(`${app.name} check (type checker)`, () => {
  before(requireVizeAndCorsaBins);

  it("vize check covers budgeted cold and warm real-world runs exactly", () => {
    const { cold, warm } = runBudgetedBatchVizeCheck(app);
    console.log(
      `fileCount=${cold.fileCount}, errorCount=${cold.errorCount}, ` +
        `coldMs=${cold.durationMs.toFixed(0)}, warmMs=${warm.durationMs.toFixed(0)}`,
    );
  });
});
