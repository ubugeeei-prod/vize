import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { parseTomlLite } from "../../../../tools/support/compat/davinci/toml-lite.ts";
import type { WatcherContext } from "./stable-watcher-boundary.ts";
type Row = Record<string, unknown>;
const object = (value: unknown): Row => {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Row;
};
const array = (value: unknown): unknown[] => {
  assert.ok(Array.isArray(value));
  return value;
};
const text = (value: unknown): string => {
  assert.equal(typeof value, "string");
  return value as string;
};
const hash = (bytes: Uint8Array | string) => createHash("sha256").update(bytes).digest("hex");
const registry = "registry+https://github.com/rust-lang/crates.io-index";
// Exact seven public SDK4 checksums/runtime lock vectors authenticated by TP's
// 446a1cf8… Rust publication handoff. Unregistered SDK generations never acquire this authority.
export const sdk4CleanupPackages = [
  [
    "corsa",
    "a88d1e7cd3fa6919d9f2ab4b51de40645547606317ac3f04dfccc1ded0ce9dcc",
    [
      "base64",
      "corsa_client",
      "corsa_core",
      "corsa_jsonrpc",
      "corsa_lsp",
      "corsa_orchestrator",
      "corsa_runtime",
      "lsp-types 0.97.0",
      "serde_json",
    ],
  ],
  [
    "corsa_client",
    "48683d933fc2fd2b2bf9f8563103c2bbcb249f0ec6a6ace1831644ca0a84d780",
    [
      "base64",
      "corsa_core",
      "corsa_jsonrpc",
      "corsa_runtime",
      "log",
      "lsp-types 0.97.0",
      "parking_lot",
      "phf 0.14.0",
      "serde",
      "serde_json",
    ],
  ],
  [
    "corsa_core",
    "c64b958bb6e54a0f6a13cde2f5df28ce95234b6bb90c289fd2a3b6027af951c7",
    [
      "base64",
      "compact_str",
      "memchr",
      "rustc-hash 2.1.3",
      "serde",
      "serde_json",
      "smallvec",
      "thiserror 2.0.21",
    ],
  ],
  [
    "corsa_jsonrpc",
    "5603d3a1ca6c9f0caadbf8e89c853b740f31f44e818a9c1a01f09c2b93da5c8f",
    ["corsa_core", "corsa_runtime", "log", "parking_lot", "serde", "serde_json"],
  ],
  [
    "corsa_lsp",
    "b6949c82fac11db1376c361c7e70a6ac6019dedce1d379110385229eb643ea25",
    ["corsa_core", "corsa_jsonrpc", "corsa_runtime", "lsp-types 0.97.0", "parking_lot", "serde"],
  ],
  [
    "corsa_orchestrator",
    "56a610d7d8fec2e576d461751bd7ccb7bf2348a030418b5dc6756473353e2c0c",
    [
      "corsa_client",
      "corsa_core",
      "corsa_lsp",
      "corsa_runtime",
      "log",
      "parking_lot",
      "serde",
      "serde_json",
    ],
  ],
  [
    "corsa_runtime",
    "71b6fd0a075158abbb1c513e754eb73b368066bbccf5967c9a2be5d6305df637",
    ["smallvec"],
  ],
] as const;
// #12934 moved the pins to 2.0.0-beta.5. The dependency vectors are unchanged;
// each checksum equals the crates.io index `cksum` for that exact version.
const sdk5Checksums: Readonly<Record<string, string>> = {
  corsa: "2423289c1b3234486f63f98940e6ebc52844eb17a65de347120c05edb4f08b15",
  corsa_client: "762f29521c89cc16bca7b00864c7ca4c52bcf024f9b9f3581ff31f69dc01eec7",
  corsa_core: "47f70f4d9cff68ca173fe4fae206e9c01c378dda59ed9eef1321bf11d236c807",
  corsa_jsonrpc: "e1a595cdbda26eedd79f89c957dab1d5425b3ea4e7f3950d95e6071e7fc03eaf",
  corsa_lsp: "ee8da211e9bb610c8a0bf5f32eabf0beefc0b62044ab39c10a32ab99f811cb30",
  corsa_orchestrator: "8de4a68aa41bfa0acf1e13e2945072aacf982d7e67e7420b260ab9079f97e6b0",
  corsa_runtime: "be088c5e72ac415932623f06f9bff3084b2324d49eb4b10ee410aad29b7e7842",
};
type CleanupPackage = readonly [string, string, readonly string[]];
export const sdk5CleanupPackages: readonly CleanupPackage[] = sdk4CleanupPackages.map(
  ([name, , dependencies]) => [name, sdk5Checksums[name]!, dependencies] as const,
);
/** Each registered SDK generation keeps the same source-authored two-close cleanup. */
export const cleanupGenerations: ReadonlyMap<string, readonly CleanupPackage[]> = new Map<
  string,
  readonly CleanupPackage[]
