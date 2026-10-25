// Sequential source replay after every latency/profile/RSS observation.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { cleanEnvironment, inventory, readAuthority, sha256 } from "./patina-project-batch-lib.mjs";

export const TEST_PARENT = "crates/vize_patina/src/linter/native_type_aware/plain_module_tests.rs";
export const TEST_CHILD =
  "crates/vize_patina/src/linter/native_type_aware/plain_module_tests/packets.rs";
export const TEST_SUFFIX = "\nmod packets;\n";
export const TEST_NAME =
  "linter::native_type_aware::plain_module_tests::packets::conditional_plain_type_packets_are_complete";
export const PROVIDER_SHA = "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4";

export function verifyAppendOnly(original, reviewed) {
  assert.deepEqual(
    reviewed,
    Buffer.concat([original, Buffer.from(TEST_SUFFIX)]),
    "reviewed test parent must preserve every original test byte and append only its module declaration",
  );
}

export function testExecutable(stdout) {
  const artifacts = stdout
    .split(/\r?\n/u)
    .filter(Boolean)
    .map((line) => JSON.parse(line))
    .filter(
      (row) =>
        row.reason === "compiler-artifact" &&
        row.target?.name === "vize_patina" &&
        row.profile?.test &&
        row.executable,
    );
  assert.equal(
    artifacts.length,
    1,
    "exactly one source-built Patina unit-test executable is required",
  );
  return artifacts[0].executable;
}

export function verifyPackets(bytes) {
  const document = JSON.parse(bytes);
  assert.equal(document.provider_version, "7.0.2");
  assert.equal(document.packets.length, 20);
  const cases = [
    "active",
    "javascript",
    "declaration-ts",
    "declaration-mts",
    "declaration-cts",
    "default",
    "host-disabled",
    "rule-disabled",
    "unavailable",
    "no-query",
  ];
  const expected = ["promise", "reactivity"].flatMap((kind) =>
    cases.map((name) => `${kind}/${name}`),
  );
  assert.deepEqual(
    document.packets.map((row) => `${row.kind}/${row.case}`),
    expected,
  );
  for (const row of document.packets) {
    assert.deepEqual(row.actual, row.expected, `${row.kind}/${row.case} whole packet changed`);
    assert.deepEqual(Object.keys(row.actual).sort(), [
      "diagnostics",
      "error_count",
      "filename",
      "warning_count",
    ]);
    assert.equal(row.actual.error_count, 0);
    assert.equal(row.actual.warning_count, row.actual.diagnostics.length);
    const rule =
      row.case === "active"
        ? `type/${row.kind === "promise" ? "no-floating-promises" : "no-reactivity-loss"}`
        : row.case === "unavailable"
          ? "type/corsa-runtime"
          : undefined;
    assert.deepEqual(
      row.actual.diagnostics.map((d) => d.rule_name),
      rule ? [rule] : [],
    );
    for (const diagnostic of row.actual.diagnostics) {
      assert.deepEqual(Object.keys(diagnostic).sort(), [
        "end",
        "fix",
        "help",
        "labels",
        "message",
        "rule_name",
        "severity",
        "start",
      ]);
    }
    assert.equal(typeof row.source, "string");
  }
  return document;
}

