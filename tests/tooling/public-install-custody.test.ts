import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const producer = path.join(root, "tools/support/release/public_install");
const hook = path.join(producer, "native-custody.cjs");
const overrideKeys = [
  "NODE_OPTIONS",
  "VIZE_PREFER_WORKSPACE_BINDING",
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
  "VIZE_ALLOW_NATIVE_VERSION_MISMATCH",
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
  "VIZE_PUBLIC_NATIVE_CUSTODY",
  "VIZE_OXLINT_NATIVE_CUSTODY",
];
const cleanEnvironment = () => {
  const env = { ...process.env };
  for (const key of overrideKeys) delete env[key];
  return env;
};
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

for (const suite of [
  "ArchiveControls",
  "RegistryControls",
  "IdentityControls",
  "JournalControls",
  "SourceControls",
]) {
  test(`public install custody inert ${suite} rejection laws`, () => {
    const result = spawnSync(
      "python3",
      [
        "-I",
        path.join(
          root,
          "tests/tooling/support/public-install",
          suite === "SourceControls" ? "source_controls.py" : "controls.py",
        ),
        suite,
      ],
      {
        cwd: root,
        env: cleanEnvironment(),
        encoding: "utf8",
        timeout: 30_000,
      },
    );
    assert.equal(result.error, undefined);
    assert.equal(result.status, 0, result.stdout + result.stderr);
    assert.match(result.stderr, /\bOK\b/u);
  });
}

test("official Rust entry rejects every nonempty override before Python or Node startup", (t) => {
  const temporary = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "public-install-guard-")),
  );
  t.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
  const entry = path.join(root, "tools/commands/release/npm/collect-public-install.rs");
  const executable = path.join(temporary, process.platform === "win32" ? "guard.exe" : "guard");
  const compile = spawnSync(
    "rustc",
    ["--edition=2024", "--crate-name", "public_install_guard", entry, "-o", executable],
    {
      encoding: "utf8",
      timeout: 30_000,
    },
  );
  assert.equal(compile.error, undefined);
  assert.equal(compile.status, 0, compile.stdout + compile.stderr);
  const sentinel = path.join(temporary, "preload-ran");
  const preload = path.join(temporary, "preload.cjs");
  fs.writeFileSync(
    preload,
    `require('node:fs').writeFileSync(${JSON.stringify(sentinel)}, 'unexpected Node startup')`,
  );
  for (const key of overrideKeys) {
    const result = spawnSync(executable, ["--print-collector-authority"], {
      env: {
        ...cleanEnvironment(),
        RUST_SCRIPT_PATH: entry,
        [key]: key === "NODE_OPTIONS" ? `--require ${preload}` : "forbidden",
      },
      encoding: "utf8",
      timeout: 10_000,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.status, 1);
    assert.equal(result.stdout, "");
    assert.equal(result.stderr.trim(), `initial source/runtime override must be empty: ${key}`);
    assert.equal(fs.existsSync(sentinel), false);
  }
});

test("direct Python collector rejects initial overrides before any probe or output", () => {
  for (const key of overrideKeys) {
    const result = spawnSync("python3", ["-I", path.join(producer, "collect.py")], {
      env: { ...cleanEnvironment(), [key]: "forbidden" },
      encoding: "utf8",
      timeout: 10_000,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.status, 1);
    assert.equal(result.stdout, "");
    assert.equal(result.stderr.trim(), `initial source/runtime overrides must be empty: ${key}`);
  }
});

function inertHookFixture() {
  const temporary = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "public-install-loader-")),
  );
  const install = path.join(temporary, "install");
  const native = path.join(install, "node_modules/@vizejs/native-darwin-arm64/vize.node");
  fs.mkdirSync(path.dirname(native), { recursive: true });
  fs.writeFileSync(native, "Inert text, never a functioning native library.\n");
  const journal = path.join(temporary, "native-journal.jsonl");
  const config = {
    schema: "vize-public-native-custody-v1",
    installRoot: install,
    nativePath: native,
    nativeSha256: digest(fs.readFileSync(native)),
    journalPath: journal,
  };
  const run = (script: string, changes: Record<string, string> = {}) =>
    spawnSync(process.execPath, ["--require", hook, "-e", script], {
      env: {
        ...cleanEnvironment(),
        VIZE_PUBLIC_NATIVE_CUSTODY: JSON.stringify(config),
        ...changes,
      },
      encoding: "utf8",
      timeout: 10_000,
    });
  const events = () =>
    fs
      .readFileSync(journal, "utf8")
      .trimEnd()
      .split("\n")
      .map((line) => JSON.parse(line));
  return { temporary, native, journal, config, run, events };
}

