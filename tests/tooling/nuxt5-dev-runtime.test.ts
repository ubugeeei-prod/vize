import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawn, spawnSync } from "node:child_process";
import { once } from "node:events";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path, { matchesGlob } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { parse } from "yaml";
import observeNitro2 from "../../tools/support/compat/nuxt/fixtures/nuxt5-dev-runtime/modules/runtime/stylesheet-observer.nitro2.ts";
import observeNitro3 from "../../tools/support/compat/nuxt/fixtures/nuxt5-dev-runtime/modules/runtime/stylesheet-observer.nitro3.ts";
import { closeConsumer } from "../../tools/support/compat/nuxt/host-types.ts";

type Step = {
  name?: string;
  run?: string;
  shell?: string;
  uses?: string;
  if?: string;
  env?: Record<string, string>;
  with?: { path?: string };
  "continue-on-error"?: boolean;
};
const root = new URL("../../", import.meta.url);

void test("consumer cleanup restores its SDK after the browser rejects closure", async () => {
  let restored = false;
  const browser = {
    async close() {
      throw new Error("controlled browser failure");
    },
  };
  await assert.rejects(
    closeConsumer(browser, undefined, () => {
      restored = true;
    }),
    /controlled browser failure/,
  );
  assert.equal(restored, true);
});

void test(
  "consumer cleanup restores its SDK when the child already exited by signal",
  { timeout: 2_000 },
  async () => {
    const server = spawn(process.execPath, ["--eval", 'process.kill(process.pid, "SIGTERM")']);
    await once(server, "exit");
    assert.equal(server.exitCode, null);
    assert.equal(server.signalCode, "SIGTERM");
    let restored = false;
    await closeConsumer(undefined, server, () => {
      restored = true;
    });
    assert.equal(restored, true);
  },
);
const fixture = new URL("tools/support/compat/nuxt/fixtures/nuxt5-dev-runtime/", root);
const workflow = parse(
  fs.readFileSync(new URL(".github/workflows/nuxt3-module-build.yml", root), "utf8"),
) as {
  on: { pull_request: { paths: string[] }; push: { paths: string[] } };
  jobs: Record<string, { "runs-on": string; "timeout-minutes": number; steps: Step[] }>;
};
const job = workflow.jobs["nuxt3-module-build"];
const find = (name: string) => {
  const step = job.steps.find((value) => value.name === name);
  assert.ok(step, name);
  return step;
};

void test("the derived development project freezes the reported dependency graph and every control input", () => {
  const provenance = JSON.parse(fs.readFileSync(new URL("provenance.json", fixture), "utf8")) as {
    files: Record<string, string>;
    versions: Record<string, string>;
  };
  for (const [file, expected] of Object.entries(provenance.files)) {
    assert.equal(
      createHash("sha256")
        .update(fs.readFileSync(new URL(file, fixture)))
        .digest("hex"),
      expected,
      file,
    );
  }
  const lock = JSON.parse(fs.readFileSync(new URL("package-lock.json", fixture), "utf8")) as {
    packages: Record<string, { version?: string }>;
  };
  for (const [name, version] of Object.entries(provenance.versions))
    assert.equal(lock.packages[`node_modules/${name}`].version, version, name);
  assert.equal(provenance.versions.nuxt, "5.0.0-2611031025-9496b7e");
  assert.equal(provenance.versions.nitro, "3.0.261029-rc");
  assert.equal(provenance.versions.vite, "8.3.5");
  assert.equal(provenance.versions.vue, "3.5.44");
});

void test("both control variants insert one broken and two identical valid stylesheet links", () => {
  for (const observer of [observeNitro2, observeNitro3]) {
    const callbacks: Array<(html: { head: string[] }) => void> = [];
    observer({
      hooks: {
        hook(name, handler) {
          assert.equal(name, "render:html");
          callbacks.push(handler);
        },
      },
    });
    assert.equal(callbacks.length, 1);
    const callback = callbacks[0];
    assert.ok(callback);
    const html = { head: ['<meta charset="utf-8">'] };
    callback(html);
    assert.deepEqual(html.head, [
      '<meta charset="utf-8">',
      '<link rel="stylesheet" href="/_nuxt/ghost-vize-entry.js">',
      '<link rel="stylesheet" href="/_nuxt/assets/stylesheet-control.css?direct">',
      '<link rel="stylesheet" href="/_nuxt/assets/stylesheet-control.css?direct">',
    ]);
  }
});

