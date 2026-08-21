import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  FRONTEND_PHPCON_E2E_API_BASE,
  FRONTEND_PHPCON_STAFF_ROUTE_RELATIVE_PATH,
  npmxGeneratorTaskArgs,
  patchNuxtPrerenderForE2E,
  readDotenvValue,
} from "../_helpers/app-fixture-runtime.ts";
import { elkApp, frontendPhpconApp, npmxApp } from "../_helpers/apps.ts";
import {
  resolveVizeE2EFastNpmMetaVersion,
  resolveVizeE2ENpmRegistryCachedResponse,
} from "../_fixtures/npmx-e2e-registry-fixtures.ts";
import {
  fullAppE2eRows,
  planAppE2eRows,
  readinessRows,
  validateAppE2eRows,
} from "../../tools/github/app-e2e-plan.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const readinessFixturePaths = ["elk", "misskey", "npmx.dev", "nuxt-ui", "reka-ui"].map(
  (id) => `tests/_fixtures/_git/${id}`,
);
const checkFixturePaths = [
  "ant-design-vue",
  "directus",
  "element-plus",
  "elk",
  "frontend-phpcon-do-website",
  "hoppscotch",
  "misskey",
  "naive-ui",
  "npmx.dev",
  "nuxt-ui",
  "primevue",
  "reka-ui",
  "voicevox",
  "vue-vben-admin",
  "vuefes-2025",
  "vuetify",
].map((id) => `tests/_fixtures/_git/${id}`);

test("full and readiness plans preserve every isolated execution row", () => {
  assert.deepEqual(
    Object.fromEntries(
      ["dev", "vrt", "preview", "check", "lint", "build"].map((suite) => [
        suite,
        planAppE2eRows("full", suite).map((row) => row.shard),
      ]),
    ),
    {
      dev: ["elk", "misskey", "npmx", "nuxt-ui", "vuefes"],
      vrt: ["elk", "frontend-phpcon", "misskey", "npmx", "vuefes"],
      preview: ["elk", "misskey", "npmx", "vuefes"],
      check: ["all"],
      lint: ["all"],
      build: ["all"],
    },
  );
  assert.equal(planAppE2eRows("full", "all").length, 17);
  assert.deepEqual(
    fullAppE2eRows.filter((row) => row.needsPlaywright).map((row) => `${row.suite}:${row.shard}`),
    [
      "dev:elk",
      "dev:misskey",
      "dev:npmx",
      "dev:nuxt-ui",
      "dev:vuefes",
      "vrt:elk",
      "vrt:frontend-phpcon",
      "vrt:misskey",
      "vrt:npmx",
      "vrt:vuefes",
    ],
  );
  assert.deepEqual(
    planAppE2eRows("readiness").map((row) => row.shard),
    ["check", "check-vuefes", "lint", "build", "dev-misskey", "dev-nuxt-ui"],
  );
  assert.equal(readinessRows.length, 6);
  assert.deepEqual(
    readinessRows.filter((row) => row.needsPlaywright).map((row) => row.shard),
    ["dev-misskey", "dev-nuxt-ui"],
  );
  assert.equal(
    fullAppE2eRows.find(
      (row) => row.profile === "full" && row.suite === "check" && row.shard === "all",
    )?.timeout,
    "75m",
  );
  assert.equal(
    readinessRows.find((row) => row.shard === "check")?.timeout,
    "8m",
    "cold readiness check rows must finish on Blacksmith without timing out",
  );
  assert.equal(readinessRows.find((row) => row.shard === "dev-misskey")?.timeout, "8m");
  assert.equal(
    readinessRows.find((row) => row.shard === "dev-nuxt-ui")?.timeout,
    "8m",
    "Nuxt UI dev readiness must fit the same Blacksmith budget as the other dev readiness row",
  );
  assert.deepEqual(
    fullAppE2eRows.filter((row) => row.suite === "vrt").map((row) => row.timeout),
    ["15m", "15m", "15m", "15m", "15m"],
    "full VRT rows should stay inside the Blacksmith runner budget",
  );
  assert.equal(
    readinessRows.find((row) => row.shard === "lint")?.timeout,
    "5m",
    "updated fixture setup must fit inside the readiness lint budget",
  );
});

