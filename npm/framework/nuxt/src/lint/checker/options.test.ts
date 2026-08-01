import assert from "node:assert/strict";
import test from "node:test";

import { resolveNuxtLintCheckerOptions } from "./options.ts";

const project = {
  buildDir: "/project/.nuxt",
  srcDir: "/project/app",
};

const defaults = {
  cache: true,
  emitError: true,
  emitWarning: true,
  exclude: ["**/node_modules/**", "/project/.nuxt"],
  fix: false,
  formatter: "stylish",
  include: ["/project/app/**/*.{js,jsx,ts,tsx,vue}"],
  lintOnStart: true,
};

void test("checker is opt-in and true resolves the full upstream defaults", () => {
  assert.equal(resolveNuxtLintCheckerOptions(undefined, project), false);
  assert.equal(resolveNuxtLintCheckerOptions(false, project), false);
  assert.deepEqual(resolveNuxtLintCheckerOptions(true, project), defaults);
  assert.deepEqual(resolveNuxtLintCheckerOptions({}, project), defaults);
});

void test("checker preserves every explicit portable option", () => {
  assert.deepEqual(
    resolveNuxtLintCheckerOptions(
      {
        cache: false,
        emitError: false,
        emitWarning: false,
        exclude: ["generated/**", "vendor/**"],
        fix: true,
        formatter: "json",
        include: ["src/**/*.vue", "server/**/*.ts"],
        lintOnStart: false,
      },
      project,
    ),
    {
      cache: false,
      emitError: false,
      emitWarning: false,
      exclude: ["generated/**", "vendor/**"],
      fix: true,
      formatter: "json",
      include: ["src/**/*.vue", "server/**/*.ts"],
      lintOnStart: false,
    },
  );
});

void test("resolved arrays never alias caller input", () => {
  const include = ["src/**/*.vue"];
  const exclude = ["build/**"];
  const resolved = resolveNuxtLintCheckerOptions({ include, exclude }, project);
  assert.notEqual(resolved, false);
  include.push("late/**/*.ts");
  exclude.push("late/**");
  assert.deepEqual(resolved.include, ["src/**/*.vue"]);
  assert.deepEqual(resolved.exclude, ["build/**"]);
});
