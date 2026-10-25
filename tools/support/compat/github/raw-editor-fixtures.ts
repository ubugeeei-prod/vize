// Closed test-contract custody, outside language/batch and native parity counts.
import { createHash } from "node:crypto";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
type Registration = { ids: string[]; sources: Set<string> };
const registry = {
  id: "typechecker/raw-editor/outside-import-contract-10695",
  inputPath: "tests/fixtures/typechecker/outside-import-editor-raw-10695/authority.json",
  inputHash: "96120b4fcedcf646511da2a43041b4d25c175b320f833fad4e932227ace8d2f4",
  sources: [
    {
      path: "crates/vize_canon/src/corsa_bridge/outside_import_editor_tests.rs",
      before: "70c65e63a14ad41e9d82153bb792d3292839c4dfe83535f39f7d89bb1e2b4807",
      after: "c0c67c1f428939ad6398ef024582b0177d4eaaa0a4447ccc81f28b58f951da6a",
    },
    {
      path: "crates/vize_canon/src/corsa_bridge/outside_import_editor_tests/alias_packet.rs",
      before: null,
      after: "80c301b11f69b9b624dfa5caf1367635c60cc037efb7a77950083df36b69ff23",
    },
  ],
  tests: [
    {
      selector:
        "corsa_bridge::outside_import_editor_tests::alias_packet::unchanged_pre_policy_packet_matches_its_independent_generated_ast",
      source: "crates/vize_canon/src/corsa_bridge/outside_import_editor_tests/alias_packet.rs",
      sha256: "80c301b11f69b9b624dfa5caf1367635c60cc037efb7a77950083df36b69ff23",
    },
    {
      selector:
        "corsa_bridge::outside_import_editor_tests::alias_packet::literal_geometry_tracks_utf16_escaped_quotes_crlf_and_different_root_lengths",
      source: "crates/vize_canon/src/corsa_bridge/outside_import_editor_tests/alias_packet.rs",
      sha256: "80c301b11f69b9b624dfa5caf1367635c60cc037efb7a77950083df36b69ff23",
    },
    {
      selector:
        "corsa_bridge::outside_import_editor_tests::alias_packet::an_unrelated_string_is_not_import_geometry_authority",
      source: "crates/vize_canon/src/corsa_bridge/outside_import_editor_tests/alias_packet.rs",
      sha256: "80c301b11f69b9b624dfa5caf1367635c60cc037efb7a77950083df36b69ff23",
    },
    {
      selector:
        "corsa_bridge::outside_import_editor_tests::alias_packet::duplicate_helper_declarations_are_not_position_authority",
      source: "crates/vize_canon/src/corsa_bridge/outside_import_editor_tests/alias_packet.rs",
      sha256: "80c301b11f69b9b624dfa5caf1367635c60cc037efb7a77950083df36b69ff23",
    },
    {
      selector:
        "corsa_bridge::vue_document_package_config_tests::native_editor_resolves_package_alias_from_monorepo_root",
      source: "crates/vize_canon/src/corsa_bridge/vue_document_package_config_tests.rs",
      sha256: "6215f9674046f9e57963c4a6043ee58e3bbf712e49566beee4f3f7a99f8d8b93",
    },
  ],
};
const owner = "crates/vize_canon/src/corsa_bridge.rs";
const fixtureRoot = "tests/fixtures/typechecker/outside-import-editor-raw-10695/";
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
function require(value: unknown, message: string): asserts value {
  if (!value) throw new Error(`invalid raw editor contract: ${message}`);
}
function object(value: unknown, keys: string[]): Record<string, unknown> {
  require(value && typeof value === "object" && !Array.isArray(value), "object required");
  require(JSON.stringify(Object.keys(value).sort()) ===
    JSON.stringify([...keys].sort()), "closed schema differs");
  return value as Record<string, unknown>;
}
function bytes(readFile: ReadFile, path: string, digest: string) {
  const value = readFile("head", path);
  require(value !== null && hash(value) === digest, `missing or changed pinned file: ${path}`);
  return value;
}
function actualPair(readFile: ReadFile): boolean {
  return registry.sources.every((source) => {
    const before = readFile("base", source.path),
      after = readFile("head", source.path);
    return (
      (before === null ? null : hash(before)) === source.before &&
      after !== null &&
      hash(after) === source.after
    );
  });
}
export function newRawEditorCases(
  previous: unknown,
  current: unknown,
  readFile: ReadFile,
  paths: readonly string[],
): Registration {
  const empty = (): Registration => ({ ids: [], sources: new Set<string>() });
  const relevant = registry.sources.some((source) => paths.includes(source.path));
  const required = relevant && actualPair(readFile);
  if (current === undefined) {
    require(!required, "known test-only repair requires its raw contract row");
    return empty();
  }
  require(Array.isArray(current), "rawEditorCases must be an array");
  require(previous === undefined ||
    Array.isArray(previous), "prior rawEditorCases must be an array");
  const old = new Set((previous ?? []).map((entry: { id?: unknown }) => entry?.id));
  const ids = new Set<string>();
  let registration = empty();
  for (const value of current) {
    const row = object(value, ["id", "state", "adapter", "targets", "input", "sources", "tests"]);
    require(typeof row.id === "string" && !ids.has(row.id), "missing or duplicate ID");
    ids.add(row.id);
    require(row.id === registry.id, "unregistered ID");
    if (old.has(row.id)) continue;
    require(row.state === "active" &&
      row.adapter === "native-editor-raw-contract-v1" &&
      JSON.stringify(row.targets) === '["raw-editor"]', "unregistered state, adapter or target");
    require(registry.sources.every((source) =>
      paths.includes(source.path),
    ), "both owned source changes are required");
    require(actualPair(readFile), "unreviewed actual source pair");
    require(JSON.stringify(row.sources) ===
      JSON.stringify(registry.sources), "source scope or hashes changed");
    require(JSON.stringify(row.tests) ===
      JSON.stringify(registry.tests), "registered producer or hash changed");
    const input = object(row.input, ["path", "sha256"]);
    require(input.path === registry.inputPath &&
      input.sha256 === registry.inputHash, "unregistered raw custody root");
    const authority = JSON.parse(
      bytes(readFile, registry.inputPath, registry.inputHash).toString("utf8"),
    );
    require(authority.schema === "vize.outside-import-editor-raw-packet-authority" &&
      authority.version === 1 &&
      authority.completeRawVectorsByteEqual === true &&
      JSON.stringify(authority.originalAssertionStatuses) ===
        "[101,101]", "original failing raw contract is missing");
    require(authority.sdk.sha256 ===
      "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4", "unobserved provider");
    require(Array.isArray(authority.files) &&
      authority.files.length === 7, "all original payloads required");
    for (const file of authority.files) {
      require(typeof file.path === "string" &&
        /^[a-zA-Z0-9_.-]+$/.test(file.path), "invalid payload path");
      const payload = bytes(readFile, fixtureRoot + file.path, file.sha256);
      require(payload.length === file.bytes, "payload byte count changed");
    }
    const root = readFile("head", owner)?.toString("utf8") ?? "";
    require(/#\[cfg\(test\)\]\s+mod outside_import_editor_tests;/.test(
      root,
    ), "test-only root missing");
    require(/#\[cfg\(test\)\]\s+#\[cfg\(unix\)\]\s+mod vue_document_package_config_tests;/.test(
      root,
    ), "real provider test root missing");
    const module = readFile("head", registry.sources[0]!.path)!.toString("utf8");
    require(/\bmod alias_packet;/.test(module), "ordinary helper module missing");
    for (const test of registry.tests) {
      const source = bytes(readFile, test.source, test.sha256).toString("utf8");
      require(source.includes(
        `fn ${test.selector.split("::").at(-1)}(`,
      ), "actual Rust producer missing");
    }
    registration = {
      ids: [registry.id],
      sources: new Set(registry.sources.map((source) => source.path)),
    };
  }
  require(!required ||
    registration.ids.length === 1, "known repair cannot use stale IDs or ordinary filler");
  return registration;
}
