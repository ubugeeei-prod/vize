// A source-bound logical editor revision protocol, outside authored Batch rates.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { NATIVE_BETA_DTO_PROTOCOL_ID } from "./native-beta-dto-fixtures.ts";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
export const LOGICAL_WORKSPACE_PROTOCOL_ID =
  "typechecker/protocol/logical-workspace-overlay-revision-11145";

// Route only one explicitly registered protocol ID; malformed/unknown rows
// remain with the original closed validator and cannot disappear in filtering.
export function protocolPartition(value: unknown, id: string, include: boolean): unknown {
  if (!Array.isArray(value)) return value;
  return value.filter((row: unknown) => {
    const matches =
      row !== null &&
      typeof row === "object" &&
      !Array.isArray(row) &&
      "id" in row &&
      row.id === id;
    return matches === include;
  });
}

/** Unknown or malformed rows still belong to the original shutdown validator. */
export function remainingShutdownProtocolCases(value: unknown): unknown {
  return protocolPartition(
    protocolPartition(value, NATIVE_BETA_DTO_PROTOCOL_ID, false),
    LOGICAL_WORKSPACE_PROTOCOL_ID,
    false,
  );
}

const root = "tests/_fixtures/differential/typechecker/";
const directory = "logical-workspace-overlay-revision-11145/";
const authorityHash = "c3950279205ac4939b10c019c5e10e55632ed336f06fd3f46bc4a8085f7ab3dd";
const inputHash = "4659fc88d76ee98c9f0ddc74f6450ecc80b720be9b3ab4a0335bf8f2a2050c58";
const selector =
  "lsp_client::editor_lsp::snapshot_source::tests::native_beta::actual_sdk_semantic_error_repair_navigation_and_two_file_scope_are_complete";
const observerPath =
  "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/tests/native_beta.rs";
const workflow = ".github/workflows/native-beta-diagnostic-dto.yml";
const production = [
  "crates/vize_canon/src/lsp_client/initial_diagnostics/diagnostic_response.rs",
  "crates/vize_canon/src/lsp_client/initial_diagnostics/diagnostic_response/location.rs",
  "crates/vize_canon/src/lsp_client/initial_diagnostics/native_diagnostics.rs",
  "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/protocol.rs",
  "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/protocol/protocol9.rs",
  "crates/vize_canon/src/lsp_client/session/capabilities.rs",
  "crates/vize_canon/src/lsp_client/editor_lsp/component_types.rs",
  "crates/vize_canon/src/lsp_client/diagnostics/batch.rs",
];
interface Image {
  path: string;
  bytes: number;
  sha256: string;
}
interface Authority {
  schema: string;
  version: number;
  id: string;
  nativeAdmission: boolean;
  production: {
    beforeRevision: string;
    afterRevision: string;
    files: {
      path: string;
      beforeBytes: number;
      afterBytes: number;
      beforeSha256: string;
      afterSha256: string;
      changed: boolean;
    }[];
  };
  consumer: {
    revision: string;
    observer: Image;
    helper: Image;
    corePrefix: { bytes: number; sha256: string };
    input: Image;
    receipt: string;
  };
  nativeRoles: { role: string; source: string; sdk: string; sha256: string }[];
}
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
function object(value: unknown, keys: readonly string[]): Record<string, unknown> {
  assert(value && typeof value === "object" && !Array.isArray(value), "object required");
  assert.deepEqual(Object.keys(value).sort(), [...keys].sort(), "closed protocol fields differ");
  return value as Record<string, unknown>;
}
function pinned(read: ReadFile, path: string, sha256: string, side: "base" | "head" = "head") {
  const bytes = read(side, path);
  assert(bytes !== null && hash(bytes) === sha256, `missing or changed ${side} source: ${path}`);
  return bytes;
}
function image(read: ReadFile, entry: Image) {
  const bytes = pinned(read, entry.path, entry.sha256);
  assert.equal(bytes.length, entry.bytes);
  return bytes;
}
function checkInput(bytes: Buffer) {
  const input = object(JSON.parse(bytes.toString("utf8")), [
    "schema",
    "id",
    "main",
    "support",
    "virtual",
    "configText",
    "phases",
  ]);
  assert.equal(input.schema, 1);
  assert.equal(input.id, LOGICAL_WORKSPACE_PROTOCOL_ID);
  for (const field of ["main", "support", "virtual", "configText"])
    assert.equal(typeof input[field], "string");
  const main = input.main as string;
  assert(main.includes("\r\n") && main.includes("\u2028\u2029") && main.includes("🧭é日本語"));
  const config = JSON.parse(input.configText as string);
  assert.deepEqual(config.files, ["main.ts", "support.ts"]);
  assert.equal(config.compilerOptions.strict, true);
  assert(Array.isArray(input.phases));
  assert.deepEqual(
    input.phases.map((value: unknown) => {
      const phase = object(value, ["id", "support", "error", "closed"]);
      assert.equal(typeof phase.support, "string");
      return [phase.id, phase.error, phase.closed];
    }),
    [
      ["valid", false, false],
      ["dependency-error", true, false],
      ["dependency-repair", false, false],
      ["dependency-close", false, true],
    ],
  );
}

