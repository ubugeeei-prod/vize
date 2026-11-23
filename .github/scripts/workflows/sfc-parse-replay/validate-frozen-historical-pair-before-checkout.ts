import { pathToFileURL } from "node:url";
import { resolve as resolveWorkflowInput } from "node:path";

const { validateRevisionInputs } = (await import(
  pathToFileURL(resolveWorkflowInput("./tools/benchmarks/scripts/sfc-parse-replay-contract.ts"))
    .href
)) as typeof import("../../../../tools/benchmarks/scripts/sfc-parse-replay-contract.ts");
validateRevisionInputs({
  base: process.env.REPLAY_BASE_SHA,
  head: process.env.REPLAY_HEAD_SHA,
  conflictingModes: process.env.REPLAY_CONFLICTING_MODES !== "false",
});
