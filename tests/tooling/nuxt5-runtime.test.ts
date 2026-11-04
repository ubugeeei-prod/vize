import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path, { matchesGlob } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

type Step = {
  shell?: string;
  name?: string;
  run?: string;
  uses?: string;
  if?: string;
  with?: Record<string, unknown>;
  "continue-on-error"?: boolean;
};
type Workflow = {
  on: { pull_request: { paths: string[] }; push: { paths: string[] } };
  jobs: Record<
    string,
    { "runs-on": string; "timeout-minutes": number; steps: Step[]; "continue-on-error"?: boolean }
  >;
};
const root = new URL("../../", import.meta.url);
const fixture = new URL("tools/support/compat/nuxt/fixtures/nuxt5-runtime-build/", root);
const workflow = parse(
  fs.readFileSync(new URL(".github/workflows/nuxt3-module-build.yml", root), "utf8"),
) as Workflow;
const job = workflow.jobs["nuxt3-module-build"];
const step = (name: string) => {
  const value = job.steps.find((entry) => entry.name === name);
  assert.ok(value, name);
  return value;
};

void test("Nuxt 5 production fixture retains the reporter's complete three source files", () => {
  const original = {
    "package.json": "5bc71d5ee6e499ecc9a6e7f94b6311dcedf914d02797c08ac2a2be3d9af255db",
    "nuxt.config.ts": "cf2d4a7e3878d9f9459685aae99c0317c51cfb51a55ac699c21d4153df17e133",
    "app/app.vue": "9907bbf1071a1117ecb79ae03090464078685f3dad3ef574d028c4f45fc6291d",
  };
  const provenance = JSON.parse(fs.readFileSync(new URL("provenance.json", fixture), "utf8")) as {
    files: Record<string, string>;
  };
  for (const [file, expected] of Object.entries(original)) {
    assert.equal(
      createHash("sha256")
        .update(fs.readFileSync(new URL(file, fixture)))
        .digest("hex"),
      expected,
    );
    assert.equal(provenance.files[file], expected);
  }
  const lockBytes = fs.readFileSync(new URL("package-lock.json", fixture));
  assert.equal(
    createHash("sha256").update(lockBytes).digest("hex"),
    provenance.files["package-lock.json"],
  );
  const lock = JSON.parse(lockBytes.toString()) as {
    packages: Record<string, { version?: string }>;
  };
  for (const [name, version] of [
    ["nuxt", "5.0.0-2611031025-9496b7e"],
    ["nitro", "3.0.261029-rc"],
    ["vite", "8.3.5"],
    ["vue", "3.5.44"],
    ["@vizejs/nuxt", "0.676.0"],
    ["@vizejs/vite-plugin", "0.676.0"],
    ["@vizejs/native", "0.676.0"],
    ["vize", "0.676.0"],
  ])
    assert.equal(lock.packages[`node_modules/${name}`].version, version);
});

void test("Nuxt runtime ownership and frozen consumer inputs trigger the existing source lane", () => {
  for (const trigger of [workflow.on.pull_request, workflow.on.push]) {
    for (const file of [
      "npm/framework/nuxt/src/ssr-runtime.ts",
      "npm/framework/nuxt/src/ssr-runtime.test.ts",
      "tools/support/compat/nuxt/nuxt5-runtime-build.ts",
      "tools/support/compat/nuxt/fixtures/nuxt5-runtime-build/package-lock.json",
      "tests/tooling/nuxt5-runtime.test.ts",
    ])
      assert.ok(
        trigger.paths.some((pattern) => matchesGlob(file, pattern)),
        file,
      );
  }
});

