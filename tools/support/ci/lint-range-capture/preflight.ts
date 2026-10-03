import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, realpathSync, statSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { isMain, cliArgs, git, readJson, save, sha256, source } from "./common.ts";
import { pins, reporterSource, versions } from "./pins.ts";
import { projects } from "./projects.ts";

export function verifySource(root: string, expected: string) {
  source(root, expected);
  assert.equal(git(root, "diff", "--name-only", "HEAD").length, 0, "Tracked source has edits");
  git(root, "merge-base", "--is-ancestor", reporterSource, expected);
  const pinRows = pins.map((pin) => {
    const bytes = readFileSync(join(root, pin.path));
    assert.equal(sha256(bytes), pin.sha256, "Producer/control drift: " + pin.path);
    assert.equal(git(root, "ls-tree", expected, "--", pin.path).toString().trim(), pin.gitEntry);
    return { path: pin.path, bytes: bytes.length, sha256: pin.sha256, gitEntry: pin.gitEntry };
  });
  assert.equal(
    git(root, "diff", "--name-only", "HEAD", "--", ...pins.map((pin) => pin.path)).length,
    0,
  );
  const registryPath = "tests/_fixtures/vue-ecosystem-fixtures.json";
  assert.equal(git(root, "diff", "--name-only", "HEAD", "--", registryPath).length, 0);
  const registry = readJson(join(root, registryPath));
  for (const project of projects) {
    const actual = registry.projects.find((row: any) => row.id === project.id);
    assert.ok(actual, "Missing original project: " + project.id);
    assert.deepEqual(
      {
        id: actual.id,
        fixturePath: actual.fixturePath,
        repository: actual.repository,
        revision: actual.revision,
        vueGlobs: actual.vueGlobs,
        expectedVueFileCount: actual.expectedVueFileCount ?? null,
        linterCovered: actual.coverage.includes("linter"),
      },
      {
        id: project.id,
        fixturePath: project.fixturePath,
        repository: project.repository,
        revision: project.revision,
        vueGlobs: project.vueGlobs,
        expectedVueFileCount: project.expectedVueFileCount,
        linterCovered: project.linterCovered,
      },
    );
    assert.equal(
      git(root, "ls-tree", expected, "--", project.fixturePath).toString().trim(),
      project.gitEntry,
    );
  }
  return {
    expectedSource: expected,
    reporterSource,
    pins: pinRows,
    registrySha256: sha256(readFileSync(join(root, registryPath))),
    projects: projects.map((p) => ({ id: p.id, revision: p.revision })),
  };
}
export function verifyRuntime(root: string, expected: string) {
  const bound = verifySource(root, expected);
  assert.equal(process.version, "v24.14.0", "Pinned Node differs");
  const fixtures = projects.map((project) => {
    const cwd = join(root, project.fixturePath);
    assert.ok(
      git(root, "submodule", "status", "--", project.fixturePath)
        .toString()
        .startsWith(" " + project.revision + " "),
    );
    assert.equal(
      execFileSync("git", ["-C", cwd, "status", "--porcelain", "--untracked-files=normal"]).length,
      0,
    );
    return { id: project.id, revision: project.revision, clean: true };
  });
  const require = createRequire(join(root, "tools/benchmarks/scripts/package.json"));
  const runtime = Object.entries(versions).map(([name, version]) => {
    const entry = realpathSync(require.resolve(name));
    let parent = dirname(entry),
      manifestPath = "",
      manifest: any;
    while (parent !== dirname(parent)) {
      manifestPath = join(parent, "package.json");
      if (existsSync(manifestPath)) {
        manifest = readJson(manifestPath);
        if (manifest.name === name) break;
      }
      parent = dirname(parent);
    }
    assert.equal(manifest?.name, name);
    assert.equal(manifest.version, version, "Installed dependency drift: " + name);
    return {
      name,
      version,
      entry,
      entrySha256: sha256(readFileSync(entry)),
      manifestSha256: sha256(readFileSync(manifestPath)),
    };
  });
  const binary = join(root, "target/ci/vize");
  assert.ok(
    existsSync(binary) && statSync(binary).isFile() && statSync(binary).mode & 0o111,
    "No built CLI; forbid reporter Cargo fallback",
  );
  return {
    ...bound,
    fixtures,
    node: process.version,
    runtime,
    cli: { path: binary, bytes: statSync(binary).size, sha256: sha256(readFileSync(binary)) },
  };
}
if (isMain(import.meta.url)) {
  const { mode, root, expected } = cliArgs();
  assert.ok(["source", "paths", "runtime"].includes(mode));
  const receipt = mode === "runtime" ? verifyRuntime(root, expected) : verifySource(root, expected);
  if (mode === "paths") console.log(projects.map((project) => project.fixturePath).join("\n"));
  else {
    save(root, mode + "-preflight.json", {
      ...receipt,
      producerExecuted: false,
      acceptance: false,
    });
    console.log(mode + " preflight passed");
  }
}
