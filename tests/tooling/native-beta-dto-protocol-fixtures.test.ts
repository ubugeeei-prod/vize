import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { test } from "node:test";
import {
  NATIVE_BETA_DTO_PROTOCOL_ID,
  nativeBetaDtoProducerCustody,
  nativeBetaDtoObserverBoundary,
  nativeBetaDtoProtocolSource,
  newNativeBetaDtoProtocolCases,
} from "../../tools/support/compat/github/native-beta-dto-fixtures.ts";
import { loadTypecheckerManifest } from "../differential/typechecker.ts";

const manifestPath = "tests/_fixtures/differential/typechecker/manifest.json";
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
const row = manifest.protocolCases.find(
  (row: { id: string }) => row.id === NATIVE_BETA_DTO_PROTOCOL_ID,
);
type MutationInput = {
  stdinBase64: string;
  stdoutBase64: string;
  stderrBase64: string;
  provider: { gitHead: string };
  beforeRevision: string;
  candidateProductionRevision: string;
  production: { afterSha256: string }[];
  producerFiles: { sha256: string }[];
};
const before = "13fa41356d075047a229f580014b637d3bbeb57f";
const candidate = "ce77ac02d1f611b3dd935cee983662ea63df7cb1";
const historicalProduction = new Set(nativeBetaDtoObserverBoundary().production);
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
function read(side: "base" | "head", path: string): Buffer | null {
  try {
    // This synthetic initial registration consumes its frozen source-pair actor.
    // Current producer/observer controls still read the actual working-tree bytes.
    return side === "head" && !historicalProduction.has(path)
      ? fs.readFileSync(path)
      : execFileSync("git", ["show", `${side === "head" ? candidate : before}:${path}`], {
          stdio: ["ignore", "pipe", "ignore"],
        });
  } catch {
    return null;
  }
}
function validate(
  rows: unknown = [row],
  previous: unknown = [],
  override: Record<string, Buffer | null> = {},
) {
  return newNativeBetaDtoProtocolCases(previous, rows, (side, path) =>
    side === "head" && Object.hasOwn(override, path) ? override[path]! : read(side, path),
  );
}
void test("one closed DTO registration consumes actual ABI and four-role authority outside batch rates", () => {
  assert.deepEqual(validate(), [NATIVE_BETA_DTO_PROTOCOL_ID]);
  const changed = "crates/vize_canon/src/lsp_client/session/capabilities.rs";
  assert.notDeepEqual(fs.readFileSync(changed), read("head", changed));
  assert.throws(
    () => validate([row], [], { [changed]: fs.readFileSync(changed) }),
    "today's changed ABI source cannot inherit the historical registration",
  );
  const loaded = loadTypecheckerManifest(manifestPath);
  assert.equal(
    loaded.cases.length,
    manifest.cases.filter((entry: { pack?: string }) => entry.pack !== undefined).length,
  );
  assert(!loaded.cases.some((entry) => entry.id === NATIVE_BETA_DTO_PROTOCOL_ID));
});
void test("typed producers retain historical bytes and only the exact registered observer successor", () => {
  const custody = nativeBetaDtoProducerCustody(read);
  assert.equal(custody.length, 5);
  assert.deepEqual(
    custody.map((entry) => entry.historicalPath),
    [
      ".github/workflows/native-beta-diagnostic-dto.yml",
      "tests/tooling/support/native-beta-dto/prepare-role.mjs",
      "tests/tooling/support/native-beta-dto/provider.mjs",
      "tests/tooling/support/native-beta-dto/audit-wire.mjs",
      "tests/tooling/support/native-beta-dto/run-source.mjs",
    ],
  );
  for (const entry of custody) {
    assert.equal(entry.currentPath, entry.historicalPath.replace(/\.mjs$/u, ".ts"));
    assert.equal(entry.historicalSha256, hash(fs.readFileSync(entry.archivedPath)));
    assert.equal(entry.currentSha256, hash(fs.readFileSync(entry.currentPath)));
    for (const path of [entry.archivedPath, entry.currentPath]) {
      assert.throws(() => validate([row], [], { [path]: null }));
      assert.throws(() => validate([row], [], { [path]: Buffer.from("changed producer\n") }));
    }
  }
  const current = custody.find((entry) => entry.currentPath.endsWith("audit-wire.ts"))!;
  assert.equal(
    current.currentRootWarningRegistration,
    "before-stable-root-watch-warning-41214579457",
  );
  assert(
    custody
      .filter((entry) => entry !== current)
      .every((entry) => entry.currentRootWarningRegistration === null),
  );
  assert.equal(
    current.currentRuntimeRegistration,
    "before-stable-post-shutdown-observation-41099824102",
  );
  assert.equal(
    current.currentRouteRegistration,
    "configured-startup-beta-semantic-route-41108828160",
  );
  assert.equal(current.currentSplitLogRegistration, "before-stable-split-watch-log-41152849787");
  assert.equal(
    current.currentMembershipRegistration,
    "source-owned-config-membership-route-41214579457",
  );
  assert(
    custody
      .filter((entry) => entry !== current)
      .every((entry) => entry.currentMembershipRegistration === null),
  );
  assert.equal(current.currentTerminalRegistration, "before-stable-terminal-stderr-41156819864");
  assert.equal(
    current.currentUnavailableTerminalRegistration,
    "before-stable-unavailable-terminal-stderr-41208964081",
  );
  assert(
    custody
      .filter((entry) => entry !== current)
      .every((entry) => entry.currentUnavailableTerminalRegistration === null),
  );
  assert.equal(
    current.currentMixedEofRegistration,
    "before-stable-unproven-eof-return-41171790405",
  );
  assert(
    custody
      .filter((entry) => entry !== current)
      .every((entry) => entry.currentMixedEofRegistration === null),
  );
  assert(
    custody
      .filter((entry) => entry !== current)
      .every((entry) => entry.currentTerminalRegistration === null),
  );
  assert(
    custody
      .filter((entry) => entry !== current)
      .every((entry) => entry.currentSplitLogRegistration === null),
  );
  assert.equal(current.currentLateRootRegistration, "before-stable-late-root-program-41176048826");
  assert(
    custody
      .filter((entry) => entry !== current)
      .every((entry) => entry.currentLateRootRegistration === null),
  );
  assert.deepEqual(current.currentTypeDependency, {
    path: "tests/tooling/support/native-beta-dto/audit-wire-types.ts",
    sha256: "3904cfaf07e59b501c2361b65410877f22780bfdc365cedf74300bb6fb88d127",
  });
  const storage =
    "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/tests/native_beta/storage_authority.rs";
  for (const entry of custody) {
    const selected = /\/(?:prepare-role|audit-wire)\.ts$/u.test(entry.currentPath);
    assert.equal(
      entry.currentStorageRegistration,
      selected ? "after-source-selected-logical-editor-revision" : null,
    );
    assert.deepEqual(
      entry.currentStorageObserverDependency,
      selected
        ? {
            path: storage,
            sha256: "e760076f8111d8d20c6e35574b79c4da01e2c6d96c7b068b35de82d9bb0c2804",
          }
        : null,
    );
    assert.deepEqual(
      entry.currentObserverIdentity,
      selected
        ? {
            path: "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/tests/native_beta.rs",
            historicalSha256: "26eb21864007e6011c69d0a95e1be173fd586605a87400bdc3aacecb5cb31dc6",
            currentSha256: "ef58d79d4bbac2ea3c7752c00ff1cb62f260664ecc0c481060ed0f65b411cc30",
          }
        : null,
    );
  }
  assert(
    custody
      .filter((entry) => entry !== current)
      .every((entry) => entry.currentTypeDependency === null),
  );
  assert(
    custody
      .filter((entry) => entry !== current)
      .every((entry) => entry.currentRuntimeRegistration === null),
  );
  const bytes = fs.readFileSync(current.currentPath, "utf8");
  assert.throws(() =>
    validate([row], [], {
      [current.currentPath]: Buffer.from(
        bytes.replace(
          "assert.equal(source.rows.length, 3);",
          "assert.equal(source.rows.length, 0);",
        ),
      ),
    }),
  );
  assert.deepEqual(
    validate([row], [], {
      [current.currentPath]: Buffer.from("type InertTypeOnly = number;\n" + bytes),
    }),
    [NATIVE_BETA_DTO_PROTOCOL_ID],
  );
  const workflow = custody[0]!;
  assert.throws(() =>
    validate([row], [], {
      [workflow.currentPath]: Buffer.from(
        fs
          .readFileSync(workflow.currentPath, "utf8")
          .replace("role: [before, after]", "role: [after]"),
      ),
    }),
  );
});
void test("current observer successor requires its exact bounded runtime and actual raw packet authority", () => {
  for (const path of [
    "tests/tooling/support/native-beta-dto/audit-wire-types.ts",
    "tests/tooling/support/native-beta-dto/stable-watcher-boundary.ts",
    "tests/tooling/support/native-beta-dto/beta-attached-route.ts",
    "tests/tooling/support/native-beta-dto/native-io.ts",
    "tests/tooling/support/native-beta-dto/execve-handoff.ts",
    "tests/tooling/support/native-beta-dto/terminal-stderr.ts",
    "tests/tooling/support/native-beta-dto/terminal-stderr-unavailable.ts",
    "tests/tooling/support/native-beta-dto/configuration-membership.ts",
    "tests/tooling/support/native-beta-dto/semantic-route.ts",
    "tests/tooling/support/native-beta-dto/logical-revision.ts",
    "tests/tooling/support/native-beta-dto/logical-attached-route.ts",
    "tools/support/compat/github/native-beta-dto-current-custody.ts",
    "tests/tooling/support/native-beta-dto/historical-root-watch-warning.ts",
    "tests/tooling/support/native-beta-dto/role-disposition.ts",
    "tests/tooling/support/native-beta-dto/stable-watcher-timing.ts",
    "tests/tooling/support/native-beta-dto/stable-watcher-mixed.ts",
    "tests/tooling/support/native-beta-dto/stable-watcher-late-root.ts",
    "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/tests/native_beta/storage_authority.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/tests/native_beta.rs",
    ...["client.bin", "server.bin", "identity.json"].map(
      (name) =>
        "tests/_fixtures/differential/typechecker/native-beta-diagnostic-dto-10731/" +
        "stable-post-shutdown-41099824102/" +
        name,
    ),
  ]) {
    assert.throws(() => validate([row], [], { [path]: null }));
    assert.throws(() => validate([row], [], { [path]: Buffer.from("changed observation\n") }));
  }
  const helper = "tests/tooling/support/native-beta-dto/stable-watcher-boundary.ts";
  assert.throws(() =>
    validate([row], [], {
      [helper]: Buffer.from(
        fs
          .readFileSync(helper, "utf8")
          .replace(
            "assert.equal(context.source, historicalSource);",
            'assert.equal(context.source, "other");',
          ),
      ),
    }),
  );
  const types = "tests/tooling/support/native-beta-dto/audit-wire-types.ts";
  const typeBytes = fs.readFileSync(types, "utf8");
  for (const mutation of [
    typeBytes + "\nthrow new Error('unexpected executable type dependency');\n",
    typeBytes.replace("phase: string;", "phase: unknown;"),
  ])
    assert.throws(() => validate([row], [], { [types]: Buffer.from(mutation) }));
  const route = "tests/tooling/support/native-beta-dto/beta-attached-route.ts";
  for (const path of [
    "tests/tooling/support/native-beta-dto/native-io.ts",
    "tests/tooling/support/native-beta-dto/terminal-stderr.ts",
    "tests/tooling/support/native-beta-dto/terminal-stderr-unavailable.ts",
    "tests/tooling/support/native-beta-dto/configuration-membership.ts",
    "tests/tooling/support/native-beta-dto/semantic-route.ts",
    "tests/tooling/support/native-beta-dto/logical-revision.ts",
    "tests/tooling/support/native-beta-dto/logical-attached-route.ts",
    "tools/support/compat/github/native-beta-dto-current-custody.ts",
    "tests/tooling/support/native-beta-dto/historical-root-watch-warning.ts",
    "tests/tooling/support/native-beta-dto/role-disposition.ts",
    "tests/tooling/support/native-beta-dto/stable-watcher-timing.ts",
    "tests/tooling/support/native-beta-dto/stable-watcher-mixed.ts",
    "tests/tooling/support/native-beta-dto/stable-watcher-late-root.ts",
  ])
    assert.throws(() =>
      validate([row], [], {
        [path]: Buffer.concat([
          fs.readFileSync(path),
          Buffer.from("\ntype ChangedContract = unknown;\n"),
        ]),
      }),
    );
  assert.throws(() =>
    validate([row], [], {
      [route]: Buffer.from(
        fs
          .readFileSync(route, "utf8")
          .replace('assert.equal(context.role, "after")', 'assert.equal(context.role, "before")'),
      ),
    }),
  );
  assert.throws(() =>
    validate([row], [], {
      [helper]: Buffer.from(
        fs
          .readFileSync(helper, "utf8")
          .replace('assert.equal(context.role, "before",', 'assert.equal(context.role, "after",'),
      ),
    }),
  );
});
void test("only the finite registered source files enter this protocol family", () => {
  for (const path of [
    "crates/vize_canon/src/lsp_client/initial_diagnostics/diagnostic_response.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/protocol/protocol9.rs",
    "crates/vize_canon/src/lsp_client/session/capabilities.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/component_types.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/tests/native_beta/storage_authority.rs",
  ])
    assert(nativeBetaDtoProtocolSource(path));
  for (const path of [
    "crates/vize_canon/src/check.rs",
    "crates/vize_canon/src/lsp_client/initial_diagnostics.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/protocol/future.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/tests/native_beta/storage_authority/future.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/protocol/../protocol9.rs",
  ])
    assert(!nativeBetaDtoProtocolSource(path));
});
void test("stale, duplicate, inactive, unknown, unpinned and mutated inputs cannot qualify", () => {
  assert.deepEqual(validate([row], [row]), []);
  for (const rows of [
    [row, row],
    [{ ...row, id: "typechecker/protocol/forged" }],
    [{ ...row, state: "draft" }],
    [{ ...row, adapter: "other" }],
    [{ ...row, inputs: [] }],
    [{ ...row, targets: ["batch"] }],
    [{ ...row, extra: true }],
  ])
    assert.throws(() => validate(rows));
  const input = `tests/_fixtures/differential/typechecker/${row.inputs[0].path}`;
  assert.throws(() => validate([row], [], { [input]: null }));
  assert.throws(() =>
    validate([row], [], { [input]: Buffer.from("not a native source envelope") }),
  );
  const changed = structuredClone(row);
  changed.tests.pop();
  assert.throws(() => validate([changed]));
});
void test("raw complete ABI source identities, request pairs and producer files are mandatory", () => {
  for (const index of [2, 3]) {
    const name = `tests/_fixtures/differential/typechecker/${row.inputs[index].path}`;
    const original = JSON.parse(fs.readFileSync(name, "utf8"));
    for (const mutate of index === 2
      ? [
          (value: MutationInput) => (value.stdinBase64 = ""),
          (value: MutationInput) =>
            (value.stdoutBase64 = Buffer.from("Content-Length: 100\r\n\r\n{}").toString("base64")),
          (value: MutationInput) => (value.provider.gitHead = "0".repeat(40)),
          (value: MutationInput) => (value.stderrBase64 = Buffer.from("error").toString("base64")),
        ]
      : [
          (value: MutationInput) => (value.beforeRevision = "0".repeat(40)),
          (value: MutationInput) => (value.candidateProductionRevision = "0".repeat(40)),
          (value: MutationInput) => value.production.pop(),
          (value: MutationInput) => (value.production[0]!.afterSha256 = "0".repeat(64)),
          (value: MutationInput) => value.producerFiles.pop(),
          (value: MutationInput) => (value.producerFiles[0]!.sha256 = "0".repeat(64)),
        ]) {
      const value = structuredClone(original);
      mutate(value);
      const bytes = Buffer.from(JSON.stringify(value));
      const candidate = structuredClone(row);
      candidate.inputs[index].sha256 = hash(bytes);
      assert.throws(() => validate([candidate], [], { [name]: bytes }));
    }
  }
});

