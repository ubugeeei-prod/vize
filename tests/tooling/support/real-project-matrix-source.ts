import { execFileSync } from "node:child_process";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after } from "node:test";

import {
  realProjectJobTemplate,
  realProjectRegistryPath,
  realProjectWorkflowPath,
} from "../../../tools/support/compat/github/real-project-matrix-plan.ts";
import { readRepoFile } from "./github-workflows.ts";

let fixture: ReturnType<typeof createSource> | undefined;
after(() => {
  if (fixture) rmSync(fixture.repoRoot, { recursive: true, force: true });
});

/** Real Git objects let readers prove that their authorities come from run.head_sha. */
export function matrixSourceFixture() {
  if (!fixture) {
    fixture = createSource();
  }
  return fixture;
}

function createSource() {
  const repoRoot = mkdtempSync(join(tmpdir(), "vize-matrix-source-"));
  const git = (...args: string[]) =>
    execFileSync("git", args, {
      cwd: repoRoot,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    }).trim();
  git("init", "-q");
  const workflow = readRepoFile(realProjectWorkflowPath);
  const registry = JSON.parse(readRepoFile(realProjectRegistryPath));
  for (const path of [realProjectWorkflowPath, realProjectRegistryPath]) {
    mkdirSync(dirname(join(repoRoot, path)), { recursive: true });
  }
  const commit = (source: string) => {
    writeFileSync(join(repoRoot, realProjectWorkflowPath), source);
    writeFileSync(join(repoRoot, realProjectRegistryPath), JSON.stringify(registry));
    git("add", ".");
    git(
      "-c",
      "user.name=Matrix Test",
      "-c",
      "user.email=matrix@example.invalid",
      "-c",
      "commit.gpgsign=false",
      "commit",
      "-qm",
      "fixture source",
    );
    return git("rev-parse", "HEAD");
  };
  const legacySha = commit(
    workflow.replace(realProjectJobTemplate, "real projects (${{ matrix.shard }}/22)"),
  );
  const namedSha = commit(workflow);
  const unknownSha = commit(
    workflow.replace(realProjectJobTemplate, "real projects (unrecognized)"),
  );
  // Uncommitted names and project IDs must never influence an older selected run.
  registry.projects[0].id = "uncommitted-project";
  writeFileSync(join(repoRoot, realProjectRegistryPath), JSON.stringify(registry));
  return { repoRoot, legacySha, namedSha, unknownSha };
}