void test("the additional Node 26 consumer follows the unchanged source pack and original Nuxt checks", () => {
  assert.deepEqual(Object.keys(workflow.jobs), ["nuxt3-module-build"]);
  assert.equal(job["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
  assert.equal(job["timeout-minutes"], 15);
  const reportedNode = step("Use the reported Node version for the additional Nuxt 5 consumer");
  assert.equal(reportedNode.uses, "actions/setup-node@249970729cb0ef3589644e2896645e5dc5ba9c38");
  assert.equal(reportedNode.with?.["node-version"], "26.13.0");
  for (const [name, run] of [
    [
      "Build and capture the actual source native compiler",
      "vp run --no-cache --filter './npm/native' build:debug",
    ],
    ["Build the candidate Vite adapter", "vp pack"],
    ["Build the candidate Nuxt integration", "vp pack"],
    [
      "Verify candidate build and SSR routes",
      'vp env exec node tools/support/compat/nuxt/nuxt3-module-build.ts "$RUNNER_TEMP/nuxt3-module-proof"',
    ],
    [
      "Verify real JavaScript Nuxt and Vite workspace products",
      'vp env exec node tools/support/compat/nuxt/javascript-workspace-build.ts "$RUNNER_TEMP/javascript-workspace-proof" "$RUNNER_TEMP/nuxt3-module-proof/source-native/custody.json"',
    ],
  ]) {
    const original = step(name);
    assert.equal(original.run, run);
    assert.ok(job.steps.indexOf(original) < job.steps.indexOf(reportedNode));
  }
  const consumer = step("Verify source-bound Nuxt 5 browser and isolated server output");
  assert.equal(
    consumer.run,
    'node tools/support/compat/nuxt/nuxt5-runtime-build.ts "$RUNNER_TEMP/nuxt5-runtime-proof/acceptance"',
  );
  assert.ok(job.steps.indexOf(reportedNode) < job.steps.indexOf(consumer));
});

void test("Nuxt 5 failures remain failures while the original and additional raw packets are retained", () => {
  assert.ok(!job["continue-on-error"]);
  for (const entry of job.steps) assert.ok(!entry["continue-on-error"]);
  const upload = step("Upload Nuxt 3 build and SSR evidence");
  assert.equal(upload.if, "always()");
  const paths = String(upload.with?.path).trim().split("\n");
  assert.deepEqual(
    paths,
    [
      "nuxt3-module-proof",
      "nuxt-prefetch-proof",
      "nuxt-lint-config-proof",
      "nuxt-compiler-defaults-proof",
      "vite-relative-glob-proof",
      "javascript-workspace-proof",
      "nuxt5-runtime-proof",
    ].map((name) => `\${{ runner.temp }}/${name}`),
  );
  const install = step("Install the frozen Nuxt 5 production reproduction");
  assert.equal(install.shell, "bash");
  assert.match(install.run ?? "", /npm ci --legacy-peer-deps --no-audit --no-fund/);
  assert.match(install.run ?? "", /tee "\$RUNNER_TEMP\/nuxt5-runtime-proof\/install\.log"/);
});

void test("the actual frozen install shell preserves npm failure through raw log capture", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "nuxt5-install-shell-"));
  try {
    const bin = path.join(directory, "bin");
    fs.mkdirSync(bin);
    fs.writeFileSync(
      path.join(bin, "npm"),
      "#!/bin/sh\nprintf 'controlled npm failure\\n' >&2\nexit 17\n",
      { mode: 0o755 },
    );
    const install = step("Install the frozen Nuxt 5 production reproduction");
    assert.equal(install.shell, "bash");
    assert.ok(install.run);
    const script = path.join(directory, "install.sh");
    fs.writeFileSync(script, install.run);
    for (const [variant, args, expected] of [
      ["default-no-pipefail", ["-e", script], 0],
      ["explicit-bash", ["--noprofile", "--norc", "-e", "-o", "pipefail", script], 17],
    ] as const) {
      const temporary = path.join(directory, variant);
      fs.mkdirSync(temporary);
      const result = spawnSync("/bin/bash", args, {
        encoding: "utf8",
        env: { ...process.env, PATH: `${bin}:${process.env.PATH}`, RUNNER_TEMP: temporary },
      });
      assert.equal(result.error, undefined);
      assert.equal(result.status, expected);
      assert.equal(
        fs.readFileSync(path.join(temporary, "nuxt5-runtime-proof/install.log"), "utf8"),
        "controlled npm failure\n",
      );
    }
  } finally {
    fs.rmSync(directory, { recursive: true });
  }
});
