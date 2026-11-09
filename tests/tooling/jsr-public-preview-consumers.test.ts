import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { after, test } from "node:test";
import { parse } from "yaml";

import {
  bunPreviewCommands,
  prepareBunPreviewConsumer,
  prepareStagedDenoPreview,
  stagedDenoArguments,
} from "../../tools/support/release/jsr/public-preview-consumers.ts";

const output = mkdtempSync(join(tmpdir(), "vize-jsr-bun-preview-test-"));
after(() => rmSync(output, { recursive: true, force: true }));

test("the Bun preview project contains only the shared consumer and minimal manifest", () => {
  const sourceRoot = resolve(import.meta.dirname, "../..");
  prepareBunPreviewConsumer(output, sourceRoot);
  assert.deepEqual(JSON.parse(readFileSync(join(output, "package.json"), "utf8")), {
    name: "vize-jsr-bun-preview",
    private: true,
    type: "module",
  });
  assert.equal(
    readFileSync(join(output, "consumer.ts"), "utf8"),
    readFileSync(join(sourceRoot, "tools/support/release/jsr/consumer.ts"), "utf8"),
  );
});

test("the Bun preview installs and executes every public tool facade", () => {
  assert.deepEqual(bunPreviewCommands("0.605.0"), [
    {
      executable: "bun",
      arguments: [
        "add",
        "--ignore-scripts",
        "--exact",
        "@vizejs/babel-plugin@0.605.0",
        "oxlint-plugin-vize@0.605.0",
        "vite@8.2.4",
        "vue@3.5.42",
      ],
      log: "install.log",
    },
    {
      executable: "bunx",
      arguments: ["--bun", "jsr@0.14.3", "add", "@vizejs/vize@0.605.0"],
      log: "jsr-add.log",
    },
    {
      executable: "bun",
      arguments: [
        "run",
        "consumer.ts",
        "@vizejs/vize",
        "--vite",
        "--babel-specifier",
        "@vizejs/babel-plugin",
        "--oxlint-specifier",
        "oxlint-plugin-vize",
      ],
      log: "runtime.log",
    },
  ]);
  assert.throws(() => bunPreviewCommands("latest"), /Invalid JSR preview version/);
});

test("the staged Deno preview copies the shared policy and imports a directory URL", () => {
  const sourceRoot = resolve(import.meta.dirname, "../..");
  const denoOutput = join(output, "deno");
  const stagedPackage = join(output, "packages", "vize");
  const prepared = prepareStagedDenoPreview(denoOutput, sourceRoot);
  assert.equal(
    readFileSync(prepared.config, "utf8"),
    readFileSync(join(sourceRoot, "tools/support/release/jsr/consumer-deno.json"), "utf8"),
  );
  assert.equal(
    readFileSync(prepared.consumer, "utf8"),
    readFileSync(join(sourceRoot, "tools/support/release/jsr/consumer.ts"), "utf8"),
  );
  const arguments_ = stagedDenoArguments(denoOutput, stagedPackage);
  assert.deepEqual(arguments_.slice(0, 7), [
    "run",
    "--config",
    prepared.config,
    "--lock",
    prepared.lock,
    "--allow-all",
    prepared.consumer,
  ]);
  assert.match(arguments_[7] ?? "", /^file:\/\/.+\/packages\/vize\/$/);
});

test("the hosted package workflow keeps orchestration out of YAML", () => {
  const file = resolve(import.meta.dirname, "../../.github/workflows/jsr-package.yml");
  const workflow = parse(readFileSync(file, "utf8")) as {
    jobs: { package: { steps: readonly { run?: string }[] } };
  };
  assert.ok(workflow.jobs.package.steps.every((step) => !step.run?.includes("\n")));
});
