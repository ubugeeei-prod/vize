import assert from "node:assert/strict";
import test from "node:test";

import { matchesNuxtLintCheckerFile } from "./filter.ts";
import type { ResolvedVizeNuxtLintCheckerOptions } from "./options.ts";

const options: ResolvedVizeNuxtLintCheckerOptions = {
  cache: true,
  emitError: true,
  emitWarning: true,
  exclude: ["**/node_modules/**", "/project/.nuxt", "generated/**"],
  fix: false,
  formatter: "stylish",
  include: ["/project/app/**/*.{js,jsx,ts,tsx,vue}", "server/**/*.ts"],
  lintOnStart: true,
};

void test("checker filtering accepts every included extension and relative glob", () => {
  for (const extension of ["js", "jsx", "ts", "tsx", "vue"]) {
    assert.equal(
      matchesNuxtLintCheckerFile(`/project/app/pages/index.${extension}`, "/project", options),
      true,
    );
  }
  assert.equal(matchesNuxtLintCheckerFile("/project/server/api.ts", "/project", options), true);
});

void test("checker filtering gives excludes precedence, including directory paths", () => {
  assert.equal(
    matchesNuxtLintCheckerFile("/project/app/node_modules/pkg/index.ts", "/project", options),
    false,
  );
  assert.equal(
    matchesNuxtLintCheckerFile("/project/.nuxt/generated/client.ts", "/project", options),
    false,
  );
  assert.equal(
    matchesNuxtLintCheckerFile("/project/generated/types.ts", "/project", options),
    false,
  );
  assert.equal(matchesNuxtLintCheckerFile("/project/docs/index.ts", "/project", options), false);
});

void test("checker filtering normalizes Windows separators deterministically", () => {
  assert.equal(
    matchesNuxtLintCheckerFile(String.raw`C:\project\app\pages\index.vue`, String.raw`C:\project`, {
      ...options,
      exclude: [String.raw`C:\project\.nuxt`],
      include: [String.raw`C:\project\app\**\*.{js,jsx,ts,tsx,vue}`],
    }),
    true,
  );
});

void test("relative include globs never escape the project root", () => {
  const config = { ...options, include: ["**/*.vue"] };
  assert.equal(matchesNuxtLintCheckerFile("/project/App.vue", "/project", config), true);
  assert.equal(matchesNuxtLintCheckerFile("/outside/App.vue", "/project", config), false);
});
