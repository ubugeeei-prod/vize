// node why-prose.ts <repo> <changed path> : which references select PR-tier tests
import { pathToFileURL } from "node:url";
import { readFileSync } from "node:fs";
const [repo, changed] = process.argv.slice(2);
const mod = await import(pathToFileURL(`${repo}/tools/support/compat/github/plan-tooling-tests.mjs`).href);
const scopes = await import(pathToFileURL(`${repo}/tools/config/vite-plus/tooling-test-scopes.ts`).href);
const { matchesGlob } = await import("node:path");
const plan = mod.planToolingTests([changed], { tier: "pr", cwd: repo });
const counts = new Map<string, Set<string>>();
const add = (k: string, f: string) => { if (!counts.has(k)) counts.set(k, new Set()); counts.get(k)!.add(f); };
for (const file of plan.tests) {
  const scope = scopes.toolingTestScopes.find((e: any) => e.tests.some((g: string) => matchesGlob(file, g)));
  if (scope) add(`scope:${scope.input}`, file);
  const imports = mod.localImportInputs(file, repo);
  if (imports.files.includes(changed)) { add("import", file); continue; }
  for (const member of imports.files) {
    const src = member.includes("node_modules") ? "" : readFileSync(`${repo}/${member}`, "utf8");
    for (const ref of mod.memberReferences(member, src, 1)) {
      if (mod.readsProse(changed, [[ref]])) add(`${member} -> ${ref}`, file);
    }
  }
}
console.log(plan.tests.length, plan.reason);
for (const [k, v] of [...counts].sort((a, b) => b[1].size - a[1].size).slice(0, 25)) console.log(v.size, k);
