import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { docsRoot, moreGuidesCoverage, renderedMoreGuideRoutes } from "./more-guides-coverage.ts";

const output = path.join(docsRoot, "../docs-render-evidence/more-guides");
mkdirSync(output, { recursive: true });
const source = spawnSync("git", ["rev-parse", "HEAD"], { cwd: docsRoot, encoding: "utf8" });
assert.equal(source.status, 0, source.stderr);
writeFileSync(
  path.join(output, "coverage.json"),
  JSON.stringify(
    {
      source: source.stdout.trim(),
      issue: moreGuidesCoverage.issue,
      routes: renderedMoreGuideRoutes,
      scope:
        "Revised reading flow and explicitly captured audited guides; deployed and installed acceptance remain pending.",
    },
    null,
    2,
  ) + "\n",
);

const render = spawnSync(
  process.execPath,
  [
    path.join(docsRoot, "scripts/verify-navigation-render.ts"),
    "--routes",
    renderedMoreGuideRoutes.join(","),
    "--output",
    output,
  ],
  { cwd: path.resolve(docsRoot, ".."), stdio: "inherit" },
);
assert.equal(render.status, 0, "More guides browser review failed");
