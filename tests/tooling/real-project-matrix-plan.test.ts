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
        workflow.replace("matrix.shardCount || '22'", "matrix.shardCount || '21'"),
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

test("scoped diagnostics select exactly one original pinned row without changing full partitions", () => {
  const registry = JSON.parse(readRepoFile(realProjectRegistryPath));
  const full = realProjectMatrix(registry);
  assert.equal(registry.projects.length, 150);
  assert.deepEqual(realProjectMatrix(registry, ""), full);
  for (const [index, project] of registry.projects.entries()) {
    const scoped = realProjectMatrix(registry, project.id);
    assert.deepEqual(scoped, [
      {
        shard: index,
        name: `scoped diagnostic: ${project.id}`,
        shardCount: registry.projects.length,
      },
    ]);
    const selected = registry.projects.filter(
      (_: unknown, row: number) => row % registry.projects.length === scoped[0].shard,
    );
    assert.deepEqual(selected, [project]);
    assert.equal(selected[0], project, "the original registry row is not rewritten");
  }
  const original = registry.projects.find((project: { id: string }) => project.id === "hoppscotch");
  assert.equal(original.revision, "1acb8a3a7581e4db32ba0d529170c4669a2e1053");
  const cli = (args: string[]) =>
    execFileSync(
      process.execPath,
      ["tools/support/compat/github/real-project-matrix-plan.ts", "matrix", ...args],
      { cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
    );
  assert.deepEqual(JSON.parse(cli(["--project", ""])), { include: full });
  const scoped = realProjectMatrix(registry, "hoppscotch");
  assert.deepEqual(JSON.parse(cli(["--project", "hoppscotch"])), { include: scoped });
  for (const invalid of [
    "missing-project",
    " hoppscotch",
    "hoppscotch ",
    "hoppscotch,pinia",
    "../hoppscotch",
  ]) {
    assert.throws(() => realProjectMatrix(registry, invalid), /Unknown .* project/);
    assert.throws(() => cli(["--project", invalid]), /Unknown .* project/);
  }
  for (const invalid of [
    ["--project"],
    ["--project", "hoppscotch", "unexpected"],
    ["--unknown", "hoppscotch"],
  ]) {
    assert.throws(() => cli(invalid), /Usage:/);
  }
});