test("upstream app fixtures keep deterministic CI setup", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-dotenv-"));
  const dotenvPath = path.join(tempDir, ".env.mock");
  fs.writeFileSync(dotenvPath, 'MOCK_USER=\'{"id":"e2e"}\'\nPLAIN=value\n');

  assert.equal(readDotenvValue(dotenvPath, "MOCK_USER"), '{"id":"e2e"}');
  assert.equal(readDotenvValue(dotenvPath, "PLAIN"), "value");
  assert.equal(readDotenvValue(path.join(tempDir, "missing.env"), "MOCK_USER"), undefined);
  assert.equal(elkApp.env?.CONTEXT, "dev");
  assert.equal(typeof elkApp.env?.MOCK_USER, "string");
  assert.match(elkApp.args.join(" "), /pnpm@10 exec nuxt dev/);
  const nuxtConfigPath = path.join(tempDir, "nuxt.config.ts");
  fs.writeFileSync(nuxtConfigPath, "'/': { prerender: true },\ncrawlLinks: true,\n");
  patchNuxtPrerenderForE2E(nuxtConfigPath);
  assert.equal(
    fs.readFileSync(nuxtConfigPath, "utf-8"),
    "'/': { prerender: false },\ncrawlLinks: false,\n",
  );

  assert.deepEqual(npmxGeneratorTaskArgs("generate:lexicons"), [
    "-y",
    "pnpm@10",
    "exec",
    "vp",
    "run",
    "--no-cache",
    "generate:lexicons",
  ]);
  assert.deepEqual(npmxGeneratorTaskArgs("generate:sprite"), [
    "-y",
    "pnpm@10",
    "exec",
    "vp",
    "run",
    "--no-cache",
    "generate:sprite",
  ]);
  assert.equal(npmxApp.env?.NUXT_TEST_FIXTURES, "true");
  assert.equal(npmxApp.env?.VIZE_E2E_DISABLE_LUNARIA, "1");
  const manifest = resolveVizeE2ENpmRegistryCachedResponse<{ dist: { unpackedSize: number } }>(
    "vue/3.5.29",
  );
  assert.equal(manifest.handled, true);
  if (!manifest.handled) throw new Error("expected vue package manifest fixture");
  assert.equal(manifest.data.data.dist.unpackedSize, 2_600_000);
  assert.deepEqual(resolveVizeE2EFastNpmMetaVersion("https://npm.antfu.dev/vue"), {
    handled: true,
    data: "3.5.29",
  });

  assert.equal(frontendPhpconApp.env?.NUXT_PUBLIC_API_BASE, FRONTEND_PHPCON_E2E_API_BASE);
  assert.equal(
    FRONTEND_PHPCON_STAFF_ROUTE_RELATIVE_PATH,
    path.join("server", "routes", "__vize_e2e", "api", "staff.get.ts"),
  );
});