test("the genuine Node loader rejects inert native text and never journals returned", (t) => {
  const fixture = inertHookFixture();
  t.after(() => fs.rmSync(fixture.temporary, { recursive: true, force: true }));
  const result = fixture.run(
    `try { require(${JSON.stringify(fixture.native)}) } catch (error) { process.stdout.write(error.message); process.exitCode = 42 }`,
  );
  assert.equal(result.error, undefined);
  assert.equal(result.status, 42);
  assert.equal(result.stderr, "");
  const events = fixture.events();
  assert.deepEqual(
    events.map((event) => event.event),
    ["initialized", "attempt", "failed", "exit"],
  );
  assert.equal(events[1].expectedNative, true);
  assert.equal(events[2].message, result.stdout);
  assert.ok(result.stdout.length > 0);
  assert.equal(events[3].code, 42);
  assert.ok(events.every((event) => event.pid === events[0].pid));
});

test("native bytes changed after hook initialization are rejected before the loader", (t) => {
  const fixture = inertHookFixture();
  t.after(() => fs.rmSync(fixture.temporary, { recursive: true, force: true }));
  const result = fixture.run(
    `require('node:fs').writeFileSync(${JSON.stringify(fixture.native)}, 'tampered'); require(${JSON.stringify(fixture.native)})`,
  );
  assert.equal(result.status, 1);
  assert.match(result.stderr, /Vize native loader escaped/u);
  assert.deepEqual(
    fixture.events().map((event) => event.event),
    ["initialized", "attempt", "rejected", "exit"],
  );
});

test("another Vize native provider is rejected instead of granting load credit", (t) => {
  const fixture = inertHookFixture();
  t.after(() => fs.rmSync(fixture.temporary, { recursive: true, force: true }));
  const alternate = path.join(path.dirname(fixture.native), "alternate.node");
  fs.writeFileSync(alternate, "second inert text library");
  const result = fixture.run(`require(${JSON.stringify(alternate)})`);
  assert.equal(result.status, 1);
  assert.match(result.stderr, /Vize native loader escaped/u);
  assert.deepEqual(
    fixture.events().map((event) => event.event),
    ["initialized", "attempt", "rejected", "exit"],
  );
});

test("native hook refuses existing journals, wrong digest, escaped paths and overrides", (t) => {
  const fixture = inertHookFixture();
  t.after(() => fs.rmSync(fixture.temporary, { recursive: true, force: true }));
  fs.writeFileSync(fixture.journal, "preserved journal");
  assert.equal(fixture.run("void 0").status, 1);
  assert.equal(fs.readFileSync(fixture.journal, "utf8"), "preserved journal");
  fs.unlinkSync(fixture.journal);
  fixture.config.nativeSha256 = "0".repeat(64);
  assert.match(fixture.run("void 0").stderr, /digest differs/u);
  fixture.config.nativeSha256 = digest(fs.readFileSync(fixture.native));
  const escaped = path.join(fixture.temporary, "escaped.node");
  fs.copyFileSync(fixture.native, escaped);
  fixture.config.nativePath = escaped;
  assert.match(fixture.run("void 0").stderr, /escaped the registry installation/u);
  fixture.config.nativePath = fixture.native;
  for (const key of overrideKeys.filter((key) => key !== "VIZE_PUBLIC_NATIVE_CUSTODY")) {
    const result = fixture.run("void 0", {
      [key]: key === "NODE_OPTIONS" ? "--no-warnings" : "forbidden",
    });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /Initial source\/native override must be empty/u);
    assert.equal(fs.existsSync(fixture.journal), false);
  }
});
