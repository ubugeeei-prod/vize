import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { test } from "node:test";

import {
  realProjectJobNames,
  realProjectJobNamesAtRevision,
  realProjectJobTemplate,
  realProjectMatrix,
  realProjectRegistryPath,
  realProjectWorkflowPath,
} from "../../tools/support/compat/github/real-project-matrix-plan.ts";
import {
  assertRequiredWorkflowJobs,
  requiredWorkflowJobNames,
} from "../../tools/support/compat/github/release-preflight-evidence.mjs";
import { readRepoFile, root } from "./support/github-workflows.ts";
import { matrixSourceFixture } from "./support/real-project-matrix-source.ts";

test("names describe every actual project in the existing balanced partitions", () => {
  const registry = JSON.parse(readRepoFile(realProjectRegistryPath));
  const matrix = realProjectMatrix(registry);
  const projects = registry.projects.map((project: { id: string }) => project.id);
  assert.equal(matrix.length, 22);
  for (const { shard, name } of matrix) {
    assert.deepEqual(
      name.split(", "),
      projects.filter((_: string, index: number) => index % 22 === shard),
    );
    assert.ok(name.length < 240, name);
  }
  const byName = (left: string, right: string) => left.localeCompare(right);
  assert.deepEqual(
    matrix.flatMap(({ name }) => name.split(", ")).sort(byName),
    [...projects].sort(byName),
  );
  assert.equal(new Set(matrix.map(({ name }) => name)).size, matrix.length);
  const planned = JSON.parse(
    execFileSync(
      process.execPath,
      ["tools/support/compat/github/real-project-matrix-plan.ts", "matrix"],
      { cwd: root, encoding: "utf8" },
    ),
  );
  assert.deepEqual(planned, { include: matrix });
});

test("historical and named job evidence reads only its selected source revision", () => {
  const source = matrixSourceFixture();
  const old = realProjectJobNamesAtRevision(source.legacySha, source.repoRoot);
  const named = realProjectJobNamesAtRevision(source.namedSha, source.repoRoot);
  assert.deepEqual(
    old,
    Array.from({ length: 22 }, (_, shard) => `real projects (${shard}/22)`),
  );
  assert.match(named[0], /^real projects \(vue-vben-admin, pinia, /);
  assert.doesNotMatch(named[0], /uncommitted-project/);
  const cli = JSON.parse(
    execFileSync(
      process.execPath,
      [
        "tools/support/compat/github/real-project-matrix-plan.ts",
        "job-names",
        "--revision",
        source.namedSha,
        "--root",
        source.repoRoot,
      ],
      { cwd: root, encoding: "utf8" },
    ),
  );
  assert.deepEqual(cli, named, "Rust CLI and JS share one name authority");
  for (const [headSha, names, wrong] of [
    [source.legacySha, old, named],
    [source.namedSha, named, old],
  ] as const) {
    const run = { head_sha: headSha };
    const jobs = names.map((name) => ({ name, status: "completed", conclusion: "success" }));
    assert.deepEqual(requiredWorkflowJobNames("Real Project Matrix", run, source.repoRoot), names);
    assert.doesNotThrow(() =>
      assertRequiredWorkflowJobs("Real Project Matrix", jobs, run, source.repoRoot),
    );
    assert.throws(
      () =>
        assertRequiredWorkflowJobs(
          "Real Project Matrix",
          jobs.map((job, index) => ({ ...job, name: wrong[index] })),
          run,
          source.repoRoot,
        ),
      /found 0/,
    );
  }
});

test("unknown, unavailable or ambiguous source naming fails closed", () => {
  const source = matrixSourceFixture();
  assert.throws(
    () => realProjectJobNamesAtRevision(source.unknownSha, source.repoRoot),
    /Unknown .* template/,
  );
  assert.throws(() => realProjectJobNamesAtRevision("HEAD", source.repoRoot), /exact source SHA/);
  assert.throws(
    () => realProjectJobNamesAtRevision("e".repeat(40), source.repoRoot),
    /source is unavailable/,
  );
  assert.throws(() => requiredWorkflowJobNames("Real Project Matrix"), /exact source SHA/);
  const workflow = readRepoFile(realProjectWorkflowPath);
  const registry = JSON.parse(readRepoFile(realProjectRegistryPath));
  assert.throws(
    () =>
      realProjectJobNames(
        workflow.replace(realProjectJobTemplate, "projects (${{ matrix.name }})"),
        registry,
      ),
    /Unknown .* template/,
  );
  assert.throws(
    () =>
      realProjectJobNames(
        workflow.replace('FIXTURE_SHARD_COUNT: "22"', 'FIXTURE_SHARD_COUNT: "21"'),
        registry,
      ),
    /shard routing/,
  );
  assert.throws(
    () => realProjectMatrix({ projects: registry.projects.slice(0, 1) }),
    /complete fixture registry/,
  );
  assert.throws(
    () => realProjectMatrix({ projects: [...registry.projects, registry.projects[0]] }),
    /unique project IDs/,
  );
});
