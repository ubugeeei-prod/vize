import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path, { matchesGlob } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

type Step = {
  id?: string;
  env?: Record<string, string>;
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
  const formatConfig = fs.readFileSync(new URL("vite.config.ts", root), "utf8");
  assert.ok(
    formatConfig.includes('"tools/support/compat/nuxt/fixtures/nuxt5-runtime-build/package.json"'),
  );
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

void test("#11802 Nuxt 5 lazy hydration inputs and runtime oracle stay pinned", () => {
  const directory = new URL("lazy-hydration/", fixture);
  const provenance = JSON.parse(fs.readFileSync(new URL("provenance.json", directory), "utf8")) as {
    issue: string;
    runtime: Record<string, string>;
    files: Record<string, string>;
  };
  assert.equal(provenance.issue, "https://github.com/ubugeeei-prod/vize/issues/11802");
  assert.deepEqual(provenance.runtime, {
    nuxt: "5.0.0-2611031025-9496b7e",
    nitro: "3.0.261029-rc",
    vite: "8.3.5",
    vue: "3.5.44",
    node: "26.13.0",
  });
  for (const [file, expected] of Object.entries(provenance.files)) {
    assert.equal(
      createHash("sha256")
        .update(fs.readFileSync(new URL(file, directory)))
        .digest("hex"),
      expected,
      file,
    );
  }

  const harness = fs.readFileSync(
    new URL("tools/support/compat/nuxt/nuxt5-runtime-build.ts", root),
    "utf8",
  );
  assert.match(harness, /assert\.doesNotMatch\(lazyHtml, \/\\bhydrate-/);
  assert.match(harness, /\["true", "false", "false"\]/);
  assert.match(harness, /\["true", "true", "false"\]/);
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
  const capture = step("Capture the source producer Node executable");
  assert.equal(capture.id, "nuxt-source-runtime");
  assert.ok(job.steps.indexOf(capture) < job.steps.indexOf(reportedNode));
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
    '"$VIZE_NUXT_SOURCE_NODE" tools/support/compat/nuxt/nuxt5-runtime-build.ts "$RUNNER_TEMP/nuxt5-runtime-proof/acceptance" "$(command -v node)"',
  );
  assert.deepEqual(consumer.env, {
    VIZE_NUXT_SOURCE_NODE: "${{ steps.nuxt-source-runtime.outputs.path }}",
    VIZE_NUXT_SOURCE_NODE_SHA256: "${{ steps.nuxt-source-runtime.outputs.sha256 }}",
  });
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
      "nuxt-component-modes-proof",
      "nuxt-prefetch-proof",
      "nuxt-lint-config-proof",
      "nuxt-compiler-defaults-proof",
      "vite-relative-glob-proof",
      "javascript-workspace-proof",
      "nuxt5-runtime-proof",
      "nuxt5-dev-runtime-proof",
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

void test("the Nuxt controller uses the source runtime while forwarding the consumer executable", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "nuxt5-runtime-shell-"));
  try {
    const bin = path.join(directory, "bin");
    fs.mkdirSync(bin);
    fs.writeFileSync(
      path.join(bin, "node"),
      '#!/bin/sh\nprintf "ambient\\n" > "$AMBIENT_NODE_LOG"\n',
      { mode: 0o755 },
    );
    fs.writeFileSync(
      path.join(bin, "vp"),
      '#!/bin/sh\ntest "$1" = env && test "$2" = exec && test "$3" = node || exit 19\nshift 3\nexec "$TEST_PRODUCER_NODE" "$@"\n',
      { mode: 0o755 },
    );
    const capture = step("Capture the source producer Node executable").run;
    assert.ok(capture);
    const output = path.join(directory, "github-output");
    const environment = {
      ...process.env,
      PATH: `${bin}:${process.env.PATH}`,
      TEST_PRODUCER_NODE: process.execPath,
      GITHUB_OUTPUT: output,
      RUNNER_TEMP: directory,
    };
    const captured = spawnSync("/bin/bash", ["-e", "-c", capture], {
      encoding: "utf8",
      env: environment,
    });
    assert.equal(captured.error, undefined);
    assert.equal(captured.status, 0);
    const values = Object.fromEntries(
      fs
        .readFileSync(output, "utf8")
        .trim()
        .split("\n")
        .map((line) => {
          const split = line.indexOf("=");
          return [line.slice(0, split), line.slice(split + 1)];
        }),
    );
    assert.equal(values.path, process.execPath);
    assert.equal(
      values.sha256,
      createHash("sha256").update(fs.readFileSync(process.execPath)).digest("hex"),
    );
    const probe = path.join(directory, "tools/support/compat/nuxt/nuxt5-runtime-build.ts");
    fs.mkdirSync(path.dirname(probe), { recursive: true });
    fs.writeFileSync(
      probe,
      'require("node:fs").writeFileSync(process.env.PROBE_LOG, JSON.stringify({producer:process.execPath,args:process.argv.slice(2)}));\n',
    );
    const current = step("Verify source-bound Nuxt 5 browser and isolated server output").run;
    assert.ok(current);
    const before =
      'node tools/support/compat/nuxt/nuxt5-runtime-build.ts "$RUNNER_TEMP/nuxt5-runtime-proof/acceptance"';
    for (const [variant, command] of [
      ["ambient-node-before", before],
      ["source-controller", current],
    ]) {
      const observation = path.join(directory, variant);
      fs.mkdirSync(observation);
      const result = spawnSync("/bin/bash", ["-e", "-c", command], {
        encoding: "utf8",
        cwd: directory,
        env: {
          ...environment,
          VIZE_NUXT_SOURCE_NODE: values.path,
          VIZE_NUXT_SOURCE_NODE_SHA256: values.sha256,
          PROBE_LOG: path.join(observation, "probe.json"),
          AMBIENT_NODE_LOG: path.join(observation, "ambient.log"),
        },
      });
      assert.equal(result.error, undefined);
      assert.equal(result.status, 0);
      if (variant === "ambient-node-before") {
        assert.equal(fs.existsSync(path.join(observation, "probe.json")), false);
        assert.ok(fs.existsSync(path.join(observation, "ambient.log")));
      } else {
        assert.equal(fs.existsSync(path.join(observation, "ambient.log")), false);
        assert.deepEqual(
          JSON.parse(fs.readFileSync(path.join(observation, "probe.json"), "utf8")),
          {
            producer: values.path,
            args: [path.join(directory, "nuxt5-runtime-proof/acceptance"), path.join(bin, "node")],
          },
        );
      }
    }
  } finally {
    fs.rmSync(directory, { recursive: true });
  }
});
