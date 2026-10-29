// One closed API/SourceFile ABI registration, outside authored-language rates.
import { createHash } from "node:crypto";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
export const NATIVE_BETA_DTO_PROTOCOL_ID = "typechecker/protocol/native-beta-diagnostic-dto-10731";
const fixtureRoot = "tests/_fixtures/differential/typechecker/";
const dto = "crates/vize_canon/src/lsp_client/initial_diagnostics/";
const snapshot = "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/";
const production = [
  `${dto}diagnostic_response.rs`,
  `${dto}diagnostic_response/location.rs`,
  `${dto}native_diagnostics.rs`,
  `${snapshot}protocol.rs`,
  `${snapshot}protocol/protocol9.rs`,
  "crates/vize_canon/src/lsp_client/session/capabilities.rs",
  "crates/vize_canon/src/lsp_client/editor_lsp/component_types.rs",
  "crates/vize_canon/src/lsp_client/diagnostics/batch.rs",
];
const tests = [
  {
    source: `${dto}diagnostic_response/beta_tests.rs`,
    selector:
      "lsp_client::initial_diagnostics::diagnostic_response::beta_tests::actual_pinned_beta_dto_and_source_reproduce_the_complete_semantic_packet",
  },
  {
    source: `${dto}diagnostic_response/beta_tests.rs`,
    selector:
      "lsp_client::initial_diagnostics::diagnostic_response::beta_tests::metadata_null_unknown_fields_partial_ranges_and_wrong_source_never_become_clean",
  },
  {
    source: `${snapshot}protocol/protocol9/tests.rs`,
    selector:
      "lsp_client::editor_lsp::snapshot_source::protocol::protocol9::tests::actual_beta_complete_text_is_independent_of_the_opaque_ast_and_source_id",
  },
  {
    source: `${snapshot}tests/native_beta.rs`,
    selector:
      "lsp_client::editor_lsp::snapshot_source::tests::native_beta::actual_sdk_semantic_error_repair_navigation_and_two_file_scope_are_complete",
  },
  {
    source: `${snapshot}tests.rs`,
    selector:
      "lsp_client::editor_lsp::snapshot_source::tests::completion_comparison_retains_metadata_nested_fields_and_duplicate_counts",
  },
];
const scope = new Set([
  ...production,
  `${dto}diagnostic_response/beta_tests.rs`,
  `${dto}native_diagnostics/tests.rs`,
  `${snapshot}protocol/protocol9/tests.rs`,
  `${snapshot}tests.rs`,
  `${snapshot}tests/native_beta.rs`,
]);
const inputs = [
  "snapshot-source-text/protocol9-beta.bin",
  "snapshot-source-text/protocol9-beta.json",
  "native-beta-diagnostic-dto-10731/actual-beta-wire.json",
  "native-beta-diagnostic-dto-10731/source-pairs.json",
];
const workflow = ".github/workflows/native-beta-diagnostic-dto.yml";
const producerFiles = [
  workflow,
  "tests/tooling/support/native-beta-dto/prepare-role.mjs",
  "tests/tooling/support/native-beta-dto/provider.mjs",
  "tests/tooling/support/native-beta-dto/audit-wire.mjs",
  "tests/tooling/support/native-beta-dto/run-source.mjs",
];
const before = "13fa41356d075047a229f580014b637d3bbeb57f";
const betaCommit = "3e9236361fa97a9e6c685f4833ac457ae2d8c50a";
const sha = (value: Buffer) => createHash("sha256").update(value).digest("hex");
function require(value: unknown, message: string): asserts value {
  if (!value) throw new Error(`invalid native beta DTO protocol case: ${message}`);
}
function object(value: unknown, keys: string[]): Record<string, unknown> {
  require(value && typeof value === "object" && !Array.isArray(value), "object required");
  require(JSON.stringify(Object.keys(value).sort()) ===
    JSON.stringify([...keys].sort()), "closed schema fields differ");
  return value as Record<string, unknown>;
}
function pinned(
  read: ReadFile,
  path: string,
  hash: unknown,
  side: "base" | "head" = "head",
): Buffer {
  require(typeof hash === "string" && /^[a-f0-9]{64}$/.test(hash), `invalid hash: ${path}`);
  const data = read(side, path);
  require(data !== null && sha(data) === hash, `missing or changed ${side} file: ${path}`);
  return data;
}
export function nativeBetaDtoProtocolSource(path: string): boolean {
  return scope.has(path);
}
function frames(data: Buffer): Record<string, unknown>[] {
  const rows: Record<string, unknown>[] = [];
  let offset = 0;
  while (offset < data.length) {
    const separator = data.indexOf("\r\n\r\n", offset);
    require(separator >= 0, "unclosed actual wire header");
    const headers = data.subarray(offset, separator).toString("ascii");
    const length = [...headers.matchAll(/^Content-Length: ([0-9]+)$/gim)];
    require(length.length === 1, "one actual body length required");
    const end = separator + 4 + Number(length[0]?.[1]);
    require(end <= data.length && end > separator + 4, "truncated actual wire body");
    const row = JSON.parse(data.subarray(separator + 4, end).toString("utf8")) as Record<
      string,
      unknown
    >;
    require(row.jsonrpc === "2.0", "actual wire envelope required");
    rows.push(row);
    offset = end;
  }
  return rows;
}
export function newNativeBetaDtoProtocolCases(
  previous: unknown,
  current: unknown,
  read: ReadFile,
): string[] {
  require(previous === undefined ||
    Array.isArray(previous), "prior DTO partition must be an array");
  require(current === undefined || Array.isArray(current), "DTO partition must be an array");
  const oldIds = new Set((previous ?? []).map((row: { id?: unknown }) => row?.id));
  require((current ?? []).length <= 1, "one closed DTO row only");
  return (current ?? []).flatMap((value: unknown) => {
    const row = object(value, ["id", "state", "adapter", "targets", "inputs", "tests"]);
    require(row.id === NATIVE_BETA_DTO_PROTOCOL_ID, "unregistered DTO ID");
    if (oldIds.has(row.id)) return [];
    require(row.state === "active" &&
      row.adapter === "native-beta-diagnostic-wire-v1", "unregistered state or adapter");
    require(JSON.stringify(row.targets) === '["native-diagnostic-dto"]', "unregistered target");
    require(Array.isArray(row.inputs) &&
      row.inputs.length === inputs.length, "all actual ABI inputs required");
    const data = inputs.map((path, index) => {
      const input = object((row.inputs as unknown[])[index], ["path", "sha256"]);
      require(input.path === path, "unregistered actual ABI input");
      return pinned(read, fixtureRoot + path, input.sha256);
    });
    const binary = data[0];
    const fixture = JSON.parse(data[1]!.toString("utf8"));
    require(binary !== undefined &&
      binary.readUInt32LE(0) === 9 << 24 &&
      sha(binary) === fixture.binarySha256, "actual protocol9 payload required");
    const wire = JSON.parse(data[2]!.toString("utf8"));
    require(wire.provider.version === "7.1.0-beta" &&
      wire.provider.gitHead === betaCommit &&
      wire.provider.sha256 ===
        "29f5eb8daa5c8cf6ef1c0cb79fbcf47c0cfdf26edabdf148bb779d0b2d56cebc", "unregistered actual ABI producer");
    const raw = (field: string) => {
      require(typeof wire[field] === "string", "complete actual stream required");
      const value = Buffer.from(wire[field], "base64");
      require(value.toString("base64") === wire[field], "noncanonical actual stream encoding");
      return value;
    };
    const requests = frames(raw("stdinBase64"));
    const responses = frames(raw("stdoutBase64"));
    require(raw("stderrBase64").length === 0, "actual API stream must close without stderr");
    require(requests.length === 5 &&
      responses.length ===
        5, "complete initialize/snapshot/semantic/source/release sequence required");
    const methods = [
      "initialize",
      "createSnapshot",
      "getSemanticDiagnostics",
      "getSourceFile",
      "release",
    ];
    for (const [index, method] of methods.entries()) {
      require(requests[index]?.method === method &&
        requests[index]?.id === responses[index]?.id &&
        !("error" in responses[index]!), "actual owned request/reply order differs");
    }
    require(JSON.stringify(responses[2]?.result) ===
      JSON.stringify(fixture.diagnostics), "whole actual semantic DTO differs");
    const encoded = responses[3]?.result as { data?: string };
    require(typeof encoded?.data === "string" &&
      Buffer.from(encoded.data, "base64").equals(
        binary,
      ), "whole same-snapshot source payload differs");
    for (const index of [2, 3, 4]) {
      const params = requests[index]?.params as { snapshot?: unknown; project?: unknown };
      require(params.snapshot === fixture.snapshot, "foreign actual snapshot");
      if (index !== 4) require(params.project === fixture.project, "foreign actual project");
    }
    const pairs = JSON.parse(data[3]!.toString("utf8"));
    require(pairs.beforeRevision === before &&
      pairs.candidateProductionRevision ===
        "ce77ac02d1f611b3dd935cee983662ea63df7cb1", "pinned source pair required");
    require(Array.isArray(pairs.production) &&
      pairs.production.length === production.length, "all exact production blobs required");
    for (const [index, path] of production.entries()) {
      const pair = object(pairs.production[index], ["path", "beforeSha256", "afterSha256"]);
      require(pair.path === path, "unregistered production source");
      pinned(read, path, pair.afterSha256);
      if (pair.beforeSha256 === null)
        require(read("base", path) === null, "new baseline source was forged");
      else pinned(read, path, pair.beforeSha256, "base");
    }
    require(Array.isArray(pairs.producerFiles) &&
      pairs.producerFiles.length === producerFiles.length, "all four-role producer files required");
    for (const [index, path] of producerFiles.entries()) {
      const entry = object(pairs.producerFiles[index], ["path", "sha256"]);
      require(entry.path === path, "unregistered four-role producer authority");
      pinned(read, path, entry.sha256);
    }
    require(Array.isArray(row.tests) &&
      row.tests.length === tests.length, "all required protocol/source selectors required");
    for (const [index, expected] of tests.entries()) {
      const test = object(row.tests[index], ["selector", "source", "sha256"]);
      require(test.selector === expected.selector &&
        test.source === expected.source, "unregistered selector or source");
      const source = pinned(read, expected.source, test.sha256).toString("utf8");
      require(source.includes(
        `fn ${expected.selector.split("::").at(-1)}(`,
      ), "required Rust test missing");
      if (index === 0)
        require(source.includes(
          "snapshot-source-text/protocol9-beta.json",
        ), "actual DTO pack is not consumed");
    }
    const actual = read("head", tests[3]!.source)?.toString("utf8") ?? "";
    for (const marker of [
      "#[ignore",
      "native_diagnostics::read(",
      "editor.diagnostics(",
      ".hover(",
      ".references(",
      "NEIGHBOR",
      "expected_refusal",
      "snapshot.release()",
      "FileChanges::Summary(",
      "request_diagnostics_full(",
      "dependency-error",
      "dependency-close",
      "component_types_raw(",
      "completion_raw(",
      "definition_raw(",
      "rename_raw(",
      "mapped_virtual",
      "whole_batch",
      "directCompletion",
    ])
      require(actual.includes(
        marker === "expected_refusal" ? "expect_refusal" : marker,
      ), `missing actual observer authority: ${marker}`);
    const job = read("head", workflow)?.toString("utf8") ?? "";
    for (const marker of [
      "role: [before, after]",
      "version: 7.0.2",
      "version: 7.1.0-beta",
      before,
      tests[3]!.selector,
      "run-source.mjs",
      "audit-wire.mjs",
    ])
      require(job.includes(marker), `missing actual four-role producer: ${marker}`);
    return [NATIVE_BETA_DTO_PROTOCOL_ID];
  });
}
