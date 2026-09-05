import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

const repoRoot = path.resolve(import.meta.dirname, "..", "..");

test("the Rust runner counts the same ignore-filtered files as vize lint", () => {
  fs.mkdirSync(path.join(repoRoot, "target"), { recursive: true });
  const fixtureDir = fs.mkdtempSync(path.join(repoRoot, "target", "lint-ignore-fixture-"));
  const outputDir = fs.mkdtempSync(path.join(repoRoot, "target", "lint-ignore-report-"));
  try {
    fs.mkdirSync(path.join(fixtureDir, "generated"));
    fs.writeFileSync(path.join(fixtureDir, ".gitignore"), "generated/\n");
    fs.writeFileSync(
      path.join(fixtureDir, "App.vue"),
      ["<template>", "  <div />", "</template>", ""].join("\n"),
    );
    fs.writeFileSync(
      path.join(fixtureDir, "generated", "Ignored.vue"),
      ["<template>", "  <section />", "</template>", ""].join("\n"),
    );
    const fakeVize = path.join(fixtureDir, "fake-vize.mjs");
    fs.writeFileSync(
      fakeVize,
      [
        "#!/usr/bin/env node",
        "if (process.argv[2] !== 'lint') process.exit(2);",
        'process.stdout.write(JSON.stringify([{ file: "App.vue", messages: [] }]));',
        "",
      ].join("\n"),
    );
    fs.chmodSync(fakeVize, 0o755);
    const registryPath = path.join(fixtureDir, "registry.json");
    fs.writeFileSync(
      registryPath,
      JSON.stringify({
        projects: [
          {
            id: "lint-ignore-fixture",
            revision: "0".repeat(40),
            fixturePath: path.relative(repoRoot, fixtureDir),
            vueGlobs: ["**/*.vue"],
            coverage: ["linter"],
          },
        ],
      }),
    );

    const result = spawnSync(
      "rust-script",
      [
        "tools/commands/fixtures/lint-divergence-report.rs",
        "--registry",
        registryPath,
        "--output-dir",
        outputDir,
        "--vize-bin",
        fakeVize,
        "--preset",
        "no-rules",
        "--budget-mode",
        "record-only",
        "--timeout-ms",
        "30000",
      ],
      { cwd: repoRoot, encoding: "utf8", env: { ...process.env, LANG: "C", LC_ALL: "C" } },
    );

    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    const artifact = JSON.parse(
      fs.readFileSync(path.join(outputDir, "lint-ignore-fixture-lint-divergence.json"), "utf8"),
    );
    assert.equal(artifact.files.comparedCount, 1);
  } finally {
    fs.rmSync(fixtureDir, { recursive: true, force: true });
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
});
