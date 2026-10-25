// Installed consumer qualification; the native compiler stays at the exact
// released version while both JavaScript config adapters use the source tree.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../../", import.meta.url));
const fixture = path.join(root, "tests/_fixtures/_projects/vitest-project-extends");
const evidence = path.resolve(process.argv[2] ?? path.join(root, "target/vitest-project-extends"));
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
fs.mkdirSync(evidence, { recursive: true });
const locked = JSON.parse(fs.readFileSync(path.join(fixture, "package-lock.json"), "utf8"));
const packages = {};
for (const [name, version] of Object.entries({
  "@vizejs/vite-plugin": "0.566.0",
  vize: "0.566.0",
  "@vizejs/native": "0.566.0",
  vite: "8.0.16",
  vitest: "5.0.1",
  vue: "3.5.43",
})) {
  const entry = locked.packages[`node_modules/${name}`];
  assert.equal(entry.version, version);
  assert.match(entry.integrity, /^sha512-/);
  const installed = path.join(fixture, "node_modules", name);
  assert.equal(
    JSON.parse(fs.readFileSync(path.join(installed, "package.json"), "utf8")).version,
    version,
  );
  packages[name] = { version, integrity: entry.integrity, resolved: entry.resolved };
}
const rows = [];
const shapes = [
  {
    id: "resolve-inherited",
    host: { extends: true, resolve: { alias: { "#x": "/dev/null" } } },
    red: true,
  },
  { id: "css-inherited", host: { extends: true, css: {} }, red: true },
  { id: "define-inherited", host: { extends: true, define: { __UNIT__: true } }, red: false },
  { id: "bare-inherited", host: { extends: true }, red: false },
  {
    id: "resolve-default-inheritance",
    host: { resolve: { alias: { "#x": "/dev/null" } } },
    red: false,
  },
];
const config = (shape) => `import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";
export default defineConfig({
  plugins: [vize()],
  test: { projects: [${JSON.stringify({ ...shape.host, test: { name: "unit", environment: "node", include: ["src/**/*.spec.ts"] } })}] },
});\n`;
const originalConfig = fs.readFileSync(path.join(fixture, "vite.config.ts"));
const component = path.join(fixture, "src/Hello.vue");
const componentBytes = fs.readFileSync(`${component}.txt`);
assert.equal(fs.existsSync(component), false, "the tracked carrier owns the exact runtime input");

function files(directory) {
  return fs
    .readdirSync(directory, { withFileTypes: true })
    .flatMap((entry) => {
      const selected = path.join(directory, entry.name);
      return entry.isDirectory()
        ? files(selected)
        : [
            {
              path: path.relative(root, selected),
              sha256: hash(fs.readFileSync(selected)),
            },
          ];
    })
    .sort((a, b) => a.path.localeCompare(b.path));
}

function run(stage, shape, expectedRed) {
  const source = config(shape);
  fs.writeFileSync(path.join(fixture, "vite.config.ts"), source);
  const reportPath = path.join(evidence, `${stage}-${shape.id}.json`);
  fs.rmSync(reportPath, { force: true });
  const args = [
    path.join(fixture, "node_modules/vitest/vitest.mjs"),
    "run",
    "--project",
    "unit",
    "--reporter=json",
    "--outputFile",
    reportPath,
  ];
  const started = performance.now();
  const result = spawnSync(process.execPath, args, {
    cwd: fixture,
    encoding: "utf8",
    timeout: 60_000,
    env: { ...process.env, NO_COLOR: "1" },
  });
  const report = fs.existsSync(reportPath) ? fs.readFileSync(reportPath, "utf8") : null;
  rows.push({
    stage,
    shape: shape.id,
    source,
    args,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdout: result.stdout,
    stderr: result.stderr,
    report,
    milliseconds: performance.now() - started,
  });
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  if (expectedRed) {
    assert.notEqual(result.status, 0, `${stage}/${shape.id} must reproduce the released failure`);
    assert.match(result.stderr, /Custom config extends is no longer supported/);
  } else {
    assert.equal(result.status, 0, result.stdout + result.stderr);
    const parsed = JSON.parse(report);
    assert.equal(parsed.success, true);
    assert.equal(parsed.numPassedTests, 1);
    assert.equal(parsed.numFailedTests, 0);
  }
}

const candidate = {};
const installedBefore = {};
const nativePackage = path.join(fixture, "node_modules/@vizejs/native-linux-x64-gnu");
const nativeBefore = files(nativePackage);
try {
  fs.writeFileSync(component, componentBytes, { flag: "wx" });
  assert.deepEqual(fs.readFileSync(component), componentBytes);
  for (const shape of shapes) run("published", shape, shape.red);
  for (const [name, source] of Object.entries({
    vize: "npm/cli/dist",
    "@vizejs/vite-plugin": "npm/builder/vite/dist",
  })) {
    const destination = path.join(fixture, "node_modules", name, "dist");
    const built = path.join(root, source);
    candidate[name] = files(built);
    assert.ok(candidate[name].length > 0);
    installedBefore[name] = files(destination);
    fs.cpSync(destination, path.join(evidence, `published-${name.replaceAll("/", "-")}`), {
      recursive: true,
    });
    fs.rmSync(destination, { recursive: true });
    fs.cpSync(built, destination, { recursive: true });
    assert.deepEqual(
      files(destination).map(({ path: entry, ...rest }) => ({
        path: entry.slice(entry.indexOf("/dist/") + 6),
        ...rest,
      })),
      candidate[name].map(({ path: entry, ...rest }) => ({
        path: entry.slice(entry.indexOf("/dist/") + 6),
        ...rest,
      })),
    );
  }
  for (const shape of shapes) run("candidate", shape, false);
  // Re-running each previously failing project proves stable configuration
  // admission in a fresh actual Vitest invocation after the first repair.
  for (const shape of shapes.filter((entry) => entry.red)) run("repaired-repeat", shape, false);
} finally {
  const componentAfter = fs.existsSync(component) ? fs.readFileSync(component) : null;
  const nativeAfter = files(nativePackage);
  fs.rmSync(component, { force: true });
  fs.writeFileSync(path.join(fixture, "vite.config.ts"), originalConfig);
  fs.writeFileSync(
    path.join(evidence, "observations.json"),
    JSON.stringify(
      {
        issue: 10621,
        sourceCommit: spawnSync("git", ["rev-parse", "HEAD"], {
          cwd: root,
          encoding: "utf8",
        }).stdout.trim(),
        sourceTree: spawnSync("git", ["rev-parse", "HEAD^{tree}"], {
          cwd: root,
          encoding: "utf8",
        }).stdout.trim(),
        lockSha256: hash(fs.readFileSync(path.join(fixture, "package-lock.json"))),
        nativeCompiler: {
          provider: "published 0.566.0",
          platform: "@vizejs/native-linux-x64-gnu",
          before: nativeBefore,
          after: nativeAfter,
        },
        component: {
          carrier: "src/Hello.vue.txt",
          runtimePath: "src/Hello.vue",
          sha256: hash(componentBytes),
          bytes: componentBytes.length,
          afterBase64: componentAfter?.toString("base64") ?? null,
        },
        packages,
        candidate,
        installedBefore,
        rows,
      },
      null,
      2,
    ) + "\n",
  );
  assert.deepEqual(
    componentAfter,
    componentBytes,
    "both adapters preserve the whole authored component",
  );
  assert.deepEqual(
    nativeAfter,
    nativeBefore,
    "candidate installation preserves the exact native compiler package",
  );
}
