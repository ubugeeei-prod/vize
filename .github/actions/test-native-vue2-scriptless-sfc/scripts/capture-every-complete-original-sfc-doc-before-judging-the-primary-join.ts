import { writeFileSync } from "node:fs";
const e = process.env;
if (!e.STATUS_PATH) throw new Error("missing status output path");
writeFileSync(
  e.STATUS_PATH,
  JSON.stringify({
    schema: "vize.native-vue2-scriptless-sfc-validation",
    version: 1,
    sourceHead: e.VIZE_GLYPH_VUE2_SFC_SOURCE_HEAD,
    executionCommit: e.GITHUB_SHA,
    workflowRun: e.GITHUB_RUN_ID,
    rustExitStatus: Number(e.RUST_STATUS),
    oracleExitStatus: e.ORACLE_EXECUTED === "true" ? Number(e.ORACLE_STATUS) : null,
    capturePresent: e.CAPTURE_PRESENT === "true",
    oracleExecuted: e.ORACLE_EXECUTED === "true",
    runtimeObservationPresent: e.RUNTIME_PRESENT === "true",
  }) + "\n",
);
