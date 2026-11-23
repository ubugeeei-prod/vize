import { pathToFileURL } from "node:url";
import { resolve as resolveWorkflowInput } from "node:path";
const { scopeBindingCases } = (await import(
  pathToFileURL(resolveWorkflowInput("./tools/benchmarks/scripts/component-scope-binding-cases.ts"))
    .href
)) as typeof import("../../../../tools/benchmarks/scripts/component-scope-binding-cases.ts");
for (const r of scopeBindingCases) console.log(r.shape, r.count, r.scriptBindings);
