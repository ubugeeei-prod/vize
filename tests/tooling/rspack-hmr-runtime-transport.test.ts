import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";

const repo = fileURLToPath(new URL("../../", import.meta.url));
const workflow = parse(readFileSync(path.join(repo, ".github/workflows/rspack-hmr.yml"), "utf8"));
const browser = workflow.jobs.browser.steps.find(
  (step: { name?: string }) => step.name === "Run browser HMR",
);
const [vp, ...argv] = browser.run.split(" ") as string[];

test("the HMR task retains its original build and all browser cases without caching a session", () => {
  const manifest = JSON.parse(
    readFileSync(path.join(repo, "npm/builder/rspack/package.json"), "utf8"),
  );
  assert.equal(manifest.scripts["test:hmr"], "pnpm build && node --test test/hmr.test.mjs");
  assert.equal(vp, "vp");
  assert.deepEqual(argv, ["run", "--no-cache", "--filter", "@vizejs/rspack-plugin", "test:hmr"]);
});

test("the real Vite Plus child resolves only the selected isolated HMR runtime", () => {
  const directory = realpathSync(
    mkdtempSync(path.join(tmpdir(), "vize-rspack-runtime-transport-")),
  );
  try {
    const runtime = path.join(directory, "isolated runtime 'selected'");
    const server = path.join(runtime, "node_modules/@rspack/dev-server/index.js");
    const output = path.join(directory, "observed.json");
    mkdirSync(path.dirname(server), { recursive: true });
    writeFileSync(server, "module.exports = {};\n");
    writeFileSync(path.join(runtime, "package.json"), '{"name":"isolated-hmr-runtime"}\n');
    writeFileSync(
      path.join(directory, "package.json"),
      JSON.stringify({ name: "@vizejs/rspack-plugin", scripts: { "test:hmr": "node probe.mjs" } }),
    );
    writeFileSync(
      path.join(directory, "probe.mjs"),
      `import { writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
const observed = { runtime: process.env.VIZE_HMR_RUNTIME ?? null, server: null, error: null };
try {
  observed.server = createRequire(path.resolve(process.env.VIZE_HMR_RUNTIME ?? ".", "package.json")).resolve("@rspack/dev-server");
} catch (error) { observed.error = error.code; process.exitCode = 1; }
writeFileSync(${JSON.stringify(output)}, JSON.stringify(observed));
`,
    );
    writeFileSync(
      path.join(directory, "vite.config.mjs"),
      "export default { run: { cache: { scripts: true, tasks: true } } };\n",
    );
    const run = (args: string[]) => {
      rmSync(output, { force: true });
      const result = spawnSync(
        process.env.VIZE_VP_BIN ?? path.join(repo, "node_modules/.bin/vp"),
        args,
        {
          cwd: directory,
          encoding: "utf8",
          timeout: 30_000,
          env: { ...process.env, VIZE_HMR_RUNTIME: runtime },
        },
      );
      assert.ok(existsSync(output), `${result.stdout}\n${result.stderr}`);
      return {
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
        observed: JSON.parse(readFileSync(output, "utf8")),
      };
    };
    assert.deepEqual(run(argv.filter((arg) => arg !== "--no-cache")), {
      status: 1,
      signal: null,
      error: null,
      observed: { runtime: null, server: null, error: "MODULE_NOT_FOUND" },
    });
    assert.deepEqual(run(argv), {
      status: 0,
      signal: null,
      error: null,
      observed: { runtime, server, error: null },
    });
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
