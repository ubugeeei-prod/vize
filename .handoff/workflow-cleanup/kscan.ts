import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
const root = process.cwd();
const { automationSizeViolations } = await import(
  path.join(root, "tools/support/ci/automation-commands.ts")
);
const { parse } = createRequire(path.join(root, "tests/tooling/x.js"))("yaml");
const out: string[] = [];
for (const d of [".github/workflows", ".github/actions"])
  for (const e of fs.readdirSync(d, { recursive: true }) as string[]) {
    if (!/\.ya?ml$/.test(e)) continue;
    const f = path.join(d, e);
    out.push(...automationSizeViolations(f, parse(fs.readFileSync(f, "utf8"))));
  }
fs.writeFileSync(process.argv[2]!, out.join("\n") + "\n");
console.log(out.length);
