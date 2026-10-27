import assert from "node:assert/strict";
import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const builder = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const root = path.resolve(builder, "../../..");
const provider = process.env.VIZE_TASK_CACHE_PROVIDER ?? "1.4.0";
const published = process.argv[2];
assert.ok(
  !published || /^\d+\.\d+\.\d+$/u.test(published),
  "optional argument is a published Vize version",
);
const publicVersion = published ?? "0.595.0";
const directory = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-task-cache-")));
const evidence = path.join(
  root,
  "target/differential/viteplus-task-cache",
  `${provider}${published ? `-published-${published}` : ""}.json`,
);
interface TaskRunOptions {
  hit?: boolean;
  status?: number;
  diagnostic?: RegExp;
  args?: string[];
  unchanged?: boolean;
}
interface TaskRunReceipt {
  label: string;
  task: string;
  args: string[];
  status: number | null;
  stdout: string;
  stderr: string;
  details: string;
  before: Record<string, string>;
  after: Record<string, string>;
  milliseconds: number;
}
const rows: TaskRunReceipt[] = [];
let packageLock: unknown;
let sourceArchive: { filename: string; sha256: string } | undefined;
const env = { ...process.env, NO_COLOR: "1", FORCE_COLOR: "0" };
const write = (file: string, contents: string): void => {
  fs.mkdirSync(path.dirname(path.join(directory, file)), { recursive: true });
  fs.writeFileSync(path.join(directory, file), contents);
};
function command(binary: string, args: string[], cwd = directory): SpawnSyncReturns<string> {
  const result = spawnSync(binary, args, {
    cwd,
    env,
    encoding: "utf8",
    timeout: 180_000,
    maxBuffer: 16 * 1024 * 1024,
  });
  assert.equal(result.error, undefined, result.error?.message);
  assert.equal(result.signal, null);
  return result;
}
function requireSuccess(binary: string, args: string[], cwd?: string): SpawnSyncReturns<string> {
  const result = command(binary, args, cwd);
  assert.equal(result.status, 0, `${args.join(" ")}\n${result.stdout}\n${result.stderr}`);
  return result;
}
function snapshot(): Record<string, string> {
  return Object.fromEntries(
    ["src/App.vue", "src/value.ts", "lint-policy.ts", "vite.config.ts"].map((file) => [
      file,
      createHash("sha256")
        .update(fs.readFileSync(path.join(directory, file)))
        .digest("hex"),
    ]),
  );
}
try {
  write(
    "package.json",
    JSON.stringify(
      {
        name: "task-cache-consumer",
        private: true,
        type: "module",
        scripts: { check: "node --version" },
        dependencies: {
          "vite-plus": provider,
          vite: `npm:@voidzero-dev/vite-plus-core@${provider}`,
          "@vizejs/vite-plugin": publicVersion,
          vize: publicVersion,
          vue: "3.5.41",
          typescript: "6.0.3",
        },
      },
      null,
      2,
    ),
  );
  requireSuccess("npm", ["install", "--legacy-peer-deps", "--no-audit", "--no-fund"]);
  const require = createRequire(path.join(directory, "package.json"));
  if (!published) {
    // Bundle the actual source integration and task runner into a real package.
    // The native CLI/addon remain independently installed public artifacts.
    const { rolldown } = await import(require.resolve("vite/rolldown"));
    const packed = path.join(directory, "packed");
    const bundle = await rolldown({
      input: {
        "vite-plus": path.join(builder, "src/vite-plus.ts"),
        "internal/vite-plus-task": path.join(builder, "src/internal/vite-plus-task.ts"),
        "internal/config-bridge": path.join(builder, "src/internal/config-bridge.ts"),
      },
      external: (id: string) => !id.startsWith(".") && !path.isAbsolute(id),
    });
    await bundle.write({
      dir: path.join(packed, "dist"),
      format: "esm",
      entryFileNames: "[name].mjs",
      chunkFileNames: "[name]-[hash].mjs",
    });
    await bundle.close();
    const manifest = JSON.parse(
      fs.readFileSync(
        path.join(
          path.dirname(require.resolve("@vizejs/vite-plugin/vite-plus")),
          "../package.json",
        ),
        "utf8",
      ),
    );
    manifest.version = `${publicVersion}-task-cache-test`;
    delete manifest.scripts;
    fs.writeFileSync(path.join(packed, "package.json"), JSON.stringify(manifest));
    const [archive] = JSON.parse(requireSuccess("npm", ["pack", "--json"], packed).stdout);
    sourceArchive = {
      filename: archive.filename,
      sha256: createHash("sha256")
        .update(fs.readFileSync(path.join(packed, archive.filename)))
        .digest("hex"),
    };
    requireSuccess("npm", [
      "install",
      "--legacy-peer-deps",
      "--no-audit",
      "--no-fund",
      path.join(packed, archive.filename),
    ]);
    // Build inputs are outside the consumer's source discovery scope.
    fs.rmSync(packed, { recursive: true, force: true });
  }
  packageLock = JSON.parse(fs.readFileSync(path.join(directory, "package-lock.json"), "utf8"));
  const vp = require.resolve("vite-plus/package.json").replace(/package\.json$/u, "bin/vp");
  const cleanVue =
    '<script setup lang="ts">\nimport { value } from "./value";\n</script>\n\n<template>\n  <div>{{ value }}</div>\n</template>\n';
  write("src/App.vue", cleanVue);
  write("src/value.ts", 'export const value: string = "hello";\n');
  write(
    "tsconfig.json",
    JSON.stringify(
      {
        compilerOptions: {
          strict: true,
          target: "ESNext",
          module: "ESNext",
          moduleResolution: "Bundler",
          noEmit: true,
          skipLibCheck: true,
        },
        include: ["src/**/*"],
      },
      null,
      2,
    ) + "\n",
  );
  write("lint-policy.ts", 'export const debuggerRule = "off";\n');
  const config = (taskCache = true) =>
    `import { defineConfig } from "@vizejs/vite-plugin/vite-plus";\nimport { debuggerRule } from "./lint-policy";\nexport default defineConfig({ lint: { rules: { "no-debugger": debuggerRule } }, fmt: { ignorePatterns: ["vite.config.ts", "lint-policy.ts", "package*.json", "tsconfig.json"] }, run: { tasks: { warmup: { command: "node --version", cache: false }, owned: { command: "node --version", cache: false } } } }, { plugin: false, taskCache: ${taskCache}, tasks: { lint: "verify:lint" } }).withVue({});\n`;
  write("vite.config.ts", config());
  // Initialize the provider's summary before checks enumerate the project.
  requireSuccess(process.execPath, [vp, "run", "warmup"]);
  function run(
    label: string,
    task: string,
    { hit, status = 0, diagnostic, args = [], unchanged = true }: TaskRunOptions = {},
  ): string {
    const before = snapshot();
    const start = performance.now();
    const result = command(process.execPath, [vp, "run", task, ...args]);
    const details = command(process.execPath, [vp, "run", "--last-details"]);
    const after = snapshot();
    const output = result.stdout + result.stderr;
    rows.push({
      label,
      task,
      args,
      status: result.status,
      stdout: result.stdout,
      stderr: result.stderr,
      details: details.stdout + details.stderr,
      before,
      after,
      milliseconds: performance.now() - start,
    });
    assert.equal(result.status, status, `${label}\n${output}`);
    const replayed = /cache hit, replaying/u.test(output);
    if (hit !== undefined) assert.equal(replayed, hit, `${label}\n${output}`);
    if (diagnostic) assert.match(output, diagnostic, label);
    if (unchanged) assert.deepEqual(after, before, `${label}: checks preserve authored files`);
    console.log(`${provider} ${label}: ${replayed ? "cache hit" : "executed"}`);
    return output;
  }
  // Real installed tools normalize the fixture before the read-only oracles.
  run("initial-format", "fmt", { unchanged: false });
  for (const task of ["verify:lint", "fmt:check", "typecheck", "vize:check"]) {
    run(`${task}-cold`, task, { hit: false });
    run(`${task}-hit`, task, { hit: true });
  }
  write("src/value.ts", 'export const value: string = "hello";\ndebugger;\n');
  run("source-change", "verify:lint", { hit: false });
  run("source-change-hit", "verify:lint", { hit: true });
  write("lint-policy.ts", 'export const debuggerRule = "error";\n');
  run("imported-config-diagnostic", "verify:lint", {
    hit: false,
    status: 1,
    diagnostic: /no-debugger/u,
  });
  write("lint-policy.ts", 'export const debuggerRule = "off";\n');
  run("imported-config-clean", "verify:lint");
  write(
    "vite.config.ts",
    config().replace('"no-debugger": debuggerRule', '"no-debugger": "error"'),
  );
  run("direct-config-diagnostic", "verify:lint", {
    hit: false,
    status: 1,
    diagnostic: /no-debugger/u,
  });
  write("vite.config.ts", config());
  write("src/value.ts", 'export const value: string = "hello";\n');
  run("typecheck-before-transitive-change", "typecheck");
  write("src/value.ts", "export const value: string = 42;\n");
  run("transitive-type-diagnostic", "typecheck", {
    hit: false,
    status: 1,
    diagnostic: /(?:number|42).*string|not assignable/u,
  });
  write("src/value.ts", 'export const value: string = "hello";\n');
  write("lint-policy.ts", 'export const debuggerRule = "error";\n');
  run("new-file-lint-baseline", "verify:lint");
  run("new-file-type-baseline", "typecheck");
  write("src/new-file.ts", "const marker: string = 1;\ndebugger;\n");
  run("new-file-lint-diagnostic", "verify:lint", {
    hit: false,
    status: 1,
    diagnostic: /new-file\.ts/u,
  });
  run("new-file-type-diagnostic", "typecheck", {
    hit: false,
    status: 1,
    diagnostic: /new-file\.ts/u,
  });
  fs.rmSync(path.join(directory, "src/new-file.ts"));
  write("lint-policy.ts", 'export const debuggerRule = "off";\n');
  write("node_modules/.vize/generated.ts", "first generation\n");
  write(".vize/generated.ts", "first generation\n");
  run("generated-files-baseline", "verify:lint");
  write("node_modules/.vize/generated.ts", "second generation\n");
  write(".vize/generated.ts", "second generation\n");
  run("generated-files-churn-hit", "verify:lint", { hit: true });
  for (const task of ["fmt", "lint:fix"]) {
    run(`${task}-first`, task, { hit: false, unchanged: false });
    run(`${task}-second-uncached`, task, { hit: false });
  }
  const dirtyVue = "<template><div   > changed </div></template>\n";
  write("src/App.vue", dirtyVue);
  run("check-fix-writes", "vize:check", { hit: false, args: ["--", "--fix"], unchanged: false });
  const formatted = fs.readFileSync(path.join(directory, "src/App.vue"), "utf8");
  assert.notEqual(formatted, dirtyVue);
  write("src/App.vue", dirtyVue);
  run("check-fix-identical-dirty-input", "vize:check", {
    hit: false,
    args: ["--", "--fix"],
    unchanged: false,
  });
  assert.equal(
    fs.readFileSync(path.join(directory, "src/App.vue"), "utf8"),
    formatted,
    "reintroducing the same dirty input must apply fixes again",
  );
  write("src/App.vue", "<template><div   > changed again </div></template>\n");
  run("check-fix-changed-source", "vize:check", {
    hit: false,
    args: ["--", "--fix"],
    unchanged: false,
  });
  assert.notEqual(
    fs.readFileSync(path.join(directory, "src/App.vue"), "utf8"),
    formatted,
    "cache must not restore an earlier fixed source",
  );
  write("vite.config.ts", config(false));
  run("opt-out-first", "verify:lint", { hit: false });
  run("opt-out-second", "verify:lint", { hit: false });
} finally {
  fs.mkdirSync(path.dirname(evidence), { recursive: true });
  fs.writeFileSync(
    evidence,
    JSON.stringify(
      {
        provider,
        publicVersion,
        published: Boolean(published),
        directory,
        sourceArchive,
        packageLock,
        rows,
      },
      null,
      2,
    ),
  );
  // Keep the independent streams and file fingerprints, remove only our fixture.
  fs.rmSync(directory, { recursive: true, force: true });
  console.log(`Evidence: ${evidence}`);
}
