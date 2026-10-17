import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const entry = path.join(root, "tools/commands/release/npm/smoke-release-install");
type Fixture = {
  name: string;
  status: number;
  manifests?: unknown[];
  args?: string[];
  fault?: string;
  prepare?: boolean;
  runtimeFlag?: string;
  realNpm?: boolean;
  expectedCommands: string[];
};
type Observation = {
  status: number | null;
  stdout: string;
  stderr: string;
  calls: unknown[];
  manifests: unknown[];
};
const base = { name: "@vizejs/parity", version: "1.2.3", main: "./index.js" };
const packInstall = ["pack", "install"];
const fixtures: Fixture[] = [
  { name: "usage", status: 1, args: [], expectedCommands: [] },
  { name: "unknown option", status: 1, args: ["--unknown"], expectedCommands: [] },
  { name: "missing package", status: 1, args: ["$MISSING"], expectedCommands: [] },
  { name: "non-object manifest", status: 1, manifests: [[]], expectedCommands: [] },
  {
    name: "private manifest",
    status: 1,
    manifests: [{ ...base, private: true }],
    expectedCommands: [],
  },
  { name: "missing name", status: 1, manifests: [{ version: "1.2.3" }], expectedCommands: [] },
  { name: "invalid name", status: 1, manifests: [{ ...base, name: 17 }], expectedCommands: [] },
  { name: "missing version", status: 1, manifests: [{ name: base.name }], expectedCommands: [] },
  {
    name: "invalid version",
    status: 1,
    manifests: [{ ...base, version: null }],
    expectedCommands: [],
  },
  {
    name: "nested missing entrypoints sorted and deduplicated",
    status: 1,
    manifests: [
      {
        ...base,
        main: "./z.js",
        types: "./a.d.ts",
        bin: { z: "./z.js" },
        exports: { ".": ["./b.js", { types: "./a.d.ts" }] },
      },
    ],
    expectedCommands: [],
  },
  {
    name: "non-file export schemes ignored",
    status: 0,
    manifests: [{ ...base, exports: ["#internal", "https://example.test/a", "node:fs", " "] }],
    expectedCommands: packInstall,
  },
  { name: "valid package", status: 0, expectedCommands: packInstall },
  {
    name: "runtime peers requested",
    status: 0,
    runtimeFlag: "--runtime-checks",
    expectedCommands: packInstall,
  },
  {
    name: "content mapper peers requested",
    status: 0,
    runtimeFlag: "--content-mapper-checks",
    expectedCommands: packInstall,
  },
  {
    name: "private false",
    status: 0,
    manifests: [{ ...base, private: false }],
    expectedCommands: packInstall,
  },
  { name: "zero tarballs", status: 1, fault: "zero-packs", expectedCommands: ["pack"] },
  { name: "multiple tarballs", status: 1, fault: "two-packs", expectedCommands: ["pack"] },
  { name: "pack command failure", status: 1, fault: "fail-pack", expectedCommands: ["pack"] },
  {
    name: "install command failure",
    status: 1,
    fault: "fail-install",
    expectedCommands: packInstall,
  },
  {
    name: "installed name mismatch",
    status: 1,
    fault: "wrong-name",
    expectedCommands: packInstall,
  },
  {
    name: "installed version mismatch",
    status: 1,
    fault: "wrong-version",
    expectedCommands: packInstall,
  },
  {
    name: "installed unresolved dependency",
    status: 1,
    fault: "installed-workspace",
    expectedCommands: packInstall,
  },
  {
    name: "installed missing entrypoint",
    status: 1,
    fault: "installed-entrypoint",
    expectedCommands: packInstall,
  },
  {
    name: "object bin installed",
    status: 0,
    manifests: [{ ...base, bin: { parity: "./cli.js" } }],
    expectedCommands: packInstall,
  },
  {
    name: "string bin installed",
    status: 0,
    manifests: [{ ...base, name: "parity", bin: "./cli.js" }],
    expectedCommands: packInstall,
  },
  {
    name: "missing installed bin",
    status: process.platform === "win32" ? 0 : 1,
    manifests: [{ ...base, bin: { parity: "./cli.js" } }],
    fault: "missing-bin",
    expectedCommands: packInstall,
  },
  {
    name: "non-executable installed bin",
    status: process.platform === "win32" ? 0 : 1,
    manifests: [{ ...base, bin: { parity: "./cli.js" } }],
    fault: "non-executable-bin",
    expectedCommands: packInstall,
  },
  {
    name: "all incompatible",
    status: 1,
    manifests: [{ ...base, os: ["!" + process.platform] }],
    expectedCommands: ["pack"],
  },
  {
    name: "empty platform lists",
    status: 0,
    manifests: [{ ...base, os: [], cpu: [], libc: [] }],
    expectedCommands: packInstall,
  },
  {
    name: "incompatible cpu",
    status: 1,
    manifests: [{ ...base, cpu: ["!" + process.arch] }],
    expectedCommands: ["pack"],
  },
  {
    name: "incompatible libc",
    status: 1,
    manifests: [{ ...base, libc: ["unsupported-libc"] }],
    expectedCommands: ["pack"],
  },
  {
    name: "compatible and pack-only packages",
    status: 0,
    manifests: [
      base,
      {
        name: "@vizejs/other-platform",
        version: "1.2.3",
        os: ["!" + process.platform],
        main: "missing-native.node",
      },
    ],
    expectedCommands: ["pack", "pack", "install"],
  },
  {
    name: "prepare workspace and catalog manifests",
    status: 0,
    prepare: true,
    manifests: [
      {
        ...base,
        dependencies: { "@vizejs/dependency": "workspace:^", unrelated: "catalog:tooling" },
      },
      { name: "@vizejs/dependency", version: "1.2.3" },
    ],
    expectedCommands: ["pack", "pack", "install"],
  },
  {
    name: "real npm local tarball and executable bin",
    status: 0,
    realNpm: true,
    manifests: [
      { ...base, bin: { parity: "./cli.js" }, dependencies: { "@vizejs/dependency": "1.2.3" } },
      { name: "@vizejs/dependency", version: "1.2.3" },
    ],
    expectedCommands: [],
  },
];
for (const section of [
  "dependencies",
  "optionalDependencies",
  "peerDependencies",
  "devDependencies",
]) {
  for (const protocol of ["workspace:*", "catalog:tooling"]) {
    fixtures.push({
      name: `${section} rejects ${protocol}`,
      status: 1,
      manifests: [{ ...base, [section]: { z: protocol, a: protocol } }],
      expectedCommands: [],
    });
  }
}

