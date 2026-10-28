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
const preflight = process.env.VIZE_TASK_CACHE_PREFLIGHT === "1";
let configPreflight: unknown;
assert.ok(
  !published || /^\d+\.\d+\.\d+$/u.test(published),
  "optional argument is a published Vize version",
);
const publicVersion = published ?? "0.595.0";
const sourceCli = process.env.VIZE_TASK_CACHE_SOURCE_CLI || undefined;
assert.ok(!published || !sourceCli, "public acceptance never replaces a published CLI");
const directory = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-task-cache-")));
const privateCache = sourceCli
  ? fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-task-private-cache-")))
  : undefined;
const evidence = path.join(
  root,
  "target/differential/viteplus-task-cache",
  `${provider}${published ? `-published-${published}` : sourceCli ? (preflight ? "-source-cli-preflight" : "-source-cli") : ""}.json`,
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
let sourceCliReceipt: unknown;
let privateStorage: { root: string; files: { path: string; bytes: number }[] } | undefined;
let privateStorageInspection: { root: string; entries: string[] } | undefined;
const env: NodeJS.ProcessEnv = { ...process.env, NO_COLOR: "1", FORCE_COLOR: "0" };
if (privateCache) env.XDG_CACHE_HOME = privateCache;
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
  if (sourceCli) {
    assert.equal(process.platform, "linux");
    const expected = process.env.VIZE_TASK_CACHE_SOURCE_BINARY_SHA256;
    assert.ok(expected && /^[a-f0-9]{64}$/u.test(expected));
    const hash = createHash("sha256");
    for await (const chunk of fs.createReadStream(sourceCli)) hash.update(chunk);
    assert.equal(hash.digest("hex"), expected, "the exact verified source CLI executes");
    const receipt = process.env.VIZE_TASK_CACHE_SOURCE_RECEIPT;
    assert.ok(receipt);
    const launcher = path.resolve(path.dirname(require.resolve("vize")), "../bin/vize");
    const originalLauncher = fs.readFileSync(launcher, "utf8");
    // Only this disposable install's launcher delegates to the real, verified
    // Rust CLI. Its original arguments, environment and all streams pass through.
    fs.writeFileSync(
      launcher,
      `#!/usr/bin/env node\nimport { spawn } from "node:child_process";\nconsole.error("VIZE_TASK_CACHE_SOURCE_ENV " + JSON.stringify({ cwd: process.cwd(), xdgCacheHome: process.env.XDG_CACHE_HOME, tmpdir: process.env.TMPDIR, argv: process.argv.slice(2) }));\nconst child = spawn(${JSON.stringify(sourceCli)}, process.argv.slice(2), { stdio: "inherit" });\nconst stop = (signal) => { if (!child.killed) child.kill(signal); };\nprocess.on("SIGTERM", () => stop("SIGTERM"));\nprocess.on("SIGINT", () => stop("SIGINT"));\nchild.on("error", (error) => { throw error; });\nchild.on("exit", (code, signal) => {\n  if (signal) process.kill(process.pid, signal);\n  else process.exitCode = code ?? 1;\n});\n`,
    );
    sourceCliReceipt = {
      verified: JSON.parse(fs.readFileSync(receipt, "utf8")) as unknown,
      launcher: {
        path: launcher,
        original: originalLauncher,
        originalSha256: createHash("sha256").update(originalLauncher).digest("hex"),
        delegatedSha256: createHash("sha256").update(fs.readFileSync(launcher)).digest("hex"),
      },
    };
  }
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
        // Register a genuine installed declaration so this source-only fixture
        // exercises the installed-input private store rather than the ordinary root.
        ...(sourceCli ? { files: ["node_modules/vue/jsx.d.ts"] } : {}),
        include: ["src/**/*"],
      },
      null,
      2,
    ) + "\n",
  );
  write("lint-policy.ts", 'export const debuggerRule = "off";\n');
  const config = (taskCache = true) =>
    `import { defineConfig } from "@vizejs/vite-plugin/vite-plus";\nimport { debuggerRule } from "./lint-policy";\nexport default defineConfig({ lint: { rules: { "no-debugger": debuggerRule } }, fmt: { ignorePatterns: ["vite.config.ts", "lint-policy.ts", "package*.json", "tsconfig.json"] }, run: { tasks: { warmup: { command: "node --version", cache: false }, owned: { command: "node --version", cache: false } } } }, { plugin: false, taskCache: ${taskCache}, tasks: { lint: "verify:lint" } }).withVue({});\n`;
  const sourceConfig = (taskCache = true): string => {
    const original = config(taskCache);
    if (!sourceCli) return original;
    // Vite+ filters non-system environment for cached tasks. Declare the owned
    // cache redirect as a tracked input using the installed provider's API.
    return (
      original.replace("export default ", "const factory = ") +
      "export default async (env) => {\nconst config = await factory(env);\n" +
      "for (const task of Object.values(config.run.tasks)) {\n" +
      '  if (task && typeof task === "object" && task.cache) {\n' +
      '    const cache = typeof task.cache === "object" ? task.cache : task;\n' +
      '    cache.env = [...new Set([...(cache.env ?? []), "XDG_CACHE_HOME"])];\n' +
      "  }\n" +
      "}\nreturn config;\n};\n"
    );
  };
  write("vite.config.ts", sourceConfig());
  if (preflight) {
    assert.ok(sourceCli, "preflight requires the verified source CLI");
    const normalized = () =>
      JSON.parse(
        requireSuccess(process.execPath, [
          "--input-type=commonjs",
          "-e",
          'const assert=require("node:assert/strict");const{loadConfigFromFile}=require("vite-plus");(async()=>{const loaded=await loadConfigFromFile({command:"build",mode:"production"});assert(loaded&&loaded.config&&loaded.config.run&&loaded.config.run.tasks);process.stdout.write(JSON.stringify(loaded.config.run.tasks));})().catch(e=>{console.error(e);process.exitCode=1;});',
        ]).stdout,
      );
    write("vite.config.ts", config());
    const before = normalized();
    write("vite.config.ts", sourceConfig());
    const after = normalized();
    const verification = JSON.parse(
      requireSuccess(process.execPath, [
        "--input-type=commonjs",
        "-e",
        'const assert=require("node:assert/strict");const{before,after}=JSON.parse(process.argv[1]);assert.deepEqual(Object.keys(after).sort(),Object.keys(before).sort());let checked=0;for(const name of Object.keys(after)){const current=after[name],previous=before[name];if(current&&typeof current==="object"&&current.cache){const now=typeof current.cache==="object"?current.cache:current;const old=typeof previous.cache==="object"?previous.cache:previous;const original=old.env??[];assert(Array.isArray(now.env));assert(now.env.includes("XDG_CACHE_HOME"));for(const key of original)assert(now.env.includes(key));assert.equal(new Set(now.env).size,now.env.length);assert.equal(now.env.length,new Set([...original,"XDG_CACHE_HOME"]).size);delete now.env;delete old.env;checked++;}}assert.equal(checked,4);assert.deepEqual(after,before,"all other normalized task fields are unchanged");process.stdout.write(JSON.stringify({resolvedFactory:true,trackedEnvironmentVerified:checked,allOtherTaskFieldsUnchanged:true}));',
        JSON.stringify({ before, after }),
      ]).stdout,
    );
    configPreflight = { before, after, verification };
  }
  // Initialize the provider's summary before checks enumerate the project.
  requireSuccess(process.execPath, [vp, "run", "warmup"]);
  function run(
    label: string,
    task: string,
    { hit, status = 0, diagnostic, args = [], unchanged = true }: TaskRunOptions = {},
  ): string {
    // Explicit source directories exercise the CLI's installed ambient-input closure.
    const taskArgs =
      sourceCli && ["verify:lint", "fmt:check", "typecheck", "vize:check"].includes(task)
        ? [...args, "src"]
        : args;
    const before = snapshot();
    const start = performance.now();
    const result = command(process.execPath, [vp, "run", task, ...taskArgs]);
    const details = command(process.execPath, [vp, "run", "--last-details"]);
    const after = snapshot();
    const output = result.stdout + result.stderr;
    rows.push({
      label,
      task,
      args: taskArgs,
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
    if (preflight) {
      assert.ok(privateCache);
      assert.deepEqual(args, []);
      assert.deepEqual(taskArgs, ["src"]);
      assert.doesNotMatch(output, /No files found/u);
      const delegates = [...result.stderr.matchAll(/^VIZE_TASK_CACHE_SOURCE_ENV (.+)$/gmu)].map(
        (match) => JSON.parse(match[1]) as { cwd: string; xdgCacheHome: string; argv: string[] },
      );
      const expected =
        task === "vize:check"
          ? ["check", "lint", "fmt"]
          : task === "typecheck"
            ? ["check"]
            : task === "fmt:check"
              ? ["fmt"]
              : ["lint"];
      assert.deepEqual(
        delegates.map((selected) => selected.argv[0]),
        expected,
      );
      for (const selected of delegates) {
        assert.equal(selected.cwd, directory);
        assert.equal(selected.xdgCacheHome, privateCache);
        assert.equal(selected.argv.at(-1), selected.argv[0] === "fmt" ? "src/App.vue" : "src");
      }
      if (task === "typecheck" || task === "vize:check")
        assert.match(output, /Type checked 3 files/u);
      if (task === "verify:lint" || task === "vize:check")
        assert.match(output, /No problems found in 2 file\(s\)/u);
      if (task !== "typecheck") assert.match(output, /on [1-9]\d* files/u);
      if (task === "fmt:check" || task === "vize:check")
        assert.match(result.stderr, /Checked 1 file\(s\)/u);
    }
    console.log(`${provider} ${label}: ${replayed ? "cache hit" : "executed"}`);
    return output;
  }
  // Real installed tools normalize the fixture before the read-only oracles.
  if (!preflight) run("initial-format", "fmt", { unchanged: false });
  for (const task of ["verify:lint", "fmt:check", "typecheck", "vize:check"]) {
    run(`${task}-cold`, task, { hit: false });
    if (task === "typecheck" && privateCache) {
      const row = rows.find((selected) => selected.label === "typecheck-cold");
      assert.ok(row);
      const delegatedEnvironment = /^VIZE_TASK_CACHE_SOURCE_ENV (.+)$/mu.exec(row.stderr);
      assert.ok(delegatedEnvironment, "retain the actual delegated process environment");
      const actualEnvironment = JSON.parse(delegatedEnvironment[1]) as {
        cwd: string;
        xdgCacheHome: string;
      };
      assert.equal(actualEnvironment.cwd, directory);
      assert.equal(actualEnvironment.xdgCacheHome, privateCache);
      privateStorageInspection = {
        root: privateCache,
        entries: fs.readdirSync(privateCache, { recursive: true, encoding: "utf8" }),
      };
      const storage = path.join(privateCache, "vize/canon/private-batch-v1");
      const files: { path: string; bytes: number }[] = [];
      const visit = (selected: string): void => {
        for (const entry of fs.readdirSync(selected, { withFileTypes: true })) {
          const file = path.join(selected, entry.name);
          if (entry.isDirectory()) visit(file);
          else if (entry.isFile())
            files.push({ path: path.relative(storage, file), bytes: fs.statSync(file).size });
        }
      };
      visit(storage);
      assert.ok(files.length > 0, "typecheck uses the new private batch store outside the project");
      privateStorage = { root: storage, files };
    }
    if (!preflight) run(`${task}-hit`, task, { hit: true });
  }
  if (privateCache) {
    privateStorageInspection = {
      root: privateCache,
      entries: fs.readdirSync(privateCache, { recursive: true, encoding: "utf8" }),
    };
  }
  if (!preflight) {
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
      sourceConfig().replace('"no-debugger": debuggerRule', '"no-debugger": "error"'),
    );
    run("direct-config-diagnostic", "verify:lint", {
      hit: false,
      status: 1,
      diagnostic: /no-debugger/u,
    });
    write("vite.config.ts", sourceConfig());
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
    write("vite.config.ts", sourceConfig(false));
    run("opt-out-first", "verify:lint", { hit: false });
    run("opt-out-second", "verify:lint", { hit: false });
  }
} finally {
  fs.mkdirSync(path.dirname(evidence), { recursive: true });
  fs.writeFileSync(
    evidence,
    JSON.stringify(
      {
        runtime: {
          node: process.version,
          platform: process.platform,
          arch: process.arch,
          cpus: os.cpus().length,
          availableParallelism: os.availableParallelism(),
          runnerName: process.env.RUNNER_NAME,
          runnerOs: process.env.RUNNER_OS,
          imageOs: process.env.ImageOS,
          imageVersion: process.env.ImageVersion,
        },
        provider,
        preflight,
        configPreflight,
        publicVersion,
        published: Boolean(published),
        directory,
        sourceArchive,
        sourceCliReceipt,
        privateStorage,
        privateStorageInspection,
        packageLock,
        rows,
      },
      null,
      2,
    ),
  );
  // Keep the independent streams and file fingerprints, remove only our fixture.
  fs.rmSync(directory, { recursive: true, force: true });
  if (privateCache) fs.rmSync(privateCache, { recursive: true, force: true });
  console.log(`Evidence: ${evidence}`);
}