void test("shared report requires the genuine DTO row and keeps mixed Canon batch requirements", async () => {
  const { analyzeLegacyFix } =
    await import("../../tools/support/compat/github/report-legacy-fix-fixtures.ts");
  const dtoPath = "crates/vize_canon/src/lsp_client/initial_diagnostics/diagnostic_response.rs";
  const ordinary = Buffer.from("<template><div /></template>\n");
  const batch = {
    id: "typechecker/unit-only-ordinary/default",
    state: "active",
    targets: ["batch"],
    inputs: [{ path: "unit-only-ordinary/App.vue.txt", sha256: hash(ordinary) }],
  };
  const analyze = (rows: unknown[], paths = [dtoPath], additional: object[] = []) =>
    analyzeLegacyFix({
      title: "fix(canon): validate native beta diagnostics",
      paths,
      readFile: (side, path) => {
        if (path === manifestPath)
          return Buffer.from(
            JSON.stringify({
              ...manifest,
              cases: [...manifest.cases, ...(side === "head" ? additional : [])],
              protocolCases:
                side === "head"
                  ? rows
                  : manifest.protocolCases.filter(
                      (entry: { id: string }) => entry.id !== NATIVE_BETA_DTO_PROTOCOL_ID,
                    ),
            }),
          );
        if (
          side === "head" &&
          path === "tests/_fixtures/differential/typechecker/unit-only-ordinary/App.vue.txt"
        )
          return ordinary;
        return read(side, path);
      },
    }).products[0];
  assert.equal(analyze(manifest.protocolCases)?.state, "new-inputs-registered");
  const missing = manifest.protocolCases.filter(
    (entry: { id: string }) => entry.id !== NATIVE_BETA_DTO_PROTOCOL_ID,
  );
  assert.equal(analyze(missing)?.state, "new-input-missing");
  assert.equal(
    analyze(missing, [dtoPath], [batch])?.state,
    "new-input-missing",
    "ordinary structural input cannot replace genuine DTO protocol ownership",
  );
  assert.equal(
    analyze(manifest.protocolCases, [dtoPath, "crates/vize_canon/src/check.rs"])?.state,
    "new-input-missing",
  );
  assert.equal(
    analyze(manifest.protocolCases, [dtoPath, "crates/vize_canon/src/check.rs"], [batch])?.state,
    "new-inputs-registered",
  );
  assert.equal(analyze([...manifest.protocolCases, row])?.state, "invalid-manifest-or-input");
  assert.equal(
    analyze([...manifest.protocolCases, { ...row, id: "typechecker/protocol/unknown" }])?.state,
    "invalid-manifest-or-input",
  );
});
