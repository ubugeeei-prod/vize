// node why-tooling.ts <repo> <path...> : classify why each PR-tier tooling test is selected
import { pathToFileURL } from "node:url";
const [repo, ...paths] = process.argv.slice(2);
const mod = await import(pathToFileURL(`${repo}/tools/support/compat/github/plan-tooling-tests.mjs`).href);
const product = await import(pathToFileURL(`${repo}/tools/support/compat/github/tooling-product-inputs.ts`).href);
const scopes = await import(pathToFileURL(`${repo}/tools/config/vite-plus/tooling-test-scopes.ts`).href);
const inputs = await import(pathToFileURL(`${repo}/tools/config/vite-plus/task-inputs.ts`).href);
const { matchesGlob } = await import("node:path");
const plan = mod.planToolingTests(paths, { tier: "pr", cwd: repo });
const counts = new Map<string, number>();
const examples = new Map<string, string[]>();
for (const file of plan.tests) {
  const imports = mod.localImportInputs(file, repo);
  let why = "prose/path reference";
  if (paths.includes(file)) why = "changed test";
  else if (paths.some((p) => imports.files.includes(p))) why = "import closure";
  else {
    const sel = product.toolingProductSelection(file, paths, () => ({
      complete: imports.complete,
      files: imports.files,
      identity: product.toolingSourceConeIdentity(repo, imports.files),
      references: [],
    }));
    if (sel.required) why = "product selection";
    else {
      const scope = scopes.toolingTestScopes.find((e: any) => e.tests.some((g: string) => matchesGlob(file, g)));
      const scoped = scope && (imports.complete ? inputs.cacheInputs[scope.input] : inputs.cacheInputs.tooling);
      if (scoped && sel.paths.some((p: string) => scoped.some((g: string) => matchesGlob(p, g)))) why = `scope ${scope.input}${imports.complete ? "" : " (incomplete closure)"}`;
    }
  }
  counts.set(why, (counts.get(why) ?? 0) + 1);
  if (!examples.has(why)) examples.set(why, []);
  if (examples.get(why)!.length < 4) examples.get(why)!.push(file);
}
console.log(plan.tests.length, plan.reason);
for (const [k, v] of [...counts].sort((a, b) => b[1] - a[1])) console.log(v, k, examples.get(k)!.join(" "));
