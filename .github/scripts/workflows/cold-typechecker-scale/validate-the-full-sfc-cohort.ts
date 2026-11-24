import { pathToFileURL } from "node:url";
import { resolve as resolveWorkflowInput } from "node:path";
const { coldScaleSupportScriptFiles } = (await import(
  pathToFileURL(resolveWorkflowInput("./tools/benchmarks/scripts/cold-typechecker-scale.ts")).href
)) as typeof import("../../../../tools/benchmarks/scripts/cold-typechecker-scale.ts");
console.log(coldScaleSupportScriptFiles);
