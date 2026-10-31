// Sequential source replay after every latency/profile/RSS observation.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { cleanEnvironment, inventory, readAuthority, sha256 } from "./patina-project-batch-lib.mjs";
import { assertPlainTypeRuntimeCounts } from "./patina-plain-type-runtime-counts.ts";
import {
  LEGACY_PACKET_REVISION,
  LEGACY_PACKET_SHA,
  PACKET_MODULE_SUFFIX,
  planPacketReplay,
  verifyPacketTransition,
} from "./patina-plain-type-contracts.ts";

export const TEST_PARENT = "crates/vize_patina/src/linter/native_type_aware/plain_module_tests.rs";
export const TEST_CHILD =
  "crates/vize_patina/src/linter/native_type_aware/plain_module_tests/packets.rs";
export const TEST_SUFFIX = PACKET_MODULE_SUFFIX;
export const TEST_NAME =
  "linter::native_type_aware::plain_module_tests::packets::conditional_plain_type_packets_are_complete";
export const PROVIDER_SHA = "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4";
export const INPUT_ROOT = "tests/_fixtures/differential/linter/conditional-plain-types-10517";

export function readPacketInputs(root) {
  const manifest = JSON.parse(
    fs.readFileSync(path.join(root, "tests/_fixtures/differential/linter/manifest.json")),
  );
  const entry = manifest.cases.find(
    (row) => row.id === "linter/current-api/conditional-plain-types-10517",
  );
  assert.equal(entry.inputs.root, "conditional-plain-types-10517");
  assert.equal(entry.inputs.files.length, 13);
  const inputs = new Map(
    entry.inputs.files.map((file) => {
      assert.match(
        file.path,
        /^(?:(?:promise|reactivity)(?:-default)?\.(?:tsx|jsx|d\.ts|d\.mts|d\.cts)|no-query\.tsx)\.txt$/u,
      );
      const bytes = fs.readFileSync(path.join(root, INPUT_ROOT, file.path));
      assert.equal(sha256(bytes), file.sha256, `pinned packet input changed: ${file.path}`);
      return [file.path, bytes.toString()];
    }),
  );
  assert.equal(inputs.size, 13, "pinned packet names must be unique");
  return inputs;
}

export function verifyAppendOnly(original, reviewed) {
  const expected = Buffer.concat([original, Buffer.from(TEST_SUFFIX)]);
  assert.deepEqual(reviewed, expected, "original tests must retain every byte");
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
  assert.equal(artifacts.length, 1, "expected one source-built Patina test executable");
  return artifacts[0].executable;
}

