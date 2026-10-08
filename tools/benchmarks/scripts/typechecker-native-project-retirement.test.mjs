/** Adversarial observer controls; only the real hosted native law grants product credit. */
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import {
  authenticateCopiedObserver,
  qualifyNativeProjectRetirement,
} from "./typechecker-native-project-retirement.mjs";

function fixture(mode, body) {
  const root = mkdtempSync(join(tmpdir(), "vize-project-retirement-observer-"));
  try {
    const controlPath = join(root, "native-control");
    writeFileSync(controlPath, "observer control, not a native product receipt");
    const nativeBinary = realpathSync(controlPath);
    const recipePath = join(root, "recipe.json");
    writeFileSync(recipePath, JSON.stringify({ mode }));
    for (const name of ["editor-original-script", "editor-monorepo-alias"]) {
      mkdirSync(join(root, name));
      writeFileSync(join(root, name, "runtime.json"), JSON.stringify({ nativeBinary }));
    }
    body({ capture: root, recipePath, sourceSha: "a".repeat(40), nativeBinary, sourceRoot: root });
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

await test("immutable historical source needs no new test or fixture and receives no process credit", () => {
  fixture("immutable-source-step", (options) => {
    const receipt = qualifyNativeProjectRetirement({
      ...options,
      run: () => assert.fail("historical sources must not run the new native law"),
    });
    assert.deepEqual(receipt, {
      status: "not-qualified",
      reason: "historical-immutable-source",
      sourceSha: options.sourceSha,
    });
  });
});

await test("unknown recipe mode cannot fall back to historical no-credit behavior", () => {
  fixture("unknown", (options) =>
    assert.throws(() =>
      qualifyNativeProjectRetirement({
        ...options,
        run: () => assert.fail("unknown mode must fail before launching anything"),
      }),
    ),
  );
});

await test("a successful zero-test filter cannot qualify the physical native law", () => {
  fixture("current-inline", (options) => {
    let calls = 0;
    assert.throws(() =>
      qualifyNativeProjectRetirement({
        ...options,
        run: () => {
          calls++;
          return { status: 0, signal: null, stdout: "running 0 tests\n", stderr: "" };
        },
      }),
    );
    assert.equal(calls, 1);
  });
});

await test("historical mode rejects a stale producer log instead of reusing its credit", () => {
  fixture("immutable-source-step", (options) => {
    writeFileSync(join(options.capture, "native-project-retirement-3952.log"), "stale");
    assert.throws(() => qualifyNativeProjectRetirement(options));
  });
});

await test("historical mode retains the original editor and monorepo alias binary gates", () => {
  fixture("immutable-source-step", (options) => {
    const foreign = join(options.capture, "foreign-control");
    writeFileSync(foreign, "different binary");
    writeFileSync(
      join(options.capture, "editor-monorepo-alias", "runtime.json"),
      JSON.stringify({ nativeBinary: foreign }),
    );
    assert.throws(() => qualifyNativeProjectRetirement(options));
  });
});

await test("copied observer requires its immutable workflow bytes and exact source/recipe binding", () => {
  const root = mkdtempSync(join(tmpdir(), "vize-project-retirement-custody-"));
  try {
    const git = (...args) => {
      const result = spawnSync("git", args, { cwd: root, encoding: "utf8" });
      assert.equal(result.status, 0, result.stderr);
      return result.stdout.trim();
    };
    git("init");
    git("config", "user.name", "Observer control");
    git("config", "user.email", "observer-control@example.invalid");
    const directory = join(root, "tools/benchmarks/scripts");
    mkdirSync(directory, { recursive: true });
    const helper = join(directory, "typechecker-native-project-retirement.mjs");
    const bytes = readFileSync(
      new URL("typechecker-native-project-retirement.mjs", import.meta.url),
    );
    writeFileSync(helper, bytes);
    git("add", ".");
    git("commit", "-m", "fixture");
    const sourceSha = git("rev-parse", "HEAD");
    const recipePath = join(root, "recipe.json");
    const recipe = {
      source: sourceSha,
      sourceTree: git("rev-parse", "HEAD^{tree}"),
      workflow: sourceSha,
    };
    writeFileSync(recipePath, JSON.stringify(recipe));
    const options = { recipePath, sourceSha, workflowSha: sourceSha, sourceRoot: root };
    authenticateCopiedObserver(options);
    writeFileSync(recipePath, JSON.stringify({ ...recipe, workflow: "b".repeat(40) }));
    assert.throws(() => authenticateCopiedObserver(options));
    writeFileSync(
      helper,
      Buffer.concat([bytes, Buffer.from("\n// different immutable observer\n")]),
    );
    git("add", "tools");
    git("commit", "-m", "different observer");
    const changed = git("rev-parse", "HEAD");
    writeFileSync(
      recipePath,
      JSON.stringify({
        source: changed,
        sourceTree: git("rev-parse", "HEAD^{tree}"),
        workflow: changed,
      }),
    );
    assert.throws(() =>
      authenticateCopiedObserver({ ...options, sourceSha: changed, workflowSha: changed }),
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
