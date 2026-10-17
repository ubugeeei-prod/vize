import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test, type TestContext } from "node:test";
import { parse } from "yaml";
import {
  verificationCacheIdentity,
  verificationCacheWrite,
} from "../../.github/actions/cache-pnpm-verification/cache-key.ts";
import { readRepoFile } from "./support/github-workflows.ts";

const config = {
  minimumReleaseAge: 1440,
  minimumReleaseAgeExclude: ["trusted-package"],
  trustPolicy: "no-downgrade",
  verifyStoreIntegrity: true,
  registry: "https://registry.npmjs.org/",
  registries: { "https://registry.npmjs.org/": { scopes: ["@"], prefix: "npmjs" } },
};

function fixture(t: TestContext): string {
  const root = mkdtempSync(join(tmpdir(), "vize-pnpm-verification-key-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  writeFileSync(join(root, "package.json"), '{"packageManager":"pnpm@12.1.0"}\n');
  writeFileSync(join(root, "pnpm-workspace.yaml"), "packages: [npm/*]\n");
  writeFileSync(join(root, "pnpm-lock.yaml"), "lockfileVersion: '9.0'\n");
  return root;
}

test("verification cache binds exact lock/config/version and active policy/registry settings", (t) => {
  const root = fixture(t);
  const cache = join(root, "cache");
  const identity = verificationCacheIdentity(root, "12.1.0", cache, config);
  assert.ok(identity);
  assert.match(identity.key, /^pnpm-verification-v1-12\.1\.0-[0-9a-f]{64}$/);
  assert.equal(identity.path, join(cache, "lockfile-verified.jsonl"));
  for (const change of [
    { minimumReleaseAge: 2880 },
    { minimumReleaseAgeExclude: [] },
    { minimumReleaseAgeIgnoreMissingTime: true },
    { trustPolicy: "off" },
    { trustPolicyExclude: ["another-package"] },
    { trustPolicyIgnoreAfter: 60 },
    { registry: "https://registry.example/" },
    { "@private:registry": "https://registry.example/" },
    { registries: { "https://registry.example/": { prefix: "npmjs", scopes: ["@"] } } },
    { verifyStoreIntegrity: false },
  ]) {
    const changed = verificationCacheIdentity(root, "12.1.0", cache, { ...config, ...change });
    assert.ok(changed);
    assert.notEqual(changed.key, identity.key, JSON.stringify(change));
  }
  for (const file of ["package.json", "pnpm-workspace.yaml", "pnpm-lock.yaml"]) {
    const path = join(root, file);
    const original = readFileSync(path, "utf8");
    writeFileSync(path, `${original}\n`);
    assert.notEqual(verificationCacheIdentity(root, "12.1.0", cache, config)?.key, identity.key);
    writeFileSync(path, original);
  }
  writeFileSync(join(root, "package.json"), '{"packageManager":"pnpm@12.2.0"}\n');
  assert.equal(verificationCacheIdentity(root, "12.1.0", cache, config), undefined);
  assert.notEqual(verificationCacheIdentity(root, "12.2.0", cache, config)?.key, identity.key);
});

test("paths, config key ordering and credentials do not leak into verification identity", (t) => {
  const first = fixture(t);
  const second = fixture(t);
  const identity = verificationCacheIdentity(first, "12.1.0", join(first, "cache"), config);
  const reordered = Object.fromEntries(Object.entries(config).reverse());
  for (const token of ["secret-one", "secret-two"]) {
    const changed = verificationCacheIdentity(second, "12.1.0", join(second, "cache"), {
      ...reordered,
      "//registry.npmjs.org/:_authToken": token,
      userAgent: "a different runner",
      cacheDir: join(second, "cache"),
    });
    assert.equal(changed?.key, identity?.key);
    assert.equal(changed?.path, join(second, "cache", "lockfile-verified.jsonl"));
  }
});

test("unknown or dynamic configuration keeps ordinary pnpm verification uncached", (t) => {
  const root = fixture(t);
  const cache = join(root, "cache");
  for (const change of [
    { trustLockfile: true },
    { newPolicy: true },
    { pnpmfile: "hooks.cjs" },
    { globalPnpmfile: "/tmp/hooks.cjs" },
    { configDependencies: { policy: "1.0.0" } },
    { lockfileDir: "elsewhere" },
    { gitBranchLockfile: true },
    { minimumReleaseAge: "1440" },
    { registry: "https://user:secret@registry.example/" },
    { registry: "https://registry.example/?token=secret" },
    { registries: { "https://registry.npmjs.org/": { scopes: ["@"], _authToken: "secret" } } },
  ])
    assert.equal(
      verificationCacheIdentity(root, "12.1.0", cache, { ...config, ...change }),
      undefined,
    );
  for (const path of ["relative", `${cache}\nother`, `${cache}/*`])
    assert.equal(verificationCacheIdentity(root, "12.1.0", path, config), undefined);
  for (const key of ["NPM_CONFIG_WORKSPACE_DIR", "npm_config_workspace_dir"])
    assert.equal(
      verificationCacheIdentity(root, "12.1.0", cache, config, { [key]: "/another/workspace" }),
      undefined,
    );
  for (const file of [".pnpmfile.cjs", ".pnpmfile.mjs"]) {
    writeFileSync(join(root, file), "module.exports = {};\n");
    assert.equal(verificationCacheIdentity(root, "12.1.0", cache, config), undefined);
    rmSync(join(root, file));
  }
});

test("only a requested exact-checkout trusted default-branch job writes verification", () => {
  const sha = "a".repeat(40);
  for (const event of ["push", "schedule", "workflow_dispatch"])
    assert.equal(verificationCacheWrite(true, event, "refs/heads/main", "main", sha, sha), true);
  for (const event of ["pull_request", "pull_request_target", "merge_group", "workflow_run"])
    assert.equal(verificationCacheWrite(true, event, "refs/heads/main", "main", sha, sha), false);
  assert.equal(verificationCacheWrite(false, "push", "refs/heads/main", "main", sha, sha), false);
  assert.equal(verificationCacheWrite(true, "push", "refs/heads/pr", "main", sha, sha), false);
  assert.equal(
    verificationCacheWrite(true, "push", "refs/heads/main", "main", sha, "b".repeat(40)),
    false,
  );
});

type Step = { uses?: string; run?: string; if?: string; with?: Record<string, unknown> };
const yaml = (file: string) => parse(readRepoFile(...file.split("/")));

test("cache integration preserves install commands and has exactly one trusted writer", () => {
  const action = yaml(".github/actions/cache-pnpm-verification/action.yml");
  const caches = (action.runs.steps as Step[]).filter((step) =>
    step.uses?.startsWith("actions/cache"),
  );
  assert.deepEqual(
    caches.map((step) => ({ uses: step.uses, if: step.if, with: step.with })),
    [
      {
        uses: "actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae",
        if: "${{ steps.key.outputs.key != '' && steps.key.outputs.save == 'true' }}",
        with: { path: "${{ steps.key.outputs.path }}", key: "${{ steps.key.outputs.key }}" },
      },
      {
        uses: "actions/cache/restore@27d5ce7f107fe9357f9df03efb73ab90386fccae",
        if: "${{ steps.key.outputs.key != '' && steps.key.outputs.save != 'true' }}",
        with: { path: "${{ steps.key.outputs.path }}", key: "${{ steps.key.outputs.key }}" },
      },
    ],
  );
  const cacheAction = "./.github/actions/cache-pnpm-verification";
  const rust = yaml(".github/workflows/pr-rust-checks.yml").jobs;
  const seed = yaml(".github/workflows/rust-pr-target-seed.yml").jobs;
  const js = yaml(".github/workflows/pr-js-checks.yml").jobs;
  const full = yaml(".github/actions/setup-js-package-tests/action.yml").runs.steps as Step[];
  const installs: Array<[Step[], string, string | undefined]> = [
    [rust["pr-rust-shard"].steps, "vp install", undefined],
    [seed["rust-differential-target-seed"].steps, "vp install", "true"],
    [full, "vp install --frozen-lockfile --prefer-offline", undefined],
    [
      js["pr-js-package-tests"].steps,
      'node tools/support/compat/github/install-js-checks.ts "$VIZE_JS_PACKAGE_GROUP"',
      undefined,
    ],
    [
      js["pr-oxlint-project-qualification"].steps,
      "node tools/support/compat/github/install-js-checks.ts original-project",
      undefined,
    ],
    [
      js["pr-js-package-conformance"].steps,
      'node tools/support/compat/github/install-js-checks.ts "$CONFORMANCE_GROUP"',
      undefined,
    ],
  ];
  for (const [steps, installCommand, save] of installs) {
    const cacheIndex = steps.findIndex((step) => step.uses === cacheAction);
    const installIndex = steps.findIndex((step) => step.run === installCommand);
    assert.ok(cacheIndex >= 0 && cacheIndex < installIndex);
    assert.equal(steps[cacheIndex].if, steps[installIndex].if);
    assert.equal(steps[cacheIndex].with?.save, save);
    assert.equal(steps.filter((step) => step.run === installCommand).length, 1);
    assert.equal(
      steps.find((step) => step.uses?.startsWith("voidzero-dev/setup-vp@"))?.with?.["run-install"],
      false,
    );
  }
});
