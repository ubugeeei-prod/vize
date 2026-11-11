// CI tests the PR merged into its base, so generated files that are fresh on
// the branch can still be stale (or conflicted) on the merge commit. Rebuild
// the merged tree without touching the checkout and re-render the catalog.
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { readWorkflowEntries, renderWorkflowCatalog } from "../../commands/ci/workflow-catalog.ts";

export function workflowCatalogMatchesMerge(
  root: string,
  base: string,
  head: string,
  log: (line: string) => void,
): boolean {
  const merge = spawnSync(
    "git",
    ["merge-tree", "--write-tree", "--name-only", "--no-messages", base, head],
    {
      cwd: root,
      encoding: "utf8",
    },
  );
  const [tree, ...conflicts] = merge.stdout.trim().split("\n");
  if (merge.status !== 0 && merge.status !== 1) {
    log(`git merge-tree failed: ${merge.stderr}`);
    return false;
  }
  const catalog = ".github/workflows/README.md";
  if (conflicts.some((file) => file.startsWith(".github/workflows/"))) {
    log(
      `merging ${base} conflicts in: ${conflicts.filter((file) => file.startsWith(".github/workflows/")).join(", ")}`,
    );
    log("regenerate the catalog after rebasing instead of resolving it by hand");
    return false;
  }
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-preflight-workflows-"));
  try {
    const archive = execFileSync("git", ["archive", "--format=tar", tree, ".github/workflows"], {
      cwd: root,
      maxBuffer: 256 * 1024 * 1024,
    });
    const extract = spawnSync("tar", ["-x", "-C", directory], { input: archive });
    if (extract.status !== 0) {
      log(`could not extract the merged workflows: ${extract.stderr.toString()}`);
      return false;
    }
    const workflows = path.join(directory, ".github/workflows");
    const expected = renderWorkflowCatalog(readWorkflowEntries(workflows));
    const actual = fs.readFileSync(path.join(directory, catalog), "utf8");
    if (actual === expected) {
      log(`merged tree ${tree.slice(0, 12)}: ${catalog} is fresh`);
      return true;
    }
    const expectedLines = expected.split("\n");
    const actualLines = actual.split("\n");
    log(`merged tree ${tree.slice(0, 12)}: ${catalog} is stale after merging the base`);
    for (
      let line = 0, shown = 0;
      line < Math.max(expectedLines.length, actualLines.length) && shown < 20;
      line++
    ) {
      if (expectedLines[line] === actualLines[line]) continue;
      log(`  line ${line + 1}: expected ${JSON.stringify(expectedLines[line] ?? "")}`);
      log(`  line ${line + 1}:   actual ${JSON.stringify(actualLines[line] ?? "")}`);
      shown++;
    }
    return false;
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}
