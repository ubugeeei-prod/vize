/** One source-role Q16 baseline. All commands retain raw bytes before verdicts. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { pathToFileURL } from "node:url";

import {
  q16Package,
  declaredBin,
  verifyQ16Lock,
  verifyQ16PublishedLauncher,
} from "./reported-q16-provider.ts";
export {
  q16Package,
  declaredBin,
  verifyQ16Lock,
  verifyQ16PublishedLauncher,
  q16PublishedLauncherPin,
} from "./reported-q16-provider.ts";

export const q16Cases = [
  [
    "model",
    "content_mapper_tsgo_cli::reported_model_props_11359::resolved_model_props_preserve_authored_contract_and_assignment_errors",
  ],
  [
    "generic",
    "content_mapper_tsgo_cli::reported_generic_components::reported_generic_slots_and_global_refs_keep_authored_members",
  ],
  [
    "dynamic",
    "content_mapper_tsgo_cli::reported_dynamic_targets::originals_and_error_repair_restore_keep_component_target_types",
  ],
  [
    "nested",
    "content_mapper_tsgo_cli::nested_callback_11361::nested_callback_keeps_original_acceptance_and_checked_body_errors",
  ],
  [
    "declaration-unused",
    "check_reported_declarations_11364_11365_cli::repeated_component_declarations_preserve_no_unused_and_authored_errors",
  ],
  [
    "declaration-fallthrough",
    "check_reported_declarations_11364_11365_cli::emitted_component_declarations_accept_fallthrough_and_reject_owned_prop_errors",
  ],
] as const;
export function verifyQ16Inputs(e: NodeJS.ProcessEnv, actual: string) {
  assert.equal(e.GITHUB_EVENT_NAME, "workflow_dispatch");
  assert.equal(e.Q16_TARGETED_TEST, "reported-q16");
  assert.equal(e.Q16_OTHER_MODES, "false");
  for (const key of ["CUT", "HEAD", "TAG", "SOURCE_PR", "RELEASE_RUN"])
    assert.equal(e[`Q16_PUBLIC_${key}`] || "", "", "public receiver inputs are separate");
  assert.match(e.EXPECTED_SOURCE_SHA || "", /^[a-f0-9]{40}$/u);
  for (const source of [actual, e.GITHUB_SHA, e.GITHUB_WORKFLOW_SHA])
    assert.equal(source, e.EXPECTED_SOURCE_SHA);
  for (const key of ["VIZE_TEST_DISABLE_TSGO", "NODE_OPTIONS", "NODE_PATH"])
    assert.equal(e[key], undefined, `unexpected inherited ${key}`);
}
export function verifyQ16List(text: string) {
  const lines = text.split(/\r?\n/u);
  for (const [, name] of q16Cases)
    assert.equal(
      lines.filter((line) => line === `${name}: test`).length,
      1,
      `one compiled test required: ${name}`,
    );
}
export function verifyQ16Execution(text: string, status: number | null) {
  assert(/(?:^|\n)running 1 test\r?\n/u.test(text), "test filter must execute exactly one test");
  assert(/test result: (?:ok|FAILED)\. [01] passed; [01] failed; 0 ignored;/u.test(text));
  return status === 0 && /test result: ok\. 1 passed; 0 failed; 0 ignored;/u.test(text);
}
export function verifyQ16RequiredEnvironment(e: NodeJS.ProcessEnv) {
  for (const key of [
    "VIZE_TEST_REQUIRE_TSGO",
    "VIZE_REQUIRE_REPORTED_GENERIC_COMPONENTS",
    "VIZE_TEST_REPORTED_DYNAMIC_TARGETS_REQUIRED",
    "VIZE_TEST_REQUIRE_NESTED_CALLBACK",
  ])
    assert.equal(e[key], "1", key);
  for (const key of [
    "CORSA_PATH",
    "VIZE_TEST_CONTENT_MAPPER_TSGO",
    "VIZE_TEST_CONTENT_MAPPER_VUE",
    "VIZE_REPORTED_MODEL_CAPTURE",
    "VIZE_REPORTED_GENERIC_EVIDENCE_DIR",
    "VIZE_TEST_REPORTED_DYNAMIC_TARGETS_CAPTURE_DIR",
    "VIZE_NESTED_CALLBACK_CAPTURE",
    "VIZE_REPORTED_DECLARATIONS_CAPTURE",
  ])
    assert(path.isAbsolute(e[key] || ""), key);
  assert.equal(e.VIZE_TEST_DISABLE_TSGO, undefined);
}
const hash = (b: Buffer) => createHash("sha256").update(b).digest("hex");
const read = (p: string) => JSON.parse(fs.readFileSync(p, "utf8"));
function save(p: string, value: unknown) {
  fs.writeFileSync(p, JSON.stringify(value, null, 2) + "\n", { flag: "wx" });
}
function identity(file: string) {
  const actual = fs.realpathSync(file),
    stat = fs.statSync(actual),
    bytes = fs.readFileSync(actual);
  assert(stat.isFile());
  return {
    path: actual,
    bytes: bytes.length,
    sha256: hash(bytes),
    device: stat.dev,
    inode: stat.ino,
  };
}
function record(
  out: string,
  name: string,
  program: string,
  args: string[],
  cwd: string,
  env = process.env,
) {
  const directory = path.join(out, name);
  fs.mkdirSync(directory);
  save(path.join(directory, "request.json"), {
    program,
    args,
    cwd,
    source: env.GITHUB_SHA,
    run: env.GITHUB_RUN_ID,
    attempt: env.GITHUB_RUN_ATTEMPT,
    environment: Object.fromEntries(
      [
        "PATH",
        "NODE_OPTIONS",
        "NODE_PATH",
        "CORSA_PATH",
        "VIZE_TEST_CONTENT_MAPPER_TSGO",
        "VIZE_TEST_CONTENT_MAPPER_VUE",
        "VIZE_TEST_REQUIRE_TSGO",
        "VIZE_REQUIRE_REPORTED_GENERIC_COMPONENTS",
        "VIZE_TEST_REPORTED_DYNAMIC_TARGETS_REQUIRED",
        "VIZE_TEST_REQUIRE_NESTED_CALLBACK",
        "GOGC",
        "GOMEMLIMIT",
        "GOMAXPROCS",
      ].map((k) => [k, env[k] ?? null]),
    ),
  });
  const result = spawnSync(program, args, {
    cwd,
    env,
    timeout: 1_800_000,
    maxBuffer: 128 * 1024 ** 2,
    stdio: ["ignore", "pipe", "pipe"],
  });
  for (const stream of ["stdout", "stderr"] as const)
    if (result[stream] !== null)
      fs.writeFileSync(path.join(directory, `${stream}.bin`), result[stream]!, { flag: "wx" });
  save(path.join(directory, "process.json"), {
    status: result.status,
    signal: result.signal,
    pid: result.pid ?? null,
    error: result.error
      ? {
          name: result.error.name,
          message: result.error.message,
          code: (result.error as NodeJS.ErrnoException).code ?? null,
        }
      : null,
    streams: Object.fromEntries(
      ["stdout", "stderr"].map((k) => {
        const b = result[k as "stdout" | "stderr"];
        return [k, b === null ? null : { bytes: b.length, sha256: hash(b) }];
      }),
    ),
  });
  return result;
}
function success(result: ReturnType<typeof record>) {
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0);
  assert(result.stdout !== null && result.stderr !== null);
  return result.stdout;
}
function manifests(root: string, out: string) {
  const rows: { path: string; bytes: number; sha256: string; body: string }[] = [],
    pending = [root];
  let bytes = 0;
  while (pending.length)
    for (const e of fs.readdirSync(pending.pop()!, { withFileTypes: true })) {
      const p = path.join(e.parentPath, e.name);
      if (e.isDirectory()) pending.push(p);
      else if (e.isFile() && e.name === "package.json") {
        const b = fs.readFileSync(p);
        bytes += b.length;
        assert(bytes <= 8 * 1024 ** 2);
        rows.push({ path: p, bytes: b.length, sha256: hash(b), body: b.toString("utf8") });
      }
    }
  save(path.join(out, "installed-manifests.json"), rows);
}
export async function runQ16(stage: string) {
  assert(["prepare", "build", "provider", "cases"].includes(stage));
  const out = process.env.Q16_OUT!,
    provider = process.env.Q16_PROVIDER!,
    root = process.cwd();
  assert(path.isAbsolute(out) && path.isAbsolute(provider));
  fs.mkdirSync(out, { recursive: true });
  const git = record(out, `${stage}-git`, "git", ["rev-parse", "HEAD", "HEAD^{tree}"], root);
  const ids = success(git).toString("utf8").trim().split("\n");
  verifyQ16Inputs(process.env, ids[0]);
  save(path.join(out, `${stage}-node.json`), {
    node: identity(process.execPath),
    version: process.version,
    platform: process.platform,
    arch: process.arch,
    source: ids[0],
    tree: ids[1],
  });
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  if (stage === "prepare") {
    fs.mkdirSync(provider);
    fs.writeFileSync(
      path.join(provider, "package.json"),
      JSON.stringify(q16Package, null, 2) + "\n",
      { flag: "wx" },
    );
    save(path.join(out, "scope.json"), {
      source: ids[0],
      tree: ids[1],
      reported: { vize: "0.638.0", node: "26.13.0", platform: "Windows 11", lock: "not supplied" },
      actual: {
        role: "current source CLI + source NAPI companion, published report host/oracles",
        platform: "Linux x64",
        generatedLock: true,
        publishedVizeInstalled: false,
      },
      nativeSourceRevision: "unobserved for published tsrs; not borrowed d6c/a183",
      noWindowsQualification: true,
    });
    return;
  }
  if (stage === "build") {
    success(
      record(out, "source-napi-build", "vp", ["run", "--filter", "./npm/native", "build:ci"], root),
    );
    success(
      record(
        out,
        "source-cli-build",
        "cargo",
        ["build", "--locked", "--profile", "ci", "-p", "vize", "--bin", "vize"],
        root,
      ),
    );
    save(path.join(out, "source-images.json"), {
      source: ids[0],
      tree: ids[1],
      cli: identity(path.join(root, "target/ci/vize")),
      napi: identity(path.join(root, "npm/native/vize-vitrine.linux-x64-gnu.node")),
      activeMapper: "Rust fixture manifest executes CARGO_BIN_EXE_vize content-mapper",
      napiRole: "built source companion; not claimed loaded by these Rust CLI adapters",
      cargo: identity(path.join(root, "Cargo.lock")),
    });
    return;
  }
  assert.equal(process.version, "v26.13.0");
  if (stage === "provider") {
    const lock = read(path.join(provider, "package-lock.json"));
    save(path.join(out, "generated-lock.json"), lock);
    manifests(path.join(provider, "node_modules"), out);
    verifyQ16Lock(lock);
    const packages: Record<string, unknown> = {};
    for (const [name, version] of Object.entries(q16Package.dependencies)) {
      const p = read(path.join(provider, "node_modules", name, "package.json"));
      packages[name] = p;
      assert.equal(p.name, name);
      assert.equal(p.version, version);
    }
    const manifest = read(path.join(provider, "node_modules/tsrs/package.json"));
    const requested = path.join(provider, "node_modules/tsrs", declaredBin(manifest, "tsrs"));
    const bin = identity(requested),
      shim = fs.realpathSync(path.join(provider, "node_modules/.bin/tsrs"));
    const bytes = fs.readFileSync(bin.path),
      manifestBytes = fs.readFileSync(path.join(provider, "node_modules/tsrs/package.json"));
    save(path.join(out, "launcher-before-version.json"), {
      bin,
      shim,
      mapping: manifest.bin,
      launcherSourceBase64: bytes.toString("base64"),
      manifestSourceBase64: manifestBytes.toString("base64"),
      integrity: lock.packages["node_modules/tsrs"].integrity,
    });
    assert.equal(shim, bin.path);
    const launcherRole = verifyQ16PublishedLauncher(
      bytes,
      manifestBytes,
      declaredBin(manifest, "tsrs"),
      lock.packages["node_modules/tsrs"].integrity,
    );
    const version = record(out, "published-tsrs-version", bin.path, ["--version"], provider);
    save(path.join(out, "provider.json"), {
      source: ids[0],
      provider,
      packages,
      generatedLock: identity(path.join(provider, "package-lock.json")),
      bin,
      ...launcherRole,
      physicalNative: null,
      launcherSourceBase64: bytes.toString("base64"),
      node: identity(process.execPath),
      nodeVersion: process.version,
      publishedCliVersionCompanion: {
        status: version.status,
        signal: version.signal,
        pid: version.pid ?? null,
      },
      noOriginalLockClaim: true,
      noD6SourceTransfer: true,
      noPublishedMapperFallback: !fs.existsSync(
        path.join(provider, "node_modules/vize/package.json"),
      ),
    });
    success(version);
    assert(!fs.existsSync(path.join(provider, "node_modules/vize/package.json")));
    return;
  }
  const host = read(path.join(out, "provider.json"));
  assert.equal(host.binRole, "source-closed standard published JavaScript launcher");
  const source = read(path.join(out, "source-images.json"));
  assert.equal(source.source, ids[0]);
  const env = {
    ...process.env,
    CORSA_PATH: host.bin.path,
    VIZE_TEST_REQUIRE_TSGO: "1",
    VIZE_TEST_CONTENT_MAPPER_TSGO: host.bin.path,
    VIZE_TEST_CONTENT_MAPPER_VUE: path.join(provider, "node_modules/vue"),
    VIZE_TEST_CONTENT_MAPPER_JAVASCRIPT_TSC: path.join(provider, "node_modules/.bin/tsc"),
    VIZE_REQUIRE_REPORTED_GENERIC_COMPONENTS: "1",
    VIZE_TEST_REPORTED_DYNAMIC_TARGETS_REQUIRED: "1",
    VIZE_TEST_REQUIRE_NESTED_CALLBACK: "1",
    VIZE_REPORTED_MODEL_CAPTURE: path.join(out, "cases/model"),
    VIZE_REPORTED_GENERIC_EVIDENCE_DIR: path.join(out, "cases/generic"),
    VIZE_TEST_REPORTED_DYNAMIC_TARGETS_CAPTURE_DIR: path.join(out, "cases/dynamic"),
    VIZE_NESTED_CALLBACK_CAPTURE: path.join(out, "cases/nested"),
    VIZE_REPORTED_DECLARATIONS_CAPTURE: path.join(out, "cases/declarations"),
  };
  verifyQ16RequiredEnvironment(env);
  assert.equal(identity(host.bin.path).sha256, host.bin.sha256);
  assert.equal(identity(source.cli.path).sha256, source.cli.sha256);
  assert.equal(identity(source.napi.path).sha256, source.napi.sha256);
  const testModules = path.join(root, "tests/node_modules"),
    backup = path.join(path.dirname(out), "reported-q16-workspace-tests-node_modules");
  const previous = fs.lstatSync(testModules, { throwIfNoEntry: false });
  assert.equal(fs.lstatSync(backup, { throwIfNoEntry: false }), undefined);
  if (previous) fs.renameSync(testModules, backup);
  fs.symlinkSync(path.join(provider, "node_modules"), testModules, "dir");
  try {
    const metadata = JSON.parse(
      success(
        record(
          out,
          "cargo-package-metadata",
          "cargo",
          ["metadata", "--no-deps", "--format-version", "1"],
          root,
          env,
        ),
      ).toString("utf8"),
    );
    const pkg = metadata.packages.filter((p: { name: string }) => p.name === "vize");
    assert.equal(pkg.length, 1);
    const compiled = success(
      record(
        out,
        "cargo-compile-reported-tests",
        "cargo",
        [
          "test",
          "--locked",
          "--profile",
          "ci",
          "-p",
          "vize",
          "--test",
          "integration",
          "--no-run",
          "--message-format=json",
        ],
        root,
        env,
      ),
    );
    const artifacts = compiled
      .toString("utf8")
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line))
      .filter(
        (p) =>
          p.reason === "compiler-artifact" &&
          p.package_id === pkg[0].id &&
          p.target.name === "integration" &&
          p.target.kind.includes("test") &&
          p.profile.test &&
          p.executable,
      );
    assert.equal(artifacts.length, 1);
    const executable = artifacts[0].executable;
    save(path.join(out, "compiled-integration.json"), {
      source: ids[0],
      artifact: artifacts[0],
      executable: identity(executable),
      cli: identity(source.cli.path),
    });
    verifyQ16List(
      success(
        record(out, "compiled-test-list", executable, ["--list", "--format", "terse"], root, env),
      ).toString("utf8"),
    );
    const results = [];
    for (const [id, name] of q16Cases) {
      const result = record(
        out,
        `cargo-case-${id}`,
        executable,
        [name, "--exact", "--nocapture", "--test-threads=1", "--color", "never"],
        root,
        env,
      );
      let passed = false,
        refusal: string | null = null;
      try {
        assert.equal(result.error, undefined);
        assert.equal(result.signal, null);
        assert(result.stdout !== null);
        passed = verifyQ16Execution(result.stdout.toString("utf8"), result.status);
      } catch (e) {
        refusal = String(e);
      }
      results.push({ id, name, status: result.status, signal: result.signal, passed, refusal });
      save(path.join(out, `case-${id}-post-images.json`), {
        cli: identity(source.cli.path),
        publishedLauncher: identity(host.bin.path),
        napi: identity(source.napi.path),
      });
    }
    save(path.join(out, "all-six-results.json"), {
      source: ids[0],
      results,
      complete: results.length === 6,
      knownBugIsNotSuccess: true,
    });
    assert.equal(identity(source.cli.path).sha256, source.cli.sha256);
    assert.equal(identity(host.bin.path).sha256, host.bin.sha256);
    assert(
      results.length === 6 && results.every((r) => r.passed),
      "one or more genuine baseline laws failed; all six raw results retained",
    );
  } finally {
    assert.equal(fs.readlinkSync(testModules), path.join(provider, "node_modules"));
    fs.unlinkSync(testModules);
    if (previous) fs.renameSync(backup, testModules);
  }
}
if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  runQ16(process.argv[2]).catch((error) => {
    process.stderr.write(String(error) + "\n");
    process.exitCode = 1;
  });
}