>([
  ["2.0.0-beta.4", sdk4CleanupPackages],
  ["2.0.0-beta.5", sdk5CleanupPackages],
]);
const names: readonly string[] = sdk4CleanupPackages.map((row) => row[0]);
const observer = "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/tests/native_beta.rs";
const helper = observer.replace(/\.rs$/u, "/storage_authority.rs");
const input =
  "tests/_fixtures/differential/typechecker/logical-workspace-overlay-revision-11145/input.json";
const observerHash = "c75ef79c388eb5aee6c93115150196c42aecb75d638ead10dab1a1b94872ca61";
const helperHash = "9bc79324540fd1b37f649f102fa2ed90a09aaf53eca1765f799cb6bc1fe88d40";
const inputHash = "4659fc88d76ee98c9f0ddc74f6450ecc80b720be9b3ab4a0335bf8f2a2050c58";
const decode = (bytes: Uint8Array) => {
  assert.ok(bytes.byteLength > 0 && bytes.byteLength <= 4 * 1024 ** 2);
  return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
};
const bindings = new WeakMap<object, string>();
/** Pure source/compiler join; this supplies no native lifecycle or qualification credit. */
export function verifyAfter4CleanupSource(
  sourceValue: unknown,
  read: (file: string) => Uint8Array,
  build: Uint8Array,
) {
  const source = object(sourceValue);
  if (source.role !== "after" || source.sdkVersion !== "7.1.0-beta") return null;
  assert.equal(
    source.sdkSha256,
    "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
  );
  const manifestBytes = read("Cargo.toml"),
    manifest = decode(manifestBytes),
    assignments: string[] = [];
  let section = "";
  for (const line of manifest.split("\n")) {
    if (/^\s*\[/u.test(line)) section = line.trim();
    if (
      /^\s*["']?corsa(?:_client|_core|_jsonrpc|_lsp|_orchestrator|_runtime)?["']?\s*=/u.test(line)
    ) {
      assert.equal(
        section,
        "[workspace.dependencies]",
        "SDK override cannot supply cleanup authority",
      );
      assignments.push(line);
    }
  }
  const pins = object(
    object(parseTomlLite("[workspace.dependencies]\n" + assignments.join("\n")).workspace)
      .dependencies,
  );
  const version = [...cleanupGenerations.keys()].find((candidate) =>
    Object.values(pins).some((value) => object(value).version === "=" + candidate),
  );
  if (version === undefined) return null;
  const generation = cleanupGenerations.get(version)!;
  assert.deepEqual(Object.keys(pins).sort(), ["corsa", "corsa_lsp", "corsa_runtime"]);
  for (const name of Object.keys(pins))
    assert.deepEqual(pins[name], { package: name, version: "=" + version });
  const lockBytes = read("Cargo.lock"),
    lock = parseTomlLite(decode(lockBytes));
  assert.equal(lock.version, 4);
  const packages = array(lock.package)
    .map(object)
    .filter((row) => names.includes(text(row.name)));
  assert.deepEqual(packages.map((row) => text(row.name)).sort(), [...names].sort());
  for (const [name, checksum, dependencies] of generation) {
    const row = packages.find((value) => value.name === name)!;
    assert.deepEqual(row, {
      name,
      version,
      source: registry,
      checksum,
      dependencies: [...dependencies],
    });
  }
  assert.equal(source.observerSha256, observerHash);
  assert.equal(hash(read(observer)), observerHash);
  assert.equal(hash(read(helper)), helperHash);
  assert.equal(hash(read(input)), inputHash);
  assert.equal(read(input).byteLength, 1108);
  assert.deepEqual(source.observerDependencies, [{ path: helper, sha256: helperHash }]);
  const attached = object(source.attached);
  assert.deepEqual(attached.observerInputDependencies, [
    { path: input, bytes: 1108, sha256: inputHash },
  ]);
  const lines = decode(build).split("\n");
  if (lines.at(-1) === "") lines.pop();
  assert.ok(lines.every((line) => line.trim().length > 0));
  const rows = lines.map((line) => object(JSON.parse(line) as unknown)),
    finished = rows.filter((row) => row.reason === "build-finished");
  assert.equal(finished.length, 1);
  assert.equal(finished[0]!.success, true);
  const artifacts = rows.filter((row) => row.reason === "compiler-artifact");
  for (const row of artifacts) assert.ok(rows.indexOf(row) < rows.indexOf(finished[0]!));
  const tests = artifacts.filter(
    (row) =>
      object(row.target).name === "vize_canon" &&
      object(row.profile).test === true &&
      row.executable !== null,
  );
  assert.equal(tests.length, 1);
  assert.equal(tests[0]!.executable, source.testExecutable);
  const libraries = names.map((name) => {
    const found = artifacts.filter(
      (row) => object(row.target).name === name || text(row.package_id).includes("#" + name + "@"),
    );
    assert.ok(found.length > 0, "missing actual SDK4 library: " + name);
    for (const row of found) {
      assert.equal(row.package_id, registry + "#" + name + "@" + version);
      const filename = text(row.manifest_path);
      assert.equal(path.posix.normalize(filename), filename);
      const escaped = (name + "-" + version).replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
      assert.match(
        filename,
        new RegExp(
          "^/(?:[^/\\\\\\u0000-\\u001f]+/)*registry/src/index\\.crates\\.io-[a-f0-9]{16}/" +
            escaped +
            "/Cargo\\.toml$",
          "u",
        ),
      );
      assert.equal(object(row.target).name, name);
      assert.ok(array(object(row.target).kind).includes("lib"));
    }
    assert.equal(new Set(found.map((row) => row.manifest_path)).size, 1);
    return { name, artifacts: found };
  });
  const proof = {
    source: text(source.source),
    version,
    observerSha256: observerHash,
    attached,
    phases: array(attached.rows),
    cargoTomlSha256: hash(manifestBytes),
    cargoLockSha256: hash(lockBytes),
    packages,
    libraries,
    testArtifact: tests[0]!,
    nativeQualification: false,
  };
  assert.match(proof.source, /^[a-f0-9]{40}$/u);
  bindings.set(proof, hash(JSON.stringify(proof)));
  return proof;
}
export type After4CleanupSource = NonNullable<ReturnType<typeof verifyAfter4CleanupSource>>;
export function requireAfter4CleanupSource(
  proof: After4CleanupSource,
  attached: unknown,
  phases: readonly unknown[],
  context: WatcherContext,
) {
  assert.equal(
    bindings.get(proof),
    hash(JSON.stringify(proof)),
    "actual unchanged source/compiler binding required",
  );
  assert.equal(proof.attached, attached);
  assert.equal(proof.phases, phases);
  assert.equal(proof.source, context.source);
  assert.equal(context.role, "after");
  assert.equal(context.sdkVersion, "7.1.0-beta");
  assert.equal(
    context.sdkSha256,
    "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
  );
}
export function loadAfter4CleanupSource(capture: string, sourceValue: unknown) {
  const source = object(sourceValue);
  if (source.role !== "after" || source.sdkVersion !== "7.1.0-beta") return null;
  const root = process.env.VIZE_NATIVE_DTO_SOURCE_ROOT;
  assert.ok(root);
  assert.match(text(source.source), /^[a-f0-9]{40}$/u);
  const read = (file: string) =>
    execFileSync("git", ["show", source.source + ":" + file], {
      cwd: root,
      maxBuffer: 4 * 1024 ** 2,
    });
  return verifyAfter4CleanupSource(
    source,
    read,
    fs.readFileSync(path.join(capture, "build.jsonl")),
  );
}