/** A registration proves consumption/source custody, never native admission. */
export function newLogicalWorkspaceProtocolCases(
  previous: unknown,
  current: unknown,
  read: ReadFile,
) {
  assert(
    previous === undefined || Array.isArray(previous),
    "prior logical partition must be an array",
  );
  assert(current === undefined || Array.isArray(current), "logical partition must be an array");
  const old = (previous ?? []) as unknown[];
  const rows = (current ?? []) as unknown[];
  assert(old.length <= 1 && rows.length <= 1, "one closed logical row only");
  if (old.length) {
    assert.deepEqual(rows, old, "registered logical history must remain unchanged");
    return { ids: [] as string[], sources: new Set<string>() };
  }
  if (!rows.length) return { ids: [] as string[], sources: new Set<string>() };
  const row = object(rows[0], ["id", "state", "adapter", "targets", "inputs", "tests"]);
  assert.equal(row.id, LOGICAL_WORKSPACE_PROTOCOL_ID);
  assert.equal(row.state, "active");
  assert.equal(row.adapter, "logical-editor-revision-protocol-v1");
  assert.deepEqual(row.targets, ["native-diagnostic-dto", "logical-workspace-revision"]);
  const inputs = row.inputs;
  assert(Array.isArray(inputs) && inputs.length === 2);
  const data = [inputHash, authorityHash].map((expected, index) => {
    const entry = object(inputs[index], ["path", "sha256"]);
    const path = directory + ["input.json", "source-authority.json"][index];
    assert.equal(entry.path, path);
    assert.equal(entry.sha256, expected);
    return pinned(read, root + path, expected);
  });
  checkInput(data[0]!);
  const authority = JSON.parse(data[1]!.toString("utf8")) as Authority;
  assert.equal(authority.schema, "vize.logical-workspace-protocol-authority");
  assert.equal(authority.version, 1);
  assert.equal(authority.id, LOGICAL_WORKSPACE_PROTOCOL_ID);
  assert.equal(authority.nativeAdmission, false);
  assert.deepEqual(
    authority.production.files.map((entry) => entry.path),
    production,
  );
  for (const entry of authority.production.files) {
    const before = pinned(read, entry.path, entry.beforeSha256, "base");
    const after = pinned(read, entry.path, entry.afterSha256);
    assert.equal(before.length, entry.beforeBytes);
    assert.equal(after.length, entry.afterBytes);
    assert.equal(!before.equals(after), entry.changed);
  }
  const consumer = authority.consumer;
  assert.equal(consumer.observer.path, observerPath);
  assert.equal(consumer.helper.path, observerPath.replace(/\.rs$/, "/storage_authority.rs"));
  const observer = image(read, consumer.observer);
  const helper = image(read, consumer.helper).toString("utf8");
  assert.equal(hash(observer.subarray(0, consumer.corePrefix.bytes)), consumer.corePrefix.sha256);
  assert.equal(consumer.corePrefix.bytes, 12150);
  assert.equal(consumer.input.path, root + directory + "input.json");
  assert.equal(consumer.input.sha256, inputHash);
  assert(image(read, consumer.input).equals(data[0]!));
  assert.equal(consumer.receipt, "attached.observerInputDependencies");
  for (const marker of [
    "include_bytes!(concat!(",
    consumer.input.path,
    "attached-observer-input.json",
    ".create_new(true)",
  ])
    assert(helper.includes(marker), `missing compiled input custody: ${marker}`);
  const text = observer.toString("utf8");
  for (const marker of [
    "storage_authority::input(capture)",
    'input["main"]',
    'input["support"]',
    'input["virtual"]',
    'input["configText"]',
    'input["phases"]',
    'row["error"]',
    'row["closed"]',
    '"observerInputDependencies":input_dependencies',
    `fn ${selector.split("::").at(-1)}(`,
  ])
    assert(text.includes(marker), `missing actual Rust consumer: ${marker}`);
  assert.deepEqual(row.tests, [
    { selector, source: observerPath, sha256: consumer.observer.sha256 },
  ]);
  const job = read("head", workflow)?.toString("utf8") ?? "";
  assert.equal(authority.nativeRoles.length, 4);
  for (const marker of [
    "role: [before, after]",
    "13fa41356d075047a229f580014b637d3bbeb57f",
    selector,
    "prepare-role.ts",
    "run-source.ts",
    "audit-wire.ts",
    ...authority.nativeRoles.flatMap((role) => [`version: ${role.sdk}`, role.sha256]),
  ])
    assert(job.includes(marker), `missing four-role native producer: ${marker}`);
  return {
    ids: [LOGICAL_WORKSPACE_PROTOCOL_ID],
    sources: new Set([...production, consumer.observer.path, consumer.helper.path]),
  };
}