function main([baseInput, headInput, baseSha, headSha, outputInput]) {
  assert.ok([baseInput, headInput, outputInput].every(Boolean));
  assert.match(baseSha, /^[a-f0-9]{40}$/u);
  assert.match(headSha, /^[a-f0-9]{40}$/u);
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  const roots = { base: fs.realpathSync(baseInput), head: fs.realpathSync(headInput) };
  const output = path.resolve(outputInput);
  assert.ok(!fs.existsSync(output));
  fs.mkdirSync(output, { recursive: true });
  const log = (name, value) => fs.writeFileSync(path.join(output, name), value);
  const run = (name, command, args, cwd, env) => {
    const result = spawnSync(command, args, {
      cwd,
      env,
      timeout: 600_000,
      maxBuffer: 128 * 1024 * 1024,
    });
    log(`${name}.stdout`, result.stdout ?? Buffer.alloc(0));
    log(`${name}.stderr`, result.stderr ?? Buffer.alloc(0));
    log(
      `${name}.process.json`,
      JSON.stringify(
        { status: result.status, signal: result.signal, error: result.error?.message },
        null,
        2,
      ),
    );
    assert.equal(result.error, undefined, `${name}: ${result.error?.message}`);
    assert.equal(result.signal, null, name);
    assert.equal(result.status, 0, `${name}: inspect complete retained stderr`);
    return result.stdout;
  };
  const git = (root, args) =>
    run(
      `git-${Object.keys(roots).find((key) => roots[key] === root)}-${args[0]}`,
      "git",
      args,
      root,
      process.env,
    );
  for (const [side, root] of Object.entries(roots)) {
    assert.equal(
      git(root, ["rev-parse", "HEAD"]).toString().trim(),
      side === "base" ? baseSha : headSha,
    );
    assert.equal(
      git(root, ["status", "--porcelain", "--untracked-files=no"]).length,
      0,
      "source checkout must start clean",
    );
  }
  const original = fs.readFileSync(path.join(roots.base, TEST_PARENT));
  const reviewed = fs.readFileSync(path.join(roots.head, TEST_PARENT));
  const child = fs.readFileSync(path.join(roots.head, TEST_CHILD));
  verifyAppendOnly(original, reviewed);
  assert.ok(
    !fs.existsSync(path.join(roots.base, TEST_CHILD)),
    "baseline must not already contain the replay test",
  );
  const provider = fs.realpathSync(
    path.join(roots.head, "node_modules/@typescript/typescript-linux-x64/lib/tsc"),
  );
  const providerPackage = JSON.parse(
    fs.readFileSync(path.join(path.dirname(provider), "../package.json")),
  );
  assert.equal(providerPackage.version, "7.0.2");
  assert.equal(sha256(fs.readFileSync(provider)), PROVIDER_SHA);
  const env = {
    ...cleanEnvironment(process.env, provider),
    VIZE_TEST_REQUIRE_TSGO: "1",
    VIZE_PLAIN_TYPE_PACKETS_REQUIRED: "1",
    VIZE_PLAIN_TYPE_PACKETS_CORSA_PATH: provider,
  };
  const fixture = path.join(roots.head, "tools/benchmarks/fixtures/patina-project-batch");
  readAuthority(fixture);
  const consumer = path.join(output, "consumer");
  fs.mkdirSync(consumer);
  for (const name of ["package.json", "package-lock.json"])
    fs.copyFileSync(
      path.join(fixture, name === "package.json" ? `${name}.txt` : name),
      path.join(consumer, name),
    );
  run(
    "dependency-install",
    "npm",
    ["ci", "--ignore-scripts", "--no-audit", "--no-fund"],
    consumer,
    env,
  );
  assert.equal(
    JSON.parse(fs.readFileSync(path.join(consumer, "node_modules/vue/package.json"))).version,
    "3.5.44",
  );
  const inputs = inventory(consumer, ["node_modules", "package-lock.json"]);
  log("dependencies-before.json", JSON.stringify(inputs, null, 2));
  log("reviewed-packets.rs", child);
  log("original-tests.rs", original);
  const provenance = {
    baseSha,
    headSha,
    provider: { version: providerPackage.version, sha256: PROVIDER_SHA, path: provider },
    overlay: {
      parent: TEST_PARENT,
      originalSha256: sha256(original),
      appended: TEST_SUFFIX,
      child: TEST_CHILD,
      childSha256: sha256(child),
    },
    profile: "ci-opt-release-thin-lto-cgu16",
    executables: {},
  };
  log("provenance.json", JSON.stringify(provenance, null, 2));
  run("rustc", "rustc", ["--version", "--verbose"], roots.head, env);
  fs.appendFileSync(path.join(roots.base, TEST_PARENT), TEST_SUFFIX);
  fs.mkdirSync(path.dirname(path.join(roots.base, TEST_CHILD)), { recursive: true });
  fs.writeFileSync(path.join(roots.base, TEST_CHILD), child, { flag: "wx" });
  for (const [side, root] of Object.entries(roots)) {
    const capture = path.join(output, side);
    fs.mkdirSync(capture);
    const buildEnv = { ...env, CARGO_TARGET_DIR: path.join(root, "target") };
    const cargo = run(
      `${side}-build`,
      "cargo",
      [
        "test",
        "--locked",
        "--manifest-path",
        path.join(root, "Cargo.toml"),
        "--profile",
        "ci-opt",
        "-p",
        "vize_patina",
        "--lib",
        TEST_NAME,
        "--no-run",
        "--message-format=json",
        "--config",
        'profile.ci-opt.inherits="release"',
        "--config",
        'profile.ci-opt.lto="thin"',
        "--config",
        "profile.ci-opt.codegen-units=16",
      ],
      root,
      buildEnv,
    );
    const executable = fs.realpathSync(testExecutable(cargo.toString()));
    provenance.executables[side] = {
      path: executable,
      sha256: sha256(fs.readFileSync(executable)),
    };
    log("provenance.json", JSON.stringify(provenance, null, 2));
    let stdout;
    try {
      stdout = run(
        `${side}-test`,
        executable,
        [TEST_NAME, "--exact", "--nocapture", "--test-threads=1", "--color", "never"],
        consumer,
        {
          ...env,
          VIZE_PLAIN_TYPE_PACKETS_CASE_ROOT: consumer,
          VIZE_PLAIN_TYPE_PACKETS_CAPTURE_DIR: capture,
        },
      );
    } finally {
      const retained = path.join(capture, "packets.json");
      if (fs.existsSync(retained)) log(`${side}-packets.json`, fs.readFileSync(retained));
    }
    assert.match(stdout.toString(), /^test result: ok\. 1 passed; 0 failed; 0 ignored;/mu);
    assert.ok(stdout.toString().includes(`test ${TEST_NAME} ... ok`));
    const packets = fs.readFileSync(path.join(capture, "packets.json"));
    verifyPackets(packets);
    log(`${side}-packets.json`, packets);
    assert.deepEqual(fs.readFileSync(path.join(root, TEST_PARENT)), reviewed);
    assert.deepEqual(fs.readFileSync(path.join(root, TEST_CHILD)), child);
    const changed = git(root, ["diff", "--name-only"])
      .toString()
      .trim()
      .split("\n")
      .filter(Boolean);
    assert.deepEqual(changed, side === "base" ? [TEST_PARENT] : []);
    const untracked = git(root, ["ls-files", "--others", "--exclude-standard"])
      .toString()
      .trim()
      .split("\n")
      .filter(Boolean);
    assert.deepEqual(untracked, side === "base" ? [TEST_CHILD] : []);
  }
  assert.deepEqual(
    fs.readFileSync(path.join(output, "base-packets.json")),
    fs.readFileSync(path.join(output, "head-packets.json")),
    "the entire ordered native packet documents must stay byte-identical across sources",
  );
  assert.equal(sha256(fs.readFileSync(provider)), PROVIDER_SHA);
  const after = inventory(consumer, ["node_modules", "package-lock.json"]);
  log("dependencies-after.json", JSON.stringify(after, null, 2));
  assert.deepEqual(after, inputs);
  log(
    "summary.md",
    `## Conditional plain-module native packets\n\nTwenty full ordered packets are byte-identical at \`${baseSha}\` → \`${headSha}\`, using TypeScript 7.0.2 (\`${PROVIDER_SHA}\`). Original tests and production files retain their bytes; the reviewed new test is an explicit append-only overlay on the baseline. Source builds and tests ran sequentially after all benchmark observations.\n`,
  );
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  main(process.argv.slice(2));
