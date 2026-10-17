import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { rustSmokeInvocation } from "../support/release-smoke-reference.ts";
import {
  NATIVE_RUNTIME_SCRIPT,
  INIT_SCRIPT,
} from "../../../tools/support/release/smoke_release_install/runtime.ts";

import { fakeNpm } from "./release-smoke-parity-peer.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const entry = path.join(root, "tools/commands/release/npm/smoke-release-install");
type Fixture = {
  name: string;
  status: number;
  manifests?: unknown[];
  manifestSources?: string[];
  expectedPreparedNumbers?: string[];
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
  manifestBytes: string[];
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
  {
    name: "well-formed Unicode strings and object keys",
    status: 0,
    manifests: [{ ...base, description: "雪😀", ["😀"]: "surrogate pair remains valid" }],
    expectedCommands: packInstall,
  },
  {
    name: "valid duplicate keys and quoted escape text retain their JSON meaning",
    status: 0,
    manifestSources: [
      String.raw`{"name":"@vizejs/parity","version":"1.2.3","main":"./index.js","key":1,"key":2,"escaped":"\\ud800 and 1e400"}`,
    ],
    expectedCommands: packInstall,
  },
  {
    name: "preparing manifests preserves exact signed and unsigned 64-bit source numbers",
    status: 0,
    prepare: true,
    manifestSources: [
      String.raw`{"name":"@vizejs/parity","version":"1.2.3","main":"./index.js","numbers":{"unsigned":18446744073709551615,"signed":-9223372036854775808,"nested":[9007199254740993]},"dependencies":{"@vizejs/parity":"workspace:*"}}`,
    ],
    expectedPreparedNumbers: ["18446744073709551615", "-9223372036854775808", "9007199254740993"],
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
      for (const number of fixture.expectedPreparedNumbers ?? []) {
        assert.ok(rust.manifestBytes[0].includes(number), `Rust must retain exact ${number}`);
        assert.ok(
          typescript.manifestBytes[0].includes(number),
          `TypeScript must retain exact ${number}`,
        );
      }
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
      fixture.manifestSources?.[index] ?? JSON.stringify(manifest, null, 2) + "\n",
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
  const invocation =
    driver === "rust"
      ? rustSmokeInvocation(argv)
      : { command: process.execPath, args: [entry + ".ts", ...argv] };
  const result = spawnSync(invocation.command, invocation.args, {
    cwd: root,
    encoding: "utf8",
    timeout: 150_000,
    env: {
      ...process.env,
      PATH: `${path.dirname(process.execPath)}${path.delimiter}${process.env.PATH ?? ""}`,
      NPM_BIN: fixture.realNpm === true ? (process.platform === "win32" ? "npm.cmd" : "npm") : npm,
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
  });
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
    manifestBytes: packageDirs.map((dir) =>
      fs.readFileSync(path.join(dir, "package.json"), "utf8"),
    ),
    manifests: packageDirs.map(
      (dir) => JSON.parse(fs.readFileSync(path.join(dir, "package.json"), "utf8")) as unknown,
    ),
  };
}

// The executed port retains the complete independently authored reference scripts,
// in addition to the installed-runtime controls in the Actions carrier.
test("native and fresh-init oracle scripts retain every original Rust byte", () => {
  const rust = fs.readFileSync(entry + ".rs", "utf8");
  const initSource = fs.readFileSync(
    path.join(root, "tools/support/release/npm_smoke_init.rs"),
    "utf8",
  );
  // Rust preprocesses CRLF into LF before tokenizing string literals:
  // https://doc.rust-lang.org/reference/tokens.html#string-literals
  // Compare the complete literal value for both checkout line-ending policies.
  for (const windowsCheckout of [false, true]) {
    const literalSource = (source: string) =>
      (windowsCheckout ? source.replace(/\r?\n/g, "\r\n") : source).replaceAll("\r\n", "\n");
    const native = /r#"(\nconst required = require\("@vizejs\/native"\);[\s\S]*?)"#/.exec(
      literalSource(rust),
    )?.[1];
    const init = /const SCRIPT: &str = r#"([\s\S]*?)"#;/.exec(literalSource(initSource))?.[1];
    assert.ok(native);
    assert.ok(init);
    assert.equal(NATIVE_RUNTIME_SCRIPT, native);
    assert.equal(INIT_SCRIPT, init);
  }
});

// Parser-engine diagnostics are retained as raw evidence, not rewritten into
// false byte parity. Both runtimes must refuse the same malformed input before
// invoking npm and keep the package path and failure context in their output.
test("both drivers refuse malformed JSON before npm and retain native parser diagnostics", () => {
  const temporary = fs.realpathSync.native(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-smoke-json-refusal-")),
  );
  try {
    const cases = [
      { source: "", operation: "parse JSON" },
      { source: '{"name":', operation: "parse JSON" },
      { source: '{"name":"unfinished', operation: "parse JSON" },
      { source: '{"name":"x",}', operation: "parse JSON" },
      { source: String.raw`{"name":"\ud800"}`, operation: "parse JSON" },
      { source: String.raw`{"name":"\udc00"}`, operation: "parse JSON" },
      { source: String.raw`{"\ud800":"x"}`, operation: "parse JSON" },
      { source: '{"name":"x","value":1e400}', operation: "parse JSON" },
      { source: '{"name":"x","value":-1e400}', operation: "parse JSON" },
      { source: String.raw`{"name":"x","value":"\ud800","value":"ok"}`, operation: "parse JSON" },
      { source: '{"name":"x","value":1e400,"value":0}', operation: "parse JSON" },
      { source: new Uint8Array([0x7b, 0x22, 0xff, 0x22, 0x3a, 0x31, 0x7d]), operation: "read" },
    ];
    for (const { source, operation } of cases) {
      const directory = path.join(temporary, "package");
      fs.mkdirSync(directory, { recursive: true });
      const manifest = path.join(directory, "package.json");
      fs.writeFileSync(manifest, source);
      const observations = [];
      for (const driver of ["rust", "typescript"]) {
        const invocation =
          driver === "rust"
            ? rustSmokeInvocation([directory])
            : { command: process.execPath, args: [entry + ".ts", directory] };
        const result = spawnSync(invocation.command, invocation.args, {
          cwd: root,
          encoding: "utf8",
          env: { ...process.env, NPM_BIN: path.join(temporary, "must-not-run-npm") },
        });
        assert.ifError(result.error);
        assert.equal(result.status, 1);
        assert.equal(result.stdout, "");
        assert.ok(result.stderr.startsWith(`cannot ${operation} ${manifest}: `), result.stderr);
        assert.ok(result.stderr.endsWith("\n"));
        observations.push({
          driver,
          status: result.status,
          stdout: result.stdout,
          stderr: result.stderr,
        });
      }
      console.log(JSON.stringify({ kind: "native-json-refusal", source, observations }));
    }
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
