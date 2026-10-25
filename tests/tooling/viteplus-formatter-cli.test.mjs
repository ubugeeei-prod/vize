import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";
import { root } from "./support/lsp/paths.ts";

const wide =
  "const value110 = someFunction(argumentNumberOne, argumentNumberTwo, argumentNumberThree, argumentNumber4);\n";
const files = {
  "a110.ts": wide,
  "App.vue": `<script setup lang="ts">\n${wide}</script>\n<template><div /></template>\n`,
  "ignored/b.ts": "const   x=1\n",
  "ignored/Keep.ts": "const   keep=2\n",
  "日本語/🎨.ts": "const   color=3\n",
};

void test("source CLI shares Vite+ formatter width and project-root exclusions across check and write", (t) => {
  const [binary] = resolveVizeLaunchCommand(undefined, path.join(root, "target/ci/vize"), {
    required: true,
    repoRoot: root,
  });
  const directory = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-fmt-config-")));
  const evidence = path.join(root, "target/differential/viteplus-formatter-settings/cli.json");
  const rows = [];
  const started = performance.now();
  try {
    fs.writeFileSync(path.join(directory, "package.json"), '{"type":"module"}');
    fs.symlinkSync(
      path.join(root, "node_modules"),
      path.join(directory, "node_modules"),
      "junction",
    );
    for (const project of [".", "app"]) {
      const projectRoot = path.join(directory, project);
      const reset = () => {
        for (const [file, source] of Object.entries(files)) {
          const selected = path.join(projectRoot, file);
          fs.mkdirSync(path.dirname(selected), { recursive: true });
          fs.writeFileSync(selected, source);
        }
      };
      const snapshot = () =>
        Object.fromEntries(
          Object.keys(files).map((file) => [
            file,
            fs.readFileSync(path.join(projectRoot, file), "utf8"),
          ]),
        );
      const config = (options) => {
        fs.writeFileSync(
          path.join(directory, "vite.config.ts"),
          `export default ${JSON.stringify({ root: project, ...options })};\n`,
        );
      };
      const run = (args) => {
        const before = snapshot();
        const start = performance.now();
        const result = spawnSync(binary, ["fmt", ...args], {
          cwd: directory,
          encoding: "utf8",
          env: { ...process.env, NO_COLOR: "1" },
          timeout: 30_000,
        });
        const after = snapshot();
        rows.push({
          project,
          args,
          config: fs.readFileSync(path.join(directory, "vite.config.ts"), "utf8"),
          status: result.status,
          signal: result.signal,
          error: result.error?.message ?? null,
          stdout: result.stdout,
          stderr: result.stderr,
          before,
          after,
          milliseconds: performance.now() - start,
        });
        assert.equal(result.error, undefined);
        assert.equal(result.signal, null);
        assert.equal(result.stdout, "");
        return { ...result, before, after };
      };
      reset();
      const inputs = ["a110.ts", "App.vue"].map((file) => path.join(project, file));
      config({});
      assert.equal(run(["--write", "--no-config", "--print-width", "120", ...inputs]).status, 0);
      const reference = snapshot();
      reset();
      config({ fmt: { printWidth: 120 } });
      assert.equal(run(["--write", ...inputs]).status, 0);
      assert.deepEqual(
        snapshot(),
        reference,
        "complete TS and SFC bytes equal the explicit CLI width control",
      );
      assert.equal(run(["--check", ...inputs]).status, 0);
      const noConfig = run(["--check", "--no-config", ...inputs]);
      assert.equal(noConfig.status, 1, "the reported width fixture must distinguish the default");
      assert.deepEqual(noConfig.after, noConfig.before);
      config({
        fmt: { printWidth: 120, vue: { printWidth: 80 } },
        vue: { formatter: { printWidth: 90 } },
      });
      reset();
      assert.equal(run(["--write", "--print-width", "120", ...inputs]).status, 0);
      assert.deepEqual(
        snapshot(),
        reference,
        "explicit CLI options win over both Vue config layers",
      );

      for (const pattern of ["ignored", "ignored/**", "**/ignored/**"]) {
        reset();
        config({ fmt: { ignorePatterns: [pattern, "!ignored/Keep.ts", "日本語/**"] } });
        const discovered = run(["--check"]);
        assert.equal(discovered.status, 1);
        assert.deepEqual(discovered.after, discovered.before);
        const discoveredPaths = [...discovered.stderr.matchAll(/^Would reformat: (.+)$/gm)].map(
          (match) => path.relative(projectRoot, path.resolve(directory, match[1])),
        );
        assert.ok(discoveredPaths.includes("ignored/Keep.ts"));
        assert.ok(!discoveredPaths.includes("ignored/b.ts"));
        assert.ok(!discoveredPaths.includes("日本語/🎨.ts"));
        const selected = ["ignored/b.ts", "ignored/Keep.ts", "日本語/🎨.ts"].map((file) =>
          path.join(project, file),
        );
        const check = run(["--check", ...selected]);
        assert.equal(check.status, 1);
        assert.deepEqual(check.after, check.before);
        const changed = [...check.stderr.matchAll(/^Would reformat: (.+)$/gm)].map((match) =>
          path.relative(projectRoot, path.resolve(directory, match[1])),
        );
        assert.deepEqual(changed, ["ignored/Keep.ts"]);
        assert.equal(run(["--write", ...selected]).status, 0);
        const expected = { ...files, "ignored/Keep.ts": "const keep = 2;\n" };
        assert.deepEqual(snapshot(), expected, "excluded files retain every original byte");
        assert.equal(run(["--check", ...selected]).status, 0);
      }
      reset();
      config({ fmt: { printWidth: "120", ignorePatterns: ["ignored"] } });
      assert.equal(run(["--write", ...inputs]).status, 2);
      assert.deepEqual(snapshot(), files, "invalid settings refuse before writes");
      if (project === ".") {
        for (const file of Object.keys(files)) fs.rmSync(path.join(projectRoot, file));
      }
    }
  } finally {
    fs.mkdirSync(path.dirname(evidence), { recursive: true });
    fs.writeFileSync(
      evidence,
      JSON.stringify({ issue: 10614, rows, milliseconds: performance.now() - started }, null, 2) +
        "\n",
    );
    fs.rmSync(directory, { recursive: true, force: true });
    t.diagnostic(`Complete raw source CLI packets and file custody: ${evidence}`);
  }
});
