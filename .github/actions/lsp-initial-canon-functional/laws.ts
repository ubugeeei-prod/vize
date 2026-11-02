import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { SDK, SOURCE_PINS } from "./source-pins.ts";

export type Obj = Record<string, unknown>;
export const FIXTURE_PATH =
  "tests/_fixtures/differential/typechecker/original-diagnosing-process-7698/input.json";
export function admittedFixture(bytes: Uint8Array): Obj {
  assert.equal(
    createHash("sha256").update(bytes).digest("hex"),
    SOURCE_PINS[FIXTURE_PATH],
    "whole compile-consumed original-diagnosing-process fixture",
  );
  return object(object(JSON.parse(Buffer.from(bytes).toString("utf8"))).admitted);
}
export const TESTS = [
  {
    label: "factory",
    fqn: "lsp_client::lifecycle::workspace::initial_project_tests::native_first_materialized_editor_matches_existing_activation_and_delta",
    marker: "INITIAL_CANON_PROJECT",
  },
  {
    label: "operations",
    fqn: "corsa_bridge::vue_document::initial_project::native_tests::intervening_native_operations_restore_exact_initial_graph",
    marker: "INITIAL_CANON_OPERATION",
  },
] as const;
export const HOVER = {
  contents: { kind: "markdown", value: "```typescript\nconst support: number\n```\n" },
  range: { start: { line: 0, character: 13 }, end: { line: 0, character: 20 } },
};
export function object(value: unknown): Obj {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Obj;
}
export function checkParent(raw: string, candidate: string): void {
  assert.match(candidate, /^[a-f0-9]{40}$/);
  assert.deepEqual(raw.split(" "), [candidate], "one direct immutable candidate parent");
}
export function keys(value: Obj, expected: readonly string[]): void {
  assert.deepEqual(Object.keys(value).sort(), [...expected].sort());
}
export function childEnv(ambient: NodeJS.ProcessEnv, native: string): NodeJS.ProcessEnv {
  assert.equal(ambient.VIZE_TEST_DISABLE_TSGO, undefined, "DISABLE presence is forbidden");
  return { ...ambient, CORSA_PATH: native, VIZE_TEST_REQUIRE_TSGO: "1" };
}
export function checkAmbient(env: NodeJS.ProcessEnv): void {
  for (const name of Object.keys(env)) {
    if (
      name.startsWith("CARGO_PROFILE_") ||
      [
        "CORSA_PATH",
        "CORSA_EXECUTABLE",
        "VIZE_CORSA_PATH",
        "TSGO_PATH",
        "TSGO_EXECUTABLE",
        "VIZE_TSGO_PATH",
        "TSRS_PATH",
        "TSRS_EXECUTABLE",
        "VIZE_TSRS_PATH",
        "VIZE_TEST_TSGO_PATH",
        "VIZE_TEST_DISABLE_TSGO",
        "RUSTC",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "CARGO_BUILD_RUSTC",
        "CARGO_BUILD_RUSTC_WRAPPER",
        "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
        "CARGO_TARGET_DIR",
      ].includes(name)
    )
      assert.equal(env[name], undefined, `ambient override ${name}`);
  }
  assert.equal(env.RUSTFLAGS ?? "", "");
  assert.equal(env.CARGO_ENCODED_RUSTFLAGS ?? "", "");
  assert.equal(env.CARGO_INCREMENTAL, "0");
  assert.equal(env.CARGO_BUILD_JOBS, "16");
  assert.equal(env.RAYON_NUM_THREADS, "8");
}
export function checkExecution(stdout: string, fqn: string): void {
  const clean = stdout.replace(/\u001b\[[0-9;]*m/g, "");
  assert.deepEqual(
    [...clean.matchAll(/^test (\S+) \.\.\./gm)].map((row) => row[1]),
    [fqn],
  );
  const rows = [
    ...clean.matchAll(
      /test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;/g,
    ),
  ];
  assert.equal(rows.length, 1);
  assert.deepEqual(rows[0]!.slice(1, 6), ["ok", "1", "0", "0", "0"]);
}
export function checkNativeExit(exit: unknown, stderr: string): void {
  assert.deepEqual(exit, { status: 0, signal: null, error: null });
  assert.equal(stderr, "");
}
function packets(stdout: string, marker: string, count: number): Obj[] {
  const tags = [...stdout.matchAll(/(INITIAL_CANON_[A-Z_]+) /g)].map((row) => row[1]);
  assert.deepEqual(
    tags,
    Array.from({ length: count }, () => marker),
  );
  return [...stdout.matchAll(new RegExp(`${marker} ([^\\r\\n]*)`, "g"))].map((row) =>
    object(JSON.parse(row[1]!)),
  );
}
export function checkPackets(
  label: string,
  stdout: string,
  native: string,
  fixture: unknown,
): Obj[] {
  if (label === "factory") {
    const rows = packets(stdout, TESTS[0].marker, 2);
    const expected = object(fixture);
    assert.deepEqual(
      rows.map((row) => row.direct),
      [false, true],
    );
    for (const row of rows) {
      keys(row, ["direct", "native", "packet", "owner", "config", "authoredRoot", "canonRoot"]);
      assert.equal(row.native, native);
      assert.notEqual(row.authoredRoot, row.canonRoot);
      assert.equal(row.config, path.join(String(row.canonRoot), "tsconfig.json"));
      const owner = object(row.owner);
      keys(owner, ["pid", "birth", "executable"]);
      assert.ok(Number.isSafeInteger(owner.pid) && Number(owner.pid) > 0);
      assert.ok(Number.isSafeInteger(owner.birth) && Number(owner.birth) > 0);
      assert.equal(owner.executable, native);
      const packet = object(row.packet);
      keys(packet, [
        "initial",
        "diagnosingProject",
        "bad",
        "repaired",
        "beforeEmpty",
        "afterEmpty",
        "canonSource",
        "canonShared",
        "canonConfig",
      ]);
      assert.equal(object(packet.diagnosingProject).configFilePath, row.config);
      assert.deepEqual(packet.bad, expected.expected);
      assert.deepEqual(packet.repaired, expected.editedExpected);
      assert.deepEqual(packet.beforeEmpty, packet.afterEmpty);
      for (const field of ["canonSource", "canonShared", "canonConfig"])
        assert.ok(
          Array.isArray(packet[field]) &&
            packet[field].every(
              (byte: unknown) => Number.isInteger(byte) && Number(byte) >= 0 && Number(byte) <= 255,
            ),
        );
      assert.deepEqual(packet.canonSource, [...Buffer.from(String(expected.source))]);
      assert.deepEqual(packet.canonShared, [...Buffer.from(String(expected.shared))]);
      assert.deepEqual(
        JSON.parse(Buffer.from(packet.canonConfig as number[]).toString("utf8")),
        expected.config,
      );
    }
    for (const field of [
      "initial",
      "diagnosingProject",
      "bad",
      "repaired",
      "canonSource",
      "canonShared",
      "canonConfig",
    ])
      assert.deepEqual(object(rows[0]!.packet)[field], object(rows[1]!.packet)[field]);
    return rows;
  }
  assert.equal(label, "operations");
  const rows = packets(stdout, TESTS[1].marker, 4);
  assert.deepEqual(
    rows.map((row) => row.operation),
    ["none", "metadata", "otherOpen", "otherConfig"],
  );
  for (const row of rows) {
    keys(row, [
      "operation",
      "native",
      "initialEpoch",
      "beforeConsume",
      "afterConsume",
      "documentUri",
      "supportUri",
      "actual",
      "expected",
    ]);
    assert.equal(row.native, native);
    assert.deepEqual(row.expected, HOVER);
    assert.deepEqual(row.actual, HOVER);
    for (const field of ["initialEpoch", "beforeConsume", "afterConsume"])
      assert.ok(Number.isSafeInteger(row[field]) && Number(row[field]) > 0);
    for (const field of ["documentUri", "supportUri"]) {
      assert.equal(typeof row[field], "string");
      assert.equal(new URL(String(row[field])).protocol, "file:");
    }
    if (row.operation === "none") {
      assert.equal(row.beforeConsume, row.initialEpoch);
      assert.equal(row.afterConsume, Number(row.beforeConsume) + 1);
    } else {
      assert.ok(Number(row.beforeConsume) > Number(row.initialEpoch));
      assert.ok(Number(row.afterConsume) >= Number(row.beforeConsume) + 2);
    }
  }
  return rows;
}
export function sdkArtifacts(rows: Obj[]): Obj[] {
  const artifacts = rows.filter((row) => row.reason === "compiler-artifact");
  assert.ok(
    artifacts.every(
      (row) =>
        Array.isArray(row.features) &&
        !row.features.includes("profiling") &&
        !row.features.includes("micro-profiling"),
    ),
  );
  const sdk = artifacts.filter((row) => Object.hasOwn(SDK, String(object(row.target).name)));
  assert.deepEqual(
    [...new Set(sdk.map((row) => row.package_id))].sort(),
    Object.entries(SDK)
      .map(([name, entry]) => `${entry.source}#${name}@${entry.version}`)
      .sort(),
  );
  assert.ok(sdk.every((row) => JSON.stringify(object(row.target).kind) === '["lib"]'));
  return sdk;
}
export function defaultCliArtifact(rows: Obj[], target: string, repo: string): Obj {
  assert.equal(
    rows.filter((row) => row.reason === "build-finished" && row.success === true).length,
    1,
  );
  const roles = [
    ["vize", "bin", "main.rs", ["default", "glyph", "maestro"]],
    ["vize_canon", "lib", "lib.rs", ["native"]],
    ["vize_maestro", "lib", "lib.rs", ["default", "glyph", "native", "retained-program"]],
  ] as const;
  const selected = roles.map(([name, kind, entry, features]) => {
    const candidates = rows.filter(
      (row) =>
        row.reason === "compiler-artifact" &&
        object(row.target).name === name &&
        JSON.stringify(object(row.target).kind) === JSON.stringify([kind]),
    );
    assert.equal(candidates.length, 1, `one default-release ${name} ${kind} artifact`);
    const row = candidates[0]!;
    const directory = path.join(repo, "crates", name);
    assert.equal(row.manifest_path, path.join(directory, "Cargo.toml"));
    assert.equal(object(row.target).src_path, path.join(directory, "src", entry));
    assert.deepEqual(object(row.target).crate_types, [kind]);
    assert.equal(String(row.package_id).split("#")[0], `path+${pathToFileURL(directory).href}`);
    assert.deepEqual(row.features, [...features]);
    assert.equal(object(row.profile).opt_level, "3");
    assert.equal(object(row.profile).test, false);
    assert.equal(object(row.profile).debug_assertions, false);
    if (kind === "bin") assert.equal(row.executable, path.join(target, "release/vize"));
    else assert.equal(row.executable, null);
    return row;
  });
  sdkArtifacts(rows);
  return selected[0]!;
}
export function libraryArtifact(rows: Obj[], target: string, repo: string): Obj {
  assert.equal(
    rows.filter((row) => row.reason === "build-finished" && row.success === true).length,
    1,
  );
  const candidates = rows.filter(
    (row) =>
      row.reason === "compiler-artifact" &&
      object(row.target).name === "vize_canon" &&
      object(row.profile).test === true &&
      typeof row.executable === "string",
  );
  assert.equal(candidates.length, 1);
  const selected = candidates[0]!;
  assert.deepEqual(object(selected.target).kind, ["lib"]);
  assert.equal(selected.manifest_path, path.join(repo, "crates/vize_canon/Cargo.toml"));
  assert.equal(object(selected.target).src_path, path.join(repo, "crates/vize_canon/src/lib.rs"));
  assert.equal(
    String(selected.package_id).split("#")[0],
    `path+${pathToFileURL(path.join(repo, "crates/vize_canon")).href}`,
  );
  assert.ok(Array.isArray(selected.features) && selected.features.includes("native"));
  assert.ok(path.relative(target, String(selected.executable)).startsWith("ci/deps/vize_canon-"));
  sdkArtifacts(rows);
  return selected;
}
