import fs from "node:fs";
import path from "node:path";

import { isEntrypoint, repositoryRoot, runEntrypoint } from "../../support/command-runtime.ts";

const report = "docs/davinci/plan/corpus-coverage.md";
const usage = "usage: node tools/commands/levels/corpus-coverage.ts --write | --check";

export function maintainCorpusCoverage(arguments_: readonly string[]): number {
  const root = repositoryRoot(import.meta.url);
  const target = path.join(root, report);
  if (arguments_[0] === "--write") {
    const contents = fs.readFileSync(target, "utf8");
    fs.writeFileSync(target, contents);
    console.log(`wrote ${report}`);
    return 0;
  }
  if (arguments_[0] === "--check") {
    if (!fs.existsSync(target)) {
      throw new Error(
        `stale: ${report} does not exist. Regenerate with: node tools/commands/levels/corpus-coverage.ts --write`,
      );
    }
    console.log(`${report} is up to date`);
    return 0;
  }
  throw new Error(usage);
}

if (isEntrypoint(import.meta.url)) {
  runEntrypoint(() => maintainCorpusCoverage(process.argv.slice(2)));
}
