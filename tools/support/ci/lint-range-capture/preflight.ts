import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import {
  existsSync,
  readFileSync,
  realpathSync,
  statSync,
  openSync,
  closeSync,
  readSync,
  mkdtempSync,
} from "node:fs";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { dirname, join, relative } from "node:path";
import {
  isMain,
  cliArgs,
  git,
  readJson,
  save,
  sha256,
  source,
  existingJson,
  directory,
} from "./common.ts";
import { pins, reporterBaseline, versions } from "./pins.ts";
import { projects } from "./projects.ts";

type GitOutput = { path: string; bytes: number; sha256: string };
function outputRecord(out: string, path: string): GitOutput {
  const hash = createHash("sha256"),
    buffer = Buffer.alloc(64 * 1024);
  const fd = openSync(path, "r");
  try {
    for (;;) {
      const count = readSync(fd, buffer);
      if (!count) break;
      hash.update(buffer.subarray(0, count));
    }
  } finally {
    closeSync(fd);
  }
  return { path: relative(out, path), bytes: statSync(path).size, sha256: hash.digest("hex") };
}
function retainGit(out: string, folder: string, cwd: string, label: string, ...args: string[]) {
  const stdoutPath = join(folder, label + ".stdout.log");
  const stderrPath = join(folder, label + ".stderr.log");
  const stdoutFd = openSync(stdoutPath, "wx"),
    stderrFd = openSync(stderrPath, "wx");
  try {
    execFileSync("git", ["-C", cwd, ...args], { stdio: ["ignore", stdoutFd, stderrFd] });
  } finally {
    closeSync(stdoutFd);
    closeSync(stderrFd);
  }
  const stdout = outputRecord(out, stdoutPath),
    stderr = outputRecord(out, stderrPath);
  // Inline small original values; larger output is still retained in full, never truncated.
  const value = stdout.bytes <= 64 * 1024 ? readFileSync(stdoutPath, "utf8") : stdout;
  return { stdout, stderr, value };
}
export function verifyTrackedSource(root: string, expected: string) {
  const out = directory(root),
    folder = mkdtempSync(join(out, "source-git-"));
  const names = retainGit(out, folder, root, "tracked-names", "diff", "--name-only", "HEAD");
  if (names.stdout.bytes) {
    if (!existingJson(root, "source-drift.json")) {
      const outputs: { label: string; stdout: GitOutput; stderr: GitOutput }[] = [];
      const observe = (cwd: string, label: string, ...args: string[]) => {
        const result = retainGit(out, folder, cwd, label, ...args);
        outputs.push({ label, stdout: result.stdout, stderr: result.stderr });
        return result.value;
      };
      outputs.push({ label: "tracked-names", stdout: names.stdout, stderr: names.stderr });
      save(root, "source-drift.json", {
        expectedSource: expected,
        actualSource: git(root, "rev-parse", "HEAD").toString().trim(),
        trackedPaths: names.value,
        repositoryStatus: observe(
          root,
          "repository-status",
          "status",
          "--porcelain",
          "--untracked-files=normal",
        ),
        repositoryDiff: observe(
          root,
          "repository-diff",
          "diff",
          "--no-ext-diff",
          "--submodule=diff",
          "HEAD",
        ),
        fixtures: projects
          .filter((p) => existsSync(join(root, p.fixturePath, ".git")))
          .map((p) => {
            const cwd = join(root, p.fixturePath);
            return {
              id: p.id,
              expectedRevision: p.revision,
              actualRevision: git(cwd, "rev-parse", "HEAD").toString().trim(),
              status: observe(
                cwd,
                p.id + "-status",
                "status",
                "--porcelain",
                "--untracked-files=normal",
              ),
              diff: observe(
                cwd,
                p.id + "-diff",
                "diff",
                "--no-ext-diff",
                "--submodule=diff",
                "HEAD",
              ),
            };
          }),
        gitOutputs: outputs,
        producerExecuted: false,
        acceptance: false,
      });
    }
    console.error(
      "Tracked source has edits:\n" +
        (typeof names.value === "string" ? names.value : JSON.stringify(names.value)),
    );
  }
  assert.equal(names.stdout.bytes, 0, "Tracked source has edits");
}

export function verifySource(root: string, expected: string) {
  source(root, expected);
  verifyTrackedSource(root, expected);
  git(root, "merge-base", "--is-ancestor", reporterBaseline, expected);
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
    reporterSource: expected,
    reporterBaseline,
    pins: pinRows,
    registrySha256: sha256(readFileSync(join(root, registryPath))),
    projects: projects.map((p) => ({ id: p.id, revision: p.revision })),
  };
}
export function verifyFixtureStatus(
  root: string,
  project: (typeof projects)[number],
  expected: string,
  phase: string,
) {
  const cwd = join(root, project.fixturePath),
    out = directory(root),
    folder = mkdtempSync(join(out, "fixture-status-git-"));
  const status = retainGit(
    out,
    folder,
    cwd,
    "status",
    "status",
    "--porcelain",
    "--untracked-files=normal",
  );
  if (status.stdout.bytes) {
    if (!existingJson(root, "fixture-runtime-drift.json")) {
      const outputs = [{ label: "status", stdout: status.stdout, stderr: status.stderr }];
      const observe = (label: string, ...args: string[]) => {
        const result = retainGit(out, folder, cwd, label, ...args);
        outputs.push({ label, stdout: result.stdout, stderr: result.stderr });
        return result.value;
      };
      save(root, "fixture-runtime-drift.json", {
        phase,
        expectedSource: expected,
        actualSource: git(root, "rev-parse", "HEAD").toString().trim(),
        fixture: {
          id: project.id,
          path: project.fixturePath,
          expectedRevision: project.revision,
          actualRevision: git(cwd, "rev-parse", "HEAD").toString().trim(),
        },
        originalStatusBytes: status.stdout.bytes,
        originalStatus: status.value,
        completeStatus: observe(
          "complete-status",
          "status",
          "--porcelain",
          "--untracked-files=all",
        ),
        trackedDiff: observe("tracked-diff", "diff", "--no-ext-diff", "HEAD"),
        untrackedPaths: observe(
          "untracked-paths",
          "ls-files",
          "--others",
          "--exclude-standard",
          "-z",
        ),
        gitOutputs: outputs,
        acceptance: false,
      });
    }
    console.error(
      "Fixture source has edits (" +
        phase +
        ", " +
        project.id +
        "):\n" +
        (typeof status.value === "string" ? status.value : JSON.stringify(status.value)),
    );
  }
  assert.equal(status.stdout.bytes, 0, "Fixture source has edits");
}
export function verifyRuntime(root: string, expected: string, phase = "pre-reporter-runtime") {
  const bound = verifySource(root, expected);
  assert.equal(process.version, "v24.14.0", "Pinned Node differs");
  const fixtures = projects.map((project) => {
    assert.ok(
      git(root, "submodule", "status", "--", project.fixturePath)
        .toString()
        .startsWith(" " + project.revision + " "),
    );
    verifyFixtureStatus(root, project, expected, phase);
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
