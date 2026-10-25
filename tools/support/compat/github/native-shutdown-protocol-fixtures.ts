// Provider lifecycle controls have no authored-language/dialect coordinates.
// This closed registry keeps them outside the batch manifest's `cases` array.
import { createHash } from "node:crypto";

type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
const root = "crates/vize_canon/src/lsp_client/editor_lsp/native_shutdown";
const fixture = "native-shutdown-completion-10524/completion-envelopes.json";
const selectorRoot = "lsp_client::editor_lsp::native_shutdown::";
const selectors = [
  {
    selector: `${selectorRoot}completion::tests::complete_provider_envelopes_and_failures_match_pinned_packets`,
    source: `${root}/completion/tests.rs`,
  },
  {
    selector: `${selectorRoot}tests::completion::buffered_completion_and_unrelated_notifications_preserve_the_whole_transcript`,
    source: `${root}/tests/completion.rs`,
  },
  {
    selector: `${selectorRoot}tests::completion::malformed_missing_stale_and_eof_completion_still_close_the_owned_process`,
    source: `${root}/tests/completion.rs`,
  },
  {
    selector: `${selectorRoot}tests::null_ack_precedes_eof_foreign_exit_is_preserved_and_failed_ack_still_closes`,
    source: `${root}/tests.rs`,
  },
];
const pairFixture = "native-provider-pairs-10599/provider-pairs.json";
const pairSelectors = [
  {
    selector: `${selectorRoot}tests::provider_pairs::admission_matches_pinned_complete_provider_pairs`,
    source: `${root}/tests/provider_pairs.rs`,
  },
  {
    selector: `${selectorRoot}tests::native_provider::pinned_mapper_sessions_keep_complete_diagnostics_and_clean_owned_shutdown`,
    source: `${root}/tests/native_provider.rs`,
  },
  ...selectors.slice(1),
];

export function nativeShutdownProtocolSource(path: string): boolean {
  if (path === `${root}.rs` || path === `${root}/peer.ts.txt`) return true;
  if (!path.startsWith(`${root}/`)) return false;
  return /^(?:[a-z_][a-z0-9_]*\/)*[a-z_][a-z0-9_]*\.rs$/.test(path.slice(root.length + 1));
}

function require(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(`invalid native shutdown protocol case: ${message}`);
}

function object(value: unknown, keys: string[]): Record<string, unknown> {
  require(value && typeof value === "object" && !Array.isArray(value), "object required");
  require(JSON.stringify(Object.keys(value).sort()) ===
    JSON.stringify([...keys].sort()), "closed schema fields differ");
  return value as Record<string, unknown>;
}

function pinned(readFile: ReadFile, path: string, hash: unknown): Buffer {
  require(typeof hash === "string" && /^[a-f0-9]{64}$/.test(hash), `invalid hash: ${path}`);
  const bytes = readFile("head", path);
  require(bytes !== null, `missing file: ${path}`);
  require(createHash("sha256").update(bytes).digest("hex") === hash, `hash mismatch: ${path}`);
  return bytes;
}