test("planned tasks, fixtures, and mutable identities are exact and unique", () => {
  const scripts = (
    JSON.parse(fs.readFileSync(path.join(root, "tests/package.json"), "utf8")) as {
      scripts: Record<string, string>;
    }
  ).scripts;
  const gitmodulePaths = new Set(
    execFileSync("git", ["config", "-f", ".gitmodules", "--get-regexp", "path"], {
      cwd: root,
      encoding: "utf8",
    })
      .trim()
      .split("\n")
      .map((line) => line.split(/\s+/).at(-1)),
  );
  const rows = [...fullAppE2eRows, ...readinessRows];
  assert.deepEqual(
    rows.map((row) => [row.profile, row.suite, row.shard, row.task, row.fixtures]),
    [
      ["full", "dev", "elk", "test:dev:elk", ["tests/_fixtures/_git/elk"]],
      ["full", "dev", "misskey", "test:dev:misskey", ["tests/_fixtures/_git/misskey"]],
      ["full", "dev", "npmx", "test:dev:npmx", ["tests/_fixtures/_git/npmx.dev"]],
      ["full", "dev", "nuxt-ui", "test:dev:nuxt-ui", ["tests/_fixtures/_git/nuxt-ui"]],
      ["full", "dev", "vuefes", "test:dev:vuefes", ["tests/_fixtures/_git/vuefes-2025"]],
      ["full", "vrt", "elk", "test:vrt:elk", ["tests/_fixtures/_git/elk"]],
      [
        "full",
        "vrt",
        "frontend-phpcon",
        "test:vrt:frontend-phpcon",
        ["tests/_fixtures/_git/frontend-phpcon-do-website"],
      ],
      ["full", "vrt", "misskey", "test:vrt:misskey", ["tests/_fixtures/_git/misskey"]],
      ["full", "vrt", "npmx", "test:vrt:npmx", ["tests/_fixtures/_git/npmx.dev"]],
      ["full", "vrt", "vuefes", "test:vrt:vuefes", ["tests/_fixtures/_git/vuefes-2025"]],
      ["full", "preview", "elk", "test:preview:elk", ["tests/_fixtures/_git/elk"]],
      ["full", "preview", "misskey", "test:preview:misskey", ["tests/_fixtures/_git/misskey"]],
      ["full", "preview", "npmx", "test:preview:npmx", ["tests/_fixtures/_git/npmx.dev"]],
      ["full", "preview", "vuefes", "test:preview:vuefes", ["tests/_fixtures/_git/vuefes-2025"]],
      [
        "full",
        "build",
        "all",
        "test:build",
        ["elk", "misskey", "npmx.dev", "vuefes-2025"].map((id) => `tests/_fixtures/_git/${id}`),
      ],
      ["full", "check", "all", "test:check", checkFixturePaths],
      [
        "full",
        "lint",
        "all",
        "test:lint",
        checkFixturePaths.filter((path) => !path.endsWith("/frontend-phpcon-do-website")),
      ],
      ["readiness", "readiness", "check", "test:readiness:check", readinessFixturePaths],
      [
        "readiness",
        "readiness",
        "check-vuefes",
        "test:readiness:check:vuefes",
        ["tests/_fixtures/_git/vuefes-2025"],
      ],
      ["readiness", "readiness", "lint", "test:readiness:lint", readinessFixturePaths],
      ["readiness", "readiness", "build", "test:readiness:build", ["tests/_fixtures/_git/elk"]],
      [
        "readiness",
        "readiness",
        "dev-misskey",
        "test:readiness:dev:misskey",
        ["tests/_fixtures/_git/misskey"],
      ],
      [
        "readiness",
        "readiness",
        "dev-nuxt-ui",
        "test:readiness:dev:nuxt-ui",
        ["tests/_fixtures/_git/nuxt-ui"],
      ],
    ],
  );
  for (const row of rows) {
    assert.ok(scripts[row.task], `missing package task ${row.task}`);
    for (const fixture of row.fixtures) assert.ok(gitmodulePaths.has(fixture), fixture);
  }
  for (const row of fullAppE2eRows.filter(
    (current) => current.suite === "dev" || current.suite === "vrt",
  )) {
    assert.equal(
      scripts[row.task],
      `playwright test --config app/playwright${row.suite === "vrt" ? ".vrt" : ""}.config.ts app/${row.suite}/${row.shard}.spec.ts`,
    );
  }
  for (const row of fullAppE2eRows.filter((current) => current.suite === "preview")) {
    assert.equal(scripts[row.task], `RUN_BUILD_TESTS=1 node app/preview/${row.shard}.ts`);
  }
  assert.equal(
    scripts["test:readiness:dev:misskey"],
    "playwright test --config app/playwright.config.ts app/dev/misskey.spec.ts",
  );
  assert.equal(
    scripts["test:readiness:dev:nuxt-ui"],
    "playwright test --config app/playwright.config.ts --retries=0 app/dev/nuxt-ui.spec.ts",
  );
  assert.equal(
    scripts["test:readiness:check:vuefes"],
    "node --test --test-concurrency=1 snapshots/check/vuefes.ts",
    "the authored-source fixture must remain an isolated PR readiness row",
  );
  for (const field of ["task", "cacheKey", "worktreeId", "artifactStem"] as const) {
    assert.equal(new Set(rows.map((row) => row[field])).size, rows.length, `${field} collision`);
  }
  for (const task of [
    "test:check",
    "test:lint",
    "test:build",
    "test:readiness:check",
    "test:readiness:check:vuefes",
    "test:readiness:lint",
    "test:readiness:build",
  ]) {
    assert.match(scripts[task]!, /--test-concurrency=1/, `${task} must stay serial`);
  }
  for (const config of ["playwright.config.ts", "playwright.vrt.config.ts"]) {
    const source = fs.readFileSync(path.join(root, "tests/app", config), "utf8");
    assert.match(source, /fullyParallel:\s*false/);
    assert.match(source, /workers:\s*1/);
  }
});

test("plan validation rejects drift instead of silently dropping coverage", () => {
  const valid = structuredClone([...fullAppE2eRows, ...readinessRows]);
  assert.doesNotThrow(() => validateAppE2eRows(valid));
  for (const current of valid) {
    assert.equal(
      current.runner,
      "blacksmith-32vcpu-ubuntu-2404",
      `${current.profile}:${current.shard} runner`,
    );
  }
  for (const [name, mutate, message] of [
    ["empty", (rows: typeof valid) => rows.splice(0), /must not be empty/],
    [
      "duplicate",
      (rows: typeof valid) => rows.push(structuredClone(rows[0]!)),
      /Duplicate identity/,
    ],
    ["unknown profile", (rows: typeof valid) => (rows[0]!.profile = "other"), /Unknown.*profile/],
    ["empty fixtures", (rows: typeof valid) => (rows[0]!.fixtures = []), /at least one fixture/],
    ["bad fixture", (rows: typeof valid) => (rows[0]!.fixtures = ["../escape"]), /invalid fixture/],
    ["bad task", (rows: typeof valid) => (rows[0]!.task = "shell:$(bad)"), /Invalid task/],
    [
      "browser drift",
      (rows: typeof valid) => (rows[0]!.needsPlaywright = false),
      /Playwright requirement drifted/,
    ],
    [
      "cache collision",
      (rows: typeof valid) => (rows[1]!.cacheKey = rows[0]!.cacheKey),
      /identity drifted/,
    ],
    ["runner drift", (rows: typeof valid) => (rows[0]!.runner = "ubuntu-24.04"), /runner drifted/],
  ] as const) {
    const rows = structuredClone(valid);
    mutate(rows);
    assert.throws(() => validateAppE2eRows(rows), message, name);
  }
  assert.throws(() => planAppE2eRows("full", "unknown"), /Unknown full App E2E suite/);
});
