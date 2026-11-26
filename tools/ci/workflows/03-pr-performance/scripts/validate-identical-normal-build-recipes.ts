import { pathToFileURL } from "node:url";
import { resolve as resolveWorkflowInput } from "node:path";
import { writeFileSync } from "node:fs";
const { verifyPrPerformanceBuildPair } = (await import(
  pathToFileURL(resolveWorkflowInput("./driver/tools/benchmarks/scripts/pr-performance-build.ts"))
    .href
)) as typeof import("../../../../tools/benchmarks/scripts/pr-performance-build.ts");
const pair = verifyPrPerformanceBuildPair(
  "performance-observation/build/base.json",
  "performance-observation/build/head.json",
);
writeFileSync("performance-observation/build/pair.json", JSON.stringify(pair, null, 2) + "\n");