export function newNativeShutdownProtocolCases(
  previous: unknown,
  current: unknown,
  readFile: ReadFile,
): string[] {
  if (current === undefined) return [];
  require(Array.isArray(current), "protocolCases must be an array");
  require(previous === undefined ||
    Array.isArray(previous), "prior protocolCases must be an array");
  const oldIds = new Set((previous ?? []).map((entry: { id?: unknown }) => entry?.id));
  const ids = new Set<string>();
  return current.flatMap((value) => {
    const row = object(value, ["id", "state", "adapter", "targets", "inputs", "tests"]);
    require(typeof row.id === "string" && !ids.has(row.id), "missing or duplicate ID");
    ids.add(row.id);
    if (oldIds.has(row.id)) return [];
    const pairs = row.id === "typechecker/protocol/native-provider-pairs-10599";
    require(pairs ||
      row.id === "typechecker/protocol/native-shutdown-completion-10524", "unregistered ID");
    const registeredFixture = pairs ? pairFixture : fixture;
    const registeredSelectors = pairs ? pairSelectors : selectors;
    require(row.state === "active", "new case must be active");
    require(row.adapter === "native-shutdown-wire-v1", "unregistered adapter");
    require(JSON.stringify(row.targets) === '["native-shutdown"]', "unregistered target");
    require(Array.isArray(row.inputs) && row.inputs.length === 1, "one raw envelope pack required");
    const input = object(row.inputs[0], ["path", "sha256"]);
    require(input.path === registeredFixture, "unregistered raw envelope pack");
    const bytes = pinned(
      readFile,
      `tests/_fixtures/differential/typechecker/${registeredFixture}`,
      input.sha256,
    );
    const envelopes: unknown = JSON.parse(bytes.toString("utf8"));
    require(Array.isArray(envelopes) && envelopes.length > 0, "empty or invalid envelope pack");
    if (pairs) {
      validateProviderPairs(envelopes);
    } else {
      const kinds = new Set<string>();
      for (const value of envelopes) {
        const envelope = object(value, ["input", "expected"]);
        const expected = envelope.expected as { kind?: unknown } | null;
        const kind = expected?.kind;
        require(typeof kind === "string" &&
          ["complete", "unrelated", "malformed"].includes(kind), "invalid outcome");
        const result = object(expected, kind === "malformed" ? ["kind", "error"] : ["kind"]);
        if (kind === "malformed")
          require(typeof result.error === "string" &&
            result.error.length > 0, "missing whole error");
        kinds.add(kind);
      }
      require(kinds.size === 3, "complete, unrelated and malformed controls required");
    }
    for (const [path, declaration] of [
      [`${root}.rs`, /\bmod completion;/],
      [`${root}.rs`, /#\[cfg\(test\)\]\s+mod tests;/],
      [`${root}/completion.rs`, /#\[cfg\(test\)\]\s+mod tests;/],
      [`${root}/tests.rs`, /#\[cfg\(unix\)\]\s+mod completion;/],
    ] as const) {
      const source = readFile("head", path)?.toString("utf8");
      require(source !== undefined &&
        declaration.test(source), `missing ordinary test module: ${path}`);
    }
    if (pairs) {
      const source = readFile("head", `${root}/tests.rs`)?.toString("utf8");
      require(source !== undefined &&
        /\bmod provider_pairs;/.test(source), "missing ordinary pair test module");
      require(source !== undefined &&
        /#\[cfg\(unix\)\]\s+mod native_provider;/.test(
          source,
        ), "missing ordinary native provider test module");
    }
    require(Array.isArray(row.tests) &&
      row.tests.length === registeredSelectors.length, "all registered selectors required");
    for (const [index, registered] of registeredSelectors.entries()) {
      const test = object(row.tests[index], ["selector", "source", "sha256"]);
      require(test.selector === registered.selector &&
        test.source === registered.source, "unregistered selector or source");
      const source = pinned(readFile, registered.source, test.sha256).toString("utf8");
      const name = registered.selector.split("::").at(-1);
      require(source.includes(`#[test]\nfn ${name}(`), "registered test body missing");
      if (index === 0)
        require(source.includes(registeredFixture), "raw envelope pack is not consumed");
    }
    return [row.id];
  });
}

function validateProviderPairs(envelopes: unknown[]) {
  const expectedIds = [
    "npm-linux-complete",
    "npm-darwin-complete",
    "mapper-linux-complete",
    "npm-linux-with-mapper-contract",
    "npm-darwin-with-mapper-contract",
    "mapper-linux-with-npm-contract",
    "unknown-physical-image",
    "changed-encoding",
    "missing-diagnostics",
    "changed-version",
    "changed-name",
    "extra-capability",
    "server-info-only",
    "null-initialize",
  ];
  require(envelopes.length === expectedIds.length, "all complete provider pair controls required");
  for (const [index, value] of envelopes.entries()) {
    const envelope = object(value, ["id", "input", "expected"]);
    require(envelope.id === expectedIds[index], "unregistered provider pair control");
    const input = object(envelope.input, ["imageSha256", "initialize"]);
    require(typeof input.imageSha256 === "string" &&
      /^[a-f0-9]{64}$/.test(input.imageSha256), "invalid physical image identity");
    const initialized = object(input.initialize, ["jsonrpc", "id", "result"]);
    require(initialized.jsonrpc === "2.0" &&
      initialized.id === 1, "whole Initialize response required");
    const expected = object(envelope.expected, ["kind"]);
    require(expected.kind ===
      (index < 3 ? "admitted" : "rejected"), "provider pair outcome changed");
  }
}
