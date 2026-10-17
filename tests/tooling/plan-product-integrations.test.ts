import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, mkdirSync, renameSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import {
  planProductIntegrations,
  productIntegrationPlanForEvent,
} from "../../tools/support/compat/github/plan-product-integrations.ts";
import { readRepoFile } from "./support/github-workflows.ts";

test("docs and unrelated tooling do not rebuild browser integrations", () => {
  assert.deepEqual(
    planProductIntegrations(["docs/content/guide.md", "tools/commands/release/main.ts"]),
    { nuxt: false, "musea-native": false, "musea-browser": false },
  );
});

test("Nuxt checks its source adapters while its native dependency remains pinned", () => {
  for (const file of [
    "npm/framework/nuxt/src/runtime/server/dev-stylesheet-links.ts",
    "npm/builder/vite/src/index.ts",
    "npm/builder/unplugin/src/index.ts",
    "npm/builder/shared/source-map.ts",
    "npm/cli/src/config/vite-runtime.mjs",
    "npm/cli/src/types/generated.ts",
    "crates/vize_carton/src/config/loader/vite-runtime.mjs",
    "tests/_fixtures/_projects/nuxt-critical-css-build/package-lock.json",
    "tools/support/compat/nuxt/dev-ssr-style.mjs",
  ]) {
    assert.equal(planProductIntegrations([file]).nuxt, true, file);
  }
  const native = planProductIntegrations(["crates/vize_vitrine/src/lib.rs"]);
  assert.equal(native.nuxt, false);
  assert.equal(native["musea-native"], true);
  assert.equal(native["musea-browser"], false);
});

test("Musea source and fixture edits retain native and Chromium assertions", () => {
  for (const file of [
    "npm/builder/vite-musea/src/art-component.ts",
    "tests/_fixtures/differential/musea/component.vue",
    "examples/vite-musea/musea.preview.ts",
    "tools/support/release/public_acceptance/gallery.ts",
  ]) {
    assert.deepEqual(planProductIntegrations([file]), {
      nuxt: false,
      "musea-native": true,
      "musea-browser": true,
    });
  }
  assert.equal(
    planProductIntegrations(["npm/builder/vite-musea-neighbor/src/a.ts"])["musea-native"],
    false,
  );
});

test("Musea browser observation helpers keep their actual consumer selected", () => {
  assert.deepEqual(
    planProductIntegrations(["tools/support/release/public_acceptance/browser_observation.ts"]),
    { nuxt: false, "musea-native": false, "musea-browser": true },
  );
  assert.equal(planProductIntegrations(["tools/support/command-runtime.ts"])["musea-native"], true);
});

test("Musea native CLI builds install their transitive rule generator runner", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "musea-browser.yml")) as {
    jobs: Record<string, { steps?: Array<{ uses?: string; run?: string }> }>;
  };
  const native = workflow.jobs["musea-native-inline"].steps!;
  const runner = native.findIndex((step) => step.uses === "./.github/actions/setup-rust-script");
  const cliBuild = native.findIndex((step) => step.run?.includes("--filter './npm/cli' build"));
  assert.ok(runner >= 0 && cliBuild > runner);
  for (const [name, job] of Object.entries(workflow.jobs)) {
    if (name === "musea-native-inline") continue;
    assert.ok(!job.steps?.some((step) => step.uses === "./.github/actions/setup-rust-script"));
  }
  assert.deepEqual(planProductIntegrations([".github/actions/setup-rust-script/action.yml"]), {
    nuxt: false,
    "musea-native": true,
    "musea-browser": false,
  });
});

test("shared dependency and planner changes exercise every integration", () => {
  for (const file of [
    "pnpm-lock.yaml",
    "package.json",
    "tools/support/compat/github/plan-product-integrations.ts",
    ".github/actions/cache-vp-bootstrap/action.yml",
    ".github/actions/cache-pnpm-verification/action.yml",
    ".github/actions/cache-pnpm-verification/cache-key.ts",
    "patches/braces@3.0.3.patch",
  ]) {
    assert.deepEqual(
      planProductIntegrations([file]),
      { nuxt: true, "musea-native": true, "musea-browser": true },
      file,
    );
  }
});

test("invalid paths and incomplete event identities fail instead of skipping checks", () => {
  for (const file of [
    "",
    "../docs/a.md",
    "/docs/a.md",
    "docs\\a.md",
    "docs//a.md",
    "docs/./a.md",
    "docs/a\n.md",
  ]) {
    assert.throws(() => planProductIntegrations([file]), /normalized repository path/);
  }
  for (const event of ["push", "pull_request", "merge_group"]) {
    assert.throws(
      () => productIntegrationPlanForEvent(event, "main", "HEAD"),
      /full base and head/,
    );
    assert.deepEqual(
      Object.values(productIntegrationPlanForEvent(event, "0".repeat(40), "a".repeat(40))),
      [true, true, true],
    );
  }
  assert.throws(() => productIntegrationPlanForEvent("unknown", "", ""), /unsupported/);
  for (const event of ["schedule", "workflow_dispatch"]) {
    assert.deepEqual(Object.values(productIntegrationPlanForEvent(event, "", "")), [
      true,
      true,
      true,
    ]);
  }
});

test("a rename away from a product still checks the deleted product input", () => {
  const cwd = mkdtempSync(path.join(os.tmpdir(), "vize-integration-plan-"));
  const git = (...args: string[]) => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
  try {
    git("init", "--quiet");
    git("config", "user.name", "Integration planner test");
    git("config", "user.email", "test@example.invalid");
    mkdirSync(path.join(cwd, "npm/framework/nuxt"), { recursive: true });
    writeFileSync(path.join(cwd, "npm/framework/nuxt/input.ts"), "export const input = true;\n");
    git("add", ".");
    git("commit", "--quiet", "-m", "fixture");
    const base = git("rev-parse", "HEAD");
    mkdirSync(path.join(cwd, "docs"));
    renameSync(path.join(cwd, "npm/framework/nuxt/input.ts"), path.join(cwd, "docs/input.ts"));
    git("add", ".");
    git("commit", "--quiet", "-m", "move fixture");
    assert.equal(
      productIntegrationPlanForEvent("merge_group", base, git("rev-parse", "HEAD"), cwd).nuxt,
      true,
    );
    assert.throws(() => productIntegrationPlanForEvent("merge_group", "f".repeat(40), base, cwd));
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});
