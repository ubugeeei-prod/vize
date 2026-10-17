import path from "node:path";
import { fileURLToPath } from "node:url";
import { runLintDivergenceReport } from "../../support/fixtures/lint_divergence/main.ts";

export { runLintDivergenceReport };
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    runLintDivergenceReport();
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exitCode = 1;
  }
}
