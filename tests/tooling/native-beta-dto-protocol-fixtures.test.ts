import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { test } from "node:test";
import {
  NATIVE_BETA_DTO_PROTOCOL_ID,
  nativeBetaDtoProducerCustody,
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
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
function read(side: "base" | "head", path: string): Buffer | null {
  try {
    return side === "head"
      ? fs.readFileSync(path)
      : execFileSync("git", ["show", `${before}:${path}`], { stdio: ["ignore", "pipe", "ignore"] });
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
  const loaded = loadTypecheckerManifest(manifestPath);
  assert.equal(
    loaded.cases.length,
    manifest.cases.filter((entry: { pack?: string }) => entry.pack !== undefined).length,
  );
  assert(!loaded.cases.some((entry) => entry.id === NATIVE_BETA_DTO_PROTOCOL_ID));
});
void test("typed current producers retain exact historical bytes and unchanged executable authority", () => {
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
void test("only the finite registered source files enter this protocol family", () => {
  for (const path of [
    "crates/vize_canon/src/lsp_client/initial_diagnostics/diagnostic_response.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/protocol/protocol9.rs",
    "crates/vize_canon/src/lsp_client/session/capabilities.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/component_types.rs",
  ])
    assert(nativeBetaDtoProtocolSource(path));
  for (const path of [
    "crates/vize_canon/src/check.rs",
    "crates/vize_canon/src/lsp_client/initial_diagnostics.rs",
    "crates/vize_canon/src/lsp_client/editor_lsp/snapshot_source/protocol/future.rs",
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
