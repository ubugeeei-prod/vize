import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  createScopedMirror,
  readScopedConfig,
  validateScopedConfig,
  validateSelectionPaths,
} from "./scoped-config.ts";
import { resolveWorkaroundSource } from "../workaround.ts";
import { withoutLintTargets } from "./args.ts";
import { collectVueLikeFilesFromTargets } from "./files.ts";

void test("target projection preserves Vue-shaped option values and the literal separator", () => {
  assert.deepEqual(
    withoutLintTargets([
      "-c",
      "App.vue",
      "--ignore-pattern",
      "[slug].vue",
      "--format=json",
      "app",
      "--",
      "-strange.vue",
      "dist",
    ]),
    ["-c", "App.vue", "--ignore-pattern", "[slug].vue", "--format=json", "--"],
  );
});

void test("all-candidate protocol refuses ambiguous non-Vue names even with a slash-path twin", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-scoped-paths-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  fs.mkdirSync(path.join(root, "back"));
  for (const name of [
    "back\\slash.js",
    "back/slash.js",
    "line\nfeed.ts",
    "carriage\rreturn.ts",
    "[slug].vue",
    "Space name.vue",
  ])
    fs.writeFileSync(path.join(root, name), "");
  const candidates = new Set<string>();
  assert.deepEqual(
    collectVueLikeFilesFromTargets(root, ["."], (file) => candidates.add(file)),
    [path.join(root, "Space name.vue"), path.join(root, "[slug].vue")],
  );
  assert.deepEqual(
    [...candidates].sort(),
    [
      root,
      path.join(root, "Space name.vue"),
      path.join(root, "[slug].vue"),
      path.join(root, "back"),
      path.join(root, "back/slash.js"),
      path.join(root, "back\\slash.js"),
      path.join(root, "carriage\rreturn.ts"),
      path.join(root, "line\nfeed.ts"),
    ].sort(),
  );
  assert.throws(() => validateSelectionPaths(root, candidates), {
    message: "Scoped Vue transport cannot preserve filenames containing CR, LF or backslash.",
  });
  for (const name of ["back\\slash.js", "line\nfeed.ts", "carriage\rreturn.ts"])
    fs.unlinkSync(path.join(root, name));
  candidates.clear();
  collectVueLikeFilesFromTargets(root, ["."], (file) => candidates.add(file));
  validateSelectionPaths(root, candidates);
  fs.writeFileSync(path.join(root, "oxlint-suppressions.json"), "{}\n");
  assert.throws(() => validateSelectionPaths(root, candidates), {
    message: "Scoped Vue transport cannot preserve path-sensitive suppression state.",
  });
});

void test("path-sensitive import/project and non-POSIX envelopes refuse before transport", () => {
  for (const [value, args] of [
    [{ plugins: ["import"] }, []],
    [{ overrides: [{ files: ["**/*.vue"], plugins: ["import"] }] }, []],
    [{}, ["--import-plugin"]],
    [{}, ["--tsconfig", "tsconfig.json"]],
  ] as const) {
    assert.throws(
      () => validateScopedConfig({ file: "/project/config.json", bytes: "", value }, args),
      {
        message: "Scoped Vue transport cannot preserve Oxlint's import/project resolution.",
      },
    );
  }
  assert.throws(
    () => validateScopedConfig({ file: "C:/project/config.json", bytes: "", value: {} }, []),
    {
      message: "Scoped Vue transport cannot preserve non-POSIX filesystem roots.",
    },
  );
});

const source = '<script setup>const title = "About";</script>\n<template>{{ title }}</template>\n';

void test("scoped mirror keeps original whole config/source and ordered namespaces", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-scoped-config-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const file = path.join(root, "app/pages/About.vue");
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, source);
  const original = {
    jsPlugins: ["./plugin.mjs"],
    settings: { custom: { preserved: [1, "two"] } },
    ignorePatterns: ["**/node_modules", "**/dist", "**/oxlint-vize-*/**"],
    overrides: [
      { files: ["**/*.vue"], rules: { example: "warn" } },
      {
        files: ["app/pages/**/*.vue"],
        excludeFiles: ["app/pages/generated/**"],
        rules: { example: "off" },
      },
    ],
  };
  const configFile = path.join(root, "config.json");
  const bytes = JSON.stringify(original, null, 2) + "\n";
  fs.writeFileSync(configFile, bytes);
  const config = readScopedConfig(root, ["--config", configFile]);
  assert.ok(config);
  const mirror = createScopedMirror(root, config, [file]);
  try {
    const copy = mirror.originalsToCopies.get(file);
    assert.ok(copy);
    assert.deepEqual(resolveWorkaroundSource(fs.readFileSync(copy, "utf8"), copy), {
      filename: file,
      source,
      usesOriginalLocations: true,
    });
    const prefix = copy.slice(0, -"app/pages/About.vue".length);
    assert.deepEqual(JSON.parse(fs.readFileSync(mirror.sibling, "utf8")), {
      ...original,
      overrides: [
        { ...original.overrides[0], excludeFiles: [copy] },
        { ...original.overrides[0], files: [`${prefix}**/*.vue`], excludeFiles: [] },
        { ...original.overrides[1], excludeFiles: ["app/pages/generated/**", copy] },
        {
          ...original.overrides[1],
          files: [`${prefix}app/pages/**/*.vue`],
          excludeFiles: [`${prefix}app/pages/generated/**`],
        },
      ],
    });
    assert.equal(path.dirname(mirror.sibling), root);
    assert.equal(fs.readFileSync(configFile, "utf8"), bytes);
    assert.equal(fs.readFileSync(file, "utf8"), source);
  } finally {
    mirror.cleanup();
  }
  assert.equal(fs.existsSync(mirror.sibling), false);
  assert.equal(fs.existsSync(mirror.originalsToCopies.get(file)!), false);
  assert.deepEqual(fs.readdirSync(root).sort(), ["app", "config.json"]);
});

void test("outside wildcard refusal cleans preparation and retains every authored byte", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-scoped-refusal-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const configDir = path.join(root, "project");
  fs.mkdirSync(configDir);
  const file = path.join(root, "Outside.vue");
  fs.writeFileSync(file, source);
  const configFile = path.join(configDir, "config.json");
  const bytes = '{"overrides":[{"files":["**/app/**/*.vue"],"rules":{"example":"off"}}]}\n';
  fs.writeFileSync(configFile, bytes);
  const config = readScopedConfig(configDir, ["-c", configFile]);
  assert.ok(config);
  assert.throws(() => createScopedMirror(configDir, config, [file]), {
    message: "Scoped Vue transport cannot preserve outside-root override pattern: **/app/**/*.vue",
  });
  assert.deepEqual(fs.readdirSync(configDir), ["config.json"]);
  assert.equal(fs.readFileSync(configFile, "utf8"), bytes);
  assert.equal(fs.readFileSync(file, "utf8"), source);
});