// These are subprocess observations of the unchanged Rust reference and the
// production TypeScript entrypoint. No duplicated smoke implementation is used.
for (const fixture of fixtures) {
  test(`release smoke Rust/TypeScript parity: ${fixture.name}`, { timeout: 180_000 }, () => {
    const temporary = fs.realpathSync.native(
      fs.mkdtempSync(path.join(os.tmpdir(), "vize-smoke-parity-")),
    );
    try {
      const rust = observe(fixture, "rust", temporary);
      const typescript = observe(fixture, "typescript", temporary);
      assert.equal(rust.status, fixture.status, JSON.stringify(rust));
      assert.equal(typescript.status, fixture.status, JSON.stringify(typescript));
      assert.deepEqual(typescript, rust);
      assert.deepEqual(commandNames(rust.calls), fixture.expectedCommands);
    } finally {
      fs.rmSync(temporary, { force: true, recursive: true });
    }
  });
}

function commandNames(calls: unknown[]): unknown[] {
  return calls.map((call) => {
    assert.ok(call !== null && typeof call === "object" && "argv" in call);
    assert.ok(Array.isArray(call.argv));
    return call.argv[0];
  });
}

function observe(fixture: Fixture, driver: "rust" | "typescript", temporary: string): Observation {
  const directory = path.join(temporary, `${driver} fixture 新規`);
  const packages = path.join(directory, "repo/npm");
  const temp = path.join(directory, "temp");
  fs.mkdirSync(packages, { recursive: true });
  fs.mkdirSync(temp);
  fs.writeFileSync(
    path.join(directory, "repo/pnpm-workspace.yaml"),
    'packages:\n  - "npm/*"\ncatalogs:\n  tooling:\n    unrelated: "2.3.4"\n',
  );
  const manifests = fixture.manifests ?? [base];
  const packageDirs = manifests.map((manifest, index) => {
    const packageDir = path.join(packages, String(index));
    fs.mkdirSync(packageDir);
    fs.writeFileSync(
      path.join(packageDir, "package.json"),
      JSON.stringify(manifest, null, 2) + "\n",
    );
    fs.writeFileSync(path.join(packageDir, "index.js"), "module.exports = { ok: true };\n");
    fs.writeFileSync(
      path.join(packageDir, "cli.js"),
      "#!/usr/bin/env node\nconsole.log('parity executable');\n",
      { mode: 0o755 },
    );
    return packageDir;
  });
  const npmPeer = path.join(directory, "fake-npm.cjs");
  fs.writeFileSync(npmPeer, `#!${process.execPath}\n${fakeNpm}`, { mode: 0o755 });
  const npm = process.platform === "win32" ? path.join(directory, "fake-npm.cmd") : npmPeer;
  if (process.platform === "win32") {
    fs.writeFileSync(
      npm,
      `@echo off\r\n"${process.execPath}" "${npmPeer}" %*\r\nexit /b %errorlevel%\r\n`,
    );
  }
  const argv = fixture.args?.map((arg) =>
    arg === "$MISSING" ? path.join(directory, "missing") : arg,
  ) ?? [
    "--keep-temp",
    ...(fixture.prepare === true ? ["--prepare-manifests"] : []),
    ...(fixture.runtimeFlag === undefined ? [] : [fixture.runtimeFlag]),
    ...packageDirs,
  ];
  const result = spawnSync(
    driver === "rust" ? "rust-script" : process.execPath,
    [entry + (driver === "rust" ? ".rs" : ".ts"), ...argv],
    {
      cwd: root,
      encoding: "utf8",
      timeout: 150_000,
      env: {
        ...process.env,
        PATH: `${path.dirname(process.execPath)}${path.delimiter}${process.env.PATH ?? ""}`,
        NPM_BIN:
          fixture.realNpm === true ? (process.platform === "win32" ? "npm.cmd" : "npm") : npm,
        NODE_BIN: process.execPath,
        TEMP: temp,
        TMP: temp,
        TMPDIR: temp,
        PARITY_LOG: path.join(directory, "calls.jsonl"),
        PARITY_FAULT: fixture.fault ?? "",
        VIZE_TEST_CONTENT_MAPPER_TSGO: "",
        npm_config_cache: path.join(directory, "npm-cache"),
        npm_config_update_notifier: "false",
      },
    },
  );
  assert.ifError(result.error);
  const kept = /^kept (.+)$/m.exec(result.stdout)?.[1];
  const normalize = (text: string): string => {
    let normalized = text;
    if (kept !== undefined) normalized = normalized.replaceAll(kept, "$SMOKE");
    normalized = normalized.replaceAll(directory, "$FIXTURE");
    return process.platform === "win32" ? normalized.replaceAll("\\", "/") : normalized;
  };
  const normalizeJson = (value: unknown): unknown => {
    if (typeof value === "string") return normalize(value);
    if (Array.isArray(value)) return value.map(normalizeJson);
    if (value !== null && typeof value === "object") {
      return Object.fromEntries(
        Object.entries(value).map(([key, item]) => [key, normalizeJson(item)]),
      );
    }
    return value;
  };
  const log = path.join(directory, "calls.jsonl");
  const calls = fs.existsSync(log)
    ? fs
        .readFileSync(log, "utf8")
        .trim()
        .split("\n")
        .map((line) => normalizeJson(JSON.parse(line)))
    : [];
  return {
    status: result.status,
    stdout: normalize(result.stdout),
    stderr: normalize(result.stderr),
    calls,
    manifests: packageDirs.map(
      (dir) => JSON.parse(fs.readFileSync(path.join(dir, "package.json"), "utf8")) as unknown,
    ),
  };
}

