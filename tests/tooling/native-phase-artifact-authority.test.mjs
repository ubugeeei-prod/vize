import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { reconstruct } from "../../tools/support/compat/github/native-phase-artifact.mjs";
import {
  manifestName,
  rawPaths,
} from "../../tools/support/compat/github/native-phase-artifact-files.mjs";

const closure = [
  "native-phase-artifact.mjs",
  "native-phase-artifact-files.mjs",
  "native-phase-artifact-paths.json",
];
const prefix = "tools/support/compat/github";
const workflow = parse(
  readFileSync(
    new URL("../../.github/workflows/typechecker-native-phases.yml", import.meta.url),
    "utf8",
  ),
);
const script = workflow.jobs["native-phases"].steps.find(
  (step) => step.id === "native-artifact-pack",
).run;
const put = (path, bytes) => {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, bytes);
};
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");

function fixture(t) {
  const root = mkdtempSync(join(tmpdir(), "vize-native-authority-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const origin = join(root, "origin");
  const checkout = join(root, "checkout");
  const capture = join(root, "capture");
  const dependencies = join(root, "dependencies");
  mkdirSync(origin);
  mkdirSync(capture);
  const git = (cwd, ...args) => {
    const result = spawnSync("git", args, {
      cwd,
      encoding: "utf8",
      env: {
        ...process.env,
        GIT_CONFIG_NOSYSTEM: "1",
        GIT_CONFIG_GLOBAL: "/dev/null",
        GIT_AUTHOR_NAME: "Native authority fixture",
        GIT_AUTHOR_EMAIL: "fixture@example.invalid",
        GIT_COMMITTER_NAME: "Native authority fixture",
        GIT_COMMITTER_EMAIL: "fixture@example.invalid",
      },
    });
    assert.equal(result.status, 0, result.stderr);
    return result.stdout.trim();
  };
  git(origin, "init", "--initial-branch=main");
  put(join(origin, "source.txt"), "old production checkout without transport helpers\n");
  git(origin, "add", ".");
  git(origin, "commit", "-m", "fixture: original source");
  const sourceSha = git(origin, "rev-parse", "HEAD");
  git(root, "clone", "--no-local", origin, checkout);
  for (const file of closure) {
    put(
      join(origin, prefix, file),
      readFileSync(new URL(`../../${prefix}/${file}`, import.meta.url)),
    );
  }
  git(origin, "add", ".");
  git(origin, "commit", "-m", "fixture: immutable workflow helper closure");
  const authority = git(origin, "rev-parse", "HEAD");
  put(join(dependencies, "opaque.bin"), Buffer.from([0, 255, 128, 10, 0]));
  put(join(dependencies, ".hidden", "raw.d.ts"), "export type 雪 = 'complete';\n");
  const expected = new Map();
  for (let index = 0; index < 21; index++) {
    const relative = `jsx-slot-parameter-types/case-${index}`;
    mkdirSync(join(capture, relative), { recursive: true });
    symlinkSync(dependencies, join(capture, relative, "node_modules"), "dir");
    put(join(capture, relative, "stderr.raw"), index === 0 ? "original failure\n" : "");
    for (const file of ["opaque.bin", ".hidden/raw.d.ts"]) {
      expected.set(`${relative}/node_modules/${file}`, readFileSync(join(dependencies, file)));
    }
    expected.set(`${relative}/stderr.raw`, readFileSync(join(capture, relative, "stderr.raw")));
  }
  put(join(capture, "typechecker-native-source-custody.json"), '{"original":true}\n');
  expected.set(
    "typechecker-native-source-custody.json",
    readFileSync(join(capture, "typechecker-native-source-custody.json")),
  );
  const output = join(root, "github-output");
  const run = (sha = authority) => {
    rmSync(output, { force: true });
    return spawnSync("bash", ["--noprofile", "--norc", "-e", "-o", "pipefail", "-c", script], {
      cwd: checkout,
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${dirname(process.execPath)}:${process.env.PATH}`,
        GITHUB_WORKFLOW_SHA: sha,
        GITHUB_OUTPUT: output,
        RUNNER_TEMP: capture,
        SOURCE_SHA: sourceSha,
        DRIVER_SHA: sourceSha,
      },
    });
  };
  return { origin, checkout, capture, root, output, git, sourceSha, authority, run, expected };
}

function assertRawFallback(f) {
  const emitted = readFileSync(f.output, "utf8");
  const original = `raw_paths<<NATIVE_RAW_PATHS\n${rawPaths.map((path) => `${f.capture}/${path}`).join("\n")}\nNATIVE_RAW_PATHS\n`;
  assert(emitted.startsWith(original), "every authority refusal retains all original raw paths");
  assert.equal(readdirSync(f.capture).includes("native-phase-artifact"), false);
}

void test("old source and poisoned checkout both pack the full raw tree using immutable workflow objects", (t) => {
  for (const poisoned of [false, true]) {
    const f = fixture(t);
    assert.equal(existsSync(join(f.checkout, prefix)), false);
    if (poisoned) {
      for (const file of closure) put(join(f.checkout, prefix, file), "checkout poison\n");
    }
    const result = f.run();
    assert.equal(result.status, 0, result.stderr);
    const envelope = join(f.capture, "native-phase-artifact");
    const receipt = JSON.parse(readFileSync(join(envelope, "receipt.json"), "utf8"));
    assert.equal(receipt.sourceSha, f.sourceSha);
    assert.equal(receipt.driverSha, f.sourceSha);
    assert.equal(receipt.fileCount, f.expected.size);
    const restored = join(f.root, "restored");
    reconstruct(envelope, restored);
    const manifest = JSON.parse(readFileSync(join(restored, manifestName), "utf8"));
    assert.deepEqual(
      manifest.files.map((row) => row.path).sort((left, right) => left.localeCompare(right, "en")),
      [...f.expected.keys()].sort((left, right) => left.localeCompare(right, "en")),
    );
    for (const row of manifest.files) {
      const original = f.expected.get(row.path);
      assert.deepEqual(readFileSync(join(restored, row.path)), original);
      assert.equal(row.sha256, digest(original));
      assert.equal(row.bytes, original.length);
    }
    assert.equal(f.git(f.checkout, "rev-parse", "HEAD"), f.sourceSha);
    for (const file of closure) {
      if (poisoned)
        assert.equal(readFileSync(join(f.checkout, prefix, file), "utf8"), "checkout poison\n");
      else assert.equal(existsSync(join(f.checkout, prefix, file)), false);
    }
  }
});

void test("malformed, missing and non-commit workflow authority refuse while retaining complete raw custody", (t) => {
  for (const kind of ["malformed", "missing", "tree"]) {
    const f = fixture(t);
    const sha =
      kind === "malformed"
        ? "main"
        : kind === "missing"
          ? "f".repeat(40)
          : f.git(f.origin, "rev-parse", "HEAD^{tree}");
    const result = f.run(sha);
    assert.notEqual(result.status, 0, `${kind} authority must fail`);
    assertRawFallback(f);
    assert.equal(f.git(f.checkout, "rev-parse", "HEAD"), f.sourceSha);
  }
});

void test("every missing authority closure member refuses without using a poisoned checkout fallback", (t) => {
  for (const missing of closure) {
    const f = fixture(t);
    f.git(f.origin, "rm", `${prefix}/${missing}`);
    f.git(f.origin, "commit", "-m", "fixture: missing authenticated helper member");
    for (const file of closure) put(join(f.checkout, prefix, file), "checkout poison\n");
    const result = f.run(f.git(f.origin, "rev-parse", "HEAD"));
    assert.notEqual(result.status, 0, `${missing} must fail`);
    assert.match(result.stderr, /does not exist in|exists on disk, but not in/);
    assertRawFallback(f);
  }
});