export function verifyPackets(bytes, inputs, contract = "runtime-error-v2") {
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
    assertPlainTypeRuntimeCounts(row, contract);
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
    if (inputs) {
      const extension =
        {
          javascript: "jsx",
          "declaration-ts": "d.ts",
          "declaration-mts": "d.mts",
          "declaration-cts": "d.cts",
        }[row.case] ?? "tsx";
      const input =
        row.case === "no-query"
          ? "no-query.tsx.txt"
          : row.case === "default"
            ? `${row.kind}-default.tsx.txt`
            : `${row.kind}.${extension}.txt`;
      assert.equal(row.source, inputs.get(input), `replayed source must match pinned ${input}`);
    }
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
  const git = (root, args, label = args[0]) =>
    run(
      `git-${Object.keys(roots).find((key) => roots[key] === root)}-${label}`,
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
  const legacy = fs.readFileSync(
    path.join(roots.head, "tools/benchmarks/fixtures/patina-plain-type-packets/warning-v1.rs"),
  );
  const replay = {};
  for (const [side, root] of Object.entries(roots)) {
    const revision = side === "base" ? baseSha : headSha;
    const parent = fs.readFileSync(path.join(root, TEST_PARENT));
    assert.deepEqual(parent, git(root, ["show", `${revision}:${TEST_PARENT}`], "original-parent"));
    const tracked =
      git(root, ["ls-tree", revision, "--", TEST_CHILD], "original-child-entry").length > 0;
    assert.equal(
      fs.existsSync(path.join(root, TEST_CHILD)),
      tracked,
      "packet test must be original source",
    );
    const child = tracked ? fs.readFileSync(path.join(root, TEST_CHILD)) : undefined;
    if (child)
      assert.deepEqual(child, git(root, ["show", `${revision}:${TEST_CHILD}`], "original-child"));
    const plan = planPacketReplay(parent, child, legacy);
    if (side === "head")
      assert.equal(plan.overlay, false, "head must carry its own original packet test");
    if (plan.overlay) verifyAppendOnly(parent, plan.parent);
    replay[side] = {
      ...plan,
      originalParentSha256: sha256(parent),
      originalChildSha256: child ? sha256(child) : null,
    };
    log(`${side}-original-tests.rs`, parent);
    log(`${side}-replayed-tests.rs`, plan.parent);
    log(`${side}-packet-test.rs`, plan.child);
  }
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
  const packetInputs = readPacketInputs(roots.head);
  log("pinned-inputs.json", JSON.stringify(Object.fromEntries(packetInputs), null, 2));
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
  log("legacy-warning-packets.rs", legacy);
  const provenance = {
    baseSha,
    headSha,
    provider: { version: providerPackage.version, sha256: PROVIDER_SHA, path: provider },
    originalWarningAuthority: { revision: LEGACY_PACKET_REVISION, sha256: LEGACY_PACKET_SHA },
    replay: Object.fromEntries(
      Object.entries(replay).map(([side, plan]) => [
        side,
        {
          contract: plan.contract,
          overlay: plan.overlay,
          parent: TEST_PARENT,
          originalParentSha256: plan.originalParentSha256,
          replayedParentSha256: sha256(plan.parent),
          child: TEST_CHILD,
          originalChildSha256: plan.originalChildSha256,
          replayedChildSha256: sha256(plan.child),
        },
      ]),
    ),
    profile: "ci-opt-release-thin-lto-cgu16",
    executables: {},
  };
  log("provenance.json", JSON.stringify(provenance, null, 2));
  run("rustc", "rustc", ["--version", "--verbose"], roots.head, env);
  if (replay.base.overlay) {
    fs.appendFileSync(path.join(roots.base, TEST_PARENT), TEST_SUFFIX);
    fs.mkdirSync(path.dirname(path.join(roots.base, TEST_CHILD)), { recursive: true });
    fs.writeFileSync(path.join(roots.base, TEST_CHILD), replay.base.child, { flag: "wx" });
  }
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
          VIZE_PLAIN_TYPE_PACKETS_INPUT_DIR: path.join(roots.head, INPUT_ROOT),
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
    verifyPackets(packets, packetInputs, replay[side].contract);
    log(`${side}-packets.json`, packets);
    assert.deepEqual(fs.readFileSync(path.join(root, TEST_PARENT)), replay[side].parent);
    assert.deepEqual(fs.readFileSync(path.join(root, TEST_CHILD)), replay[side].child);
    const changed = git(root, ["diff", "--name-only"])
      .toString()
      .trim()
      .split("\n")
      .filter(Boolean);
    assert.deepEqual(changed, replay[side].overlay ? [TEST_PARENT] : []);
    const untracked = git(root, ["ls-files", "--others", "--exclude-standard"])
      .toString()
      .trim()
      .split("\n")
      .filter(Boolean);
    assert.deepEqual(untracked, replay[side].overlay ? [TEST_CHILD] : []);
  }
  const comparison = verifyPacketTransition(
    fs.readFileSync(path.join(output, "base-packets.json")),
    fs.readFileSync(path.join(output, "head-packets.json")),
    replay.base.contract,
    replay.head.contract,
    packetInputs,
    consumer,
  );
  log("packet-comparison.json", JSON.stringify(comparison, null, 2));
  assert.equal(sha256(fs.readFileSync(provider)), PROVIDER_SHA);
  const after = inventory(consumer, ["node_modules", "package-lock.json"]);
  log("dependencies-after.json", JSON.stringify(after, null, 2));
  assert.deepEqual(after, inputs);
  log(
    "summary.md",
    `## Conditional plain-module native packets\n\nAll twenty ordered packets are retained at \`${baseSha}\` → \`${headSha}\`, using TypeScript 7.0.2 (\`${PROVIDER_SHA}\`). ${comparison.unchanged} complete cells are byte-identical; ${comparison.changed} unavailable cells follow the explicit \`${comparison.baseContract}\` → \`${comparison.headContract}\` contract. Full before/after unavailable packets and every cell hash are in packet-comparison.json. Each revision keeps its original tests; only a baseline with no packet child receives the authenticated original warning test append-only. Source builds and tests ran sequentially after all benchmark observations. No performance equivalence is implied by the intentional diagnostic migration.\n`,
  );
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  main(process.argv.slice(2));