// A deterministic npm protocol peer, not a smoke implementation: it records
// exactly what the real driver requests and can corrupt the installed artifact.
const fakeNpm = String.raw`
const fs = require("node:fs");
const path = require("node:path");
const argv = process.argv.slice(2);
const cwd = process.cwd();
const fault = process.env.PARITY_FAULT;
const manifest = JSON.parse(fs.readFileSync(path.join(cwd, "package.json"), "utf8"));
fs.appendFileSync(process.env.PARITY_LOG, JSON.stringify({ argv, cwd, manifest }) + "\n");
if (fault === "fail-" + argv[0]) {
  process.stdout.write("npm child stdout\n");
  process.stderr.write("npm child stderr\n");
  process.exit(23);
}
if (argv[0] === "pack") {
  const destination = argv[argv.indexOf("--pack-destination") + 1];
  const count = fault === "zero-packs" ? 0 : fault === "two-packs" ? 2 : 1;
  for (let index = 0; index < count; index++) {
    fs.writeFileSync(path.join(destination, manifest.name.replaceAll("/", "-") + index + ".tgz"), JSON.stringify({ cwd, manifest }));
  }
} else if (argv[0] === "install") {
  for (const [name, specifier] of Object.entries(manifest.dependencies)) {
    if (!specifier.startsWith("file:")) continue;
    const packed = JSON.parse(fs.readFileSync(specifier.slice(5), "utf8"));
    const destination = path.join(cwd, "node_modules", name);
    fs.cpSync(packed.cwd, destination, { recursive: true });
    const installed = packed.manifest;
    if (fault === "wrong-name") installed.name = "@vizejs/counterfeit";
    if (fault === "wrong-version") installed.version = "0.0.0-counterfeit";
    if (fault === "installed-workspace") installed.dependencies = { unresolved: "workspace:*" };
    if (fault === "installed-entrypoint") installed.main = "missing-installed.js";
    fs.writeFileSync(path.join(destination, "package.json"), JSON.stringify(installed));
    const bins = typeof installed.bin === "string" ? [installed.name] : Object.keys(installed.bin || {});
    for (const bin of bins) {
      if (fault === "missing-bin") continue;
      const binPath = path.join(cwd, "node_modules/.bin", bin);
      fs.mkdirSync(path.dirname(binPath), { recursive: true });
      fs.writeFileSync(binPath, "#!/bin/sh\nexit 0\n", { mode: fault === "non-executable-bin" ? 0o644 : 0o755 });
    }
  }
} else {
  throw new Error("unexpected npm command " + JSON.stringify(argv));
}
`;
