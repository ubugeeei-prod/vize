import { readFileSync } from "node:fs";
import { baselineConfig } from "../../compat/fixtures/lint-divergence-baseline.mjs";
import {
  resolveScorecardRuntime,
  runScorecardBaseline,
} from "../../compat/fixtures/lint-divergence-scorecard.ts";

const input = JSON.parse(readFileSync(0, "utf8"));
const runtime = resolveScorecardRuntime();
const baseline = await runScorecardBaseline(
  runtime,
  baselineConfig,
  input.cwd,
  input.files,
  input.rules,
);
const output = input.rawOnly ? { rawResults: baseline.rawResults } : baseline;
process.stdout.write(
  JSON.stringify({ version: runtime.version, runtimeVersions: runtime.runtimeVersions, ...output }),
);