void test("the actual peer range accepts the reported nightly and stable Nuxt hosts", () => {
  const manifest = JSON.parse(
    fs.readFileSync(new URL("npm/framework/nuxt/package.json", root), "utf8"),
  ) as { peerDependencies: { nuxt: string } };
  // semver is declared by the root tooling workspace, so resolve its CLI there.
  const semver = createRequire(new URL("package.json", root)).resolve("semver/bin/semver.js");
  for (const [version, supported] of [
    ["2.16.0", true],
    ["3.19.3", true],
    ["4.6.4", true],
    ["5.0.0-2611031025-9496b7e", true],
    ["5.0.0", true],
    ["5.1.0", true],
    ["6.0.0", false],
  ] as const) {
    const result = spawnSync(
      process.execPath,
      [semver, version, "--range", manifest.peerDependencies.nuxt],
      { encoding: "utf8" },
    );
    assert.equal(result.error, undefined);
    assert.equal(result.status, supported ? 0 : 1, version);
    assert.equal(result.stderr, "");
  }
});

void test("the existing source job appends development qualification after production without changing its runner or budget", () => {
  assert.deepEqual(Object.keys(workflow.jobs), ["nuxt3-module-build"]);
  assert.equal(job["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
  assert.equal(job["timeout-minutes"], 15);
  const production = find("Verify source-bound Nuxt 5 browser and isolated server output");
  const development = find("Verify source-bound Nuxt 5 development stylesheets and hydration");
  assert.ok(job.steps.indexOf(production) < job.steps.indexOf(development));
  assert.equal(
    development.env?.VIZE_NUXT_SOURCE_NODE,
    "${{ steps.nuxt-source-runtime.outputs.path }}",
  );
  assert.equal(
    development.env?.VIZE_NUXT_SOURCE_NODE_SHA256,
    "${{ steps.nuxt-source-runtime.outputs.sha256 }}",
  );
  assert.equal(
    development.run,
    '"$VIZE_NUXT_SOURCE_NODE" tools/support/compat/nuxt/nuxt5-dev-runtime-build.ts "$RUNNER_TEMP/nuxt5-dev-runtime-proof/acceptance" "$(command -v node)"',
  );
  assert.notEqual(development["continue-on-error"], true);
  const artifact = job.steps.find((step) => step.uses?.startsWith("actions/upload-artifact@"));
  assert.ok(artifact);
  assert.equal(artifact.if, "always()");
  assert.ok(artifact.with?.path?.includes("${{ runner.temp }}/nuxt5-dev-runtime-proof"));
  assert.ok(job.steps.indexOf(artifact) > job.steps.indexOf(development));
  for (const trigger of [workflow.on.pull_request, workflow.on.push]) {
    for (const file of [
      "tools/support/compat/nuxt/fixtures/nuxt5-dev-runtime/app/app.vue",
      "tools/support/compat/nuxt/fixtures/nuxt5-dev-runtime/modules/runtime/stylesheet-observer.nitro3.ts",
      "tools/support/compat/nuxt/nuxt5-dev-runtime-build.ts",
      "tests/tooling/nuxt5-dev-runtime.test.ts",
    ])
      assert.ok(
        trigger.paths.some((pattern) => matchesGlob(file, pattern)),
        file,
      );
  }
});

void test("the actual development install shell retains npm failure and its raw log", () => {
  const step = find("Install the frozen Nuxt 5 development reproduction");
  assert.equal(step.shell, "bash");
  assert.ok(step.run);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "nuxt5-dev-install-"));
  try {
    fs.writeFileSync(
      path.join(directory, "npm"),
      "#!/bin/sh\nprintf 'controlled-install-failure\\n'\nexit 17\n",
      { mode: 0o755 },
    );
    const result = spawnSync(
      "/bin/bash",
      ["--noprofile", "--norc", "-e", "-o", "pipefail", "-c", step.run],
      {
        encoding: "utf8",
        env: {
          ...process.env,
          PATH: `${directory}${path.delimiter}${process.env.PATH}`,
          RUNNER_TEMP: directory,
        },
      },
    );
    assert.equal(result.error, undefined);
    assert.equal(result.status, 17);
    assert.match(
      fs.readFileSync(path.join(directory, "nuxt5-dev-runtime-proof/install.log"), "utf8"),
      /controlled-install-failure/,
    );
  } finally {
    fs.rmSync(directory, { recursive: true });
  }
});

void test("the real runner rejects a foreign producer path or image before loading any native consumer", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "nuxt5-dev-producer-"));
  const runner = fileURLToPath(
    new URL("tools/support/compat/nuxt/nuxt5-dev-runtime-build.ts", root),
  );
  try {
    for (const [id, node, hash] of [
      ["wrong-path", path.join(directory, "foreign-node"), "unused"],
      ["wrong-image", process.execPath, "foreign-image"],
    ] as const) {
      const output = path.join(directory, id);
      const result = spawnSync(process.execPath, [runner, output, process.execPath], {
        encoding: "utf8",
        env: { ...process.env, VIZE_NUXT_SOURCE_NODE: node, VIZE_NUXT_SOURCE_NODE_SHA256: hash },
      });
      assert.equal(result.error, undefined);
      assert.equal(result.status, 1, result.stderr);
      assert.match(result.stderr, /AssertionError/);
      assert.deepEqual(fs.readdirSync(output), []);
    }
  } finally {
    fs.rmSync(directory, { recursive: true });
  }
});
