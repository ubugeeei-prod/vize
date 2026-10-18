import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  rejectLspOverrides,
  contained,
  publicLspCli,
} from "../../tools/support/release/public_acceptance/lsp_assets.ts";
import {
  assertBuiltSupport,
  assertSupportGraph,
} from "../../tools/support/release/public_acceptance/lsp_cargo.ts";
import {
  bindExpected,
  assertWholeCase,
} from "../../tools/support/release/public_acceptance/lsp_case.ts";
import {
  supportDependencyVersions,
  targets,
} from "../../tools/support/release/public_acceptance/lsp_source.ts";
import { independentPublicObservationLaw } from "./support/release-public-independent-observation-law.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
void test("all three original LSP targets inherit their direct registry imports from H", () => {
  // runtime_distribution.rs uses url::Url even when its own tests are filtered
  // out: Cargo must compile the complete original target before --exact runs.
  const inherited = {
    serde_json: "=1.2.3",
    tempfile: { version: "=4.5.6" },
    sha2: "=7.8.9",
    "lsp-types": { version: "=10.11.12" },
    url: { version: "=13.14.15" },
  };
  assert.deepEqual(supportDependencyVersions(inherited), {
    serde_json: "=1.2.3",
    tempfile: "=4.5.6",
    sha2: "=7.8.9",
    "lsp-types": "=10.11.12",
    url: "=13.14.15",
  });
  assert.deepEqual(supportDependencyVersions({ ...inherited, url: "=16.17.18" }).url, "=16.17.18");
  for (const name of Object.keys(inherited)) {
    const missing = { ...inherited } as Record<string, string | { version: string }>;
    delete missing[name];
    assert.throws(() => supportDependencyVersions(missing), /support dependency absent from H/u);
    for (const version of ["^1.2.3", "1.2.3", "*", "=1.2.3\nforeign = 1"])
      assert.throws(() => supportDependencyVersions({ ...inherited, [name]: version }));
  }
});
void test("public rename refuses ambient native/provider/compiler authority", () => {
  for (const key of [
    "CARGO_BIN_EXE_vize",
    "VIZE_CORSA_DEV_PATHS",
    "VIZE_TEST_DISABLE_TSGO",
    "VIZE_TEST_REQUIRE_TSGO",
    "VIZE_TEST_CONTENT_MAPPER_TSGO",
    "VIZE_TEST_CONTENT_MAPPER_VUE",
    "SOURCE_SHA",
    "RUSTC_WRAPPER",
    "CORSA_EXECUTABLE",
    "TSGO_PATH",
  ])
    assert.throws(() => rejectLspOverrides({ [key]: "inherited" }), /override must be absent/u);
  assert.doesNotThrow(() => rejectLspOverrides({}));
});
void test("an external provider symlink cannot become consumer authority", (t) => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-public-provider-refusal-"));
  t.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
  const consumer = path.join(temporary, "consumer");
  fs.mkdirSync(consumer);
  const foreign = path.join(temporary, "foreign");
  fs.writeFileSync(foreign, "inert bytes");
  fs.symlinkSync(foreign, path.join(consumer, "provider"));
  assert.throws(
    () => contained(path.join(consumer, "provider"), consumer),
    /outside consumer authority/u,
  );
});
void test("a checkout-local executable consumer is refused before any public read", async (t) => {
  const original = globalThis.fetch;
  let reads = 0;
  globalThis.fetch = async () => {
    reads++;
    throw new Error("inert forbidden public read");
  };
  t.after(() => {
    globalThis.fetch = original;
  });
  await assert.rejects(publicLspCli("unexecuted", 0, root, root), assert.AssertionError);
  assert.equal(reads, 0);
});
void test("the adapter cannot reach a product through a transitive normal/build edge", () => {
  const adapter = {
    id: "adapter",
    name: "vize_public_lsp_contracts",
    version: "0.999.0",
    source: null,
    manifest_path: "unused",
    targets: targets.map((name) => ({ name, kind: ["test"] })),
  };
  const product = {
    id: "product",
    name: "vize_canon",
    version: "0.999.0",
    source: null,
    manifest_path: "unused",
    targets: [],
  };
  for (const kind of [null, "build"] as const)
    assert.throws(
      () =>
        assertSupportGraph(
          {
            packages: [adapter, product],
            resolve: {
              root: adapter.id,
              nodes: [
                { id: adapter.id, deps: [{ pkg: product.id, dep_kinds: [{ kind }] }] },
                { id: product.id, deps: [] },
              ],
            },
          },
          root,
          path.join(root, "Cargo.lock"),
        ),
      /source product forbidden: vize_canon/u,
    );
  assert.throws(
    () =>
      assertBuiltSupport(
        '{"reason":"compiler-artifact","package_id":"product"}\n{"reason":"build-finished","success":true}',
        ["adapter"],
      ),
    /compiled product outside support graph/u,
  );
});
void test("missing original native guard or changed whole oracle cannot pass", () => {
  const file = path.join(
    root,
    "tests/_fixtures/differential/lsp/event-rename/4075/named-model/cases.json.txt",
  );
  const oracle = JSON.parse(fs.readFileSync(file, "utf8")).cases[0];
  const uris = Object.fromEntries(
    ["Child.vue", "Parent.vue", "Other.vue", "NativeGuard.vue"].map((file) => [
      file,
      `file:///inert/${file}`,
    ]),
  );
  // These refusal packets never stand in for a native reply or public acceptance.
  assert.throws(() =>
    assertWholeCase({ oracle, actual: { nativeGuard: null } }, oracle, uris, true),
  );
  assert.throws(
    () => assertWholeCase({ oracle: { ...oracle, newName: "foreign" } }, oracle, uris, true),
    /whole literal pre-query corpus/u,
  );
  assert.throws(() => bindExpected("$Foreign.vue", uris), /unknown expected URI marker/u);
});
void test("a red or absent LSP transaction does not block the independent observations or seal success", () => {
  independentPublicObservationLaw();
});

void test("all 160 declaration-origin definitions require the whole authored source target", () => {
  type Json = ReturnType<typeof JSON.parse>;
  const literal = JSON.parse(
    fs.readFileSync(
      path.join(
        root,
        "tests/_fixtures/differential/lsp/event-rename/4075/bound-emitter/cases.json.txt",
      ),
      "utf8",
    ),
  );
  const names = ["Child.vue", "Parent.vue", "Other.vue"];
  const uris = Object.fromEntries(
    [...names, "NativeGuard.vue"].map((file) => [file, `file:///inert/${file}`]),
  );
  const publication = (file: string, version: number, diagnostics: unknown[] = []) => ({
    jsonrpc: "2.0",
    method: "textDocument/publishDiagnostics",
    params: { uri: uris[file], version, diagnostics },
  });
  const reply = (id: number, result: unknown) => ({ jsonrpc: "2.0", id, result });
  let declarationCursors = 0;
  for (const oracle of literal.cases) {
    // Inert refusal fixtures use only pre-query source contracts, never captured replies.
    const invalid = publication("NativeGuard.vue", 1, [
      {
        range: { start: { line: 1, character: 6 }, end: { line: 1, character: 17 } },
        severity: 1,
        code: 2322,
        source: "vize/types",
        message: "Type 'string' is not assignable to type 'number'.",
      },
    ]);
    const repaired = publication("NativeGuard.vue", 2);
    const installed = (version: number) =>
      names.map((file) => ({
        file,
        text: oracle.completeGoldenFiles[file],
        disk: oracle.completeGoldenFiles[file],
        version,
        diagnostics: publication(file, version),
      }));
    const references = oracle.fullReferences.map((site: Json) => ({
      uri: uris[site.file],
      range: site.range,
    }));
    const changes: Record<string, Json[]> = {};
    for (const edit of oracle.fullEdits)
      (changes[uris[edit.file]] ??= []).push({ range: edit.range, newText: edit.newText });
    const site = oracle.definitionFromAllParentAndCallPositions[0];
    assert.equal(site.file, "Child.vue");
    assert.equal(site.range.start.line, site.range.end.line);
    const line = oracle.sources[site.file].split(/\r?\n/u)[site.range.start.line];
    assert.equal(line.slice(site.range.start.character, site.range.end.character), "saveItem");
    for (let index = 0; index < 8; index++)
      assert.deepEqual(oracle.queries[index], {
        file: site.file,
        position: { ...site.range.start, character: site.range.start.character + index },
      });
    const definition = { uri: uris[site.file], range: site.range };
    const queries = oracle.queries.map((query: Json, index: number) => ({
      query,
      referencesReply: reply(2 + index * 3, references),
      renameReply: reply(4 + index * 3, { changes }),
      definitionContracted: true,
      allowedWholeDefinitionResponses: [
        reply(3 + index * 3, definition),
        reply(3 + index * 3, [definition]),
      ],
    }));
    const allowed = [
      invalid,
      repaired,
      ...names.flatMap((file) => [1, 2, 3].map((version) => publication(file, version))),
    ];
    const packet = {
      oracle,
      allDefinitionOriginsContracted: true,
      expected: {
        queries,
        selectedApplicationQueryIndex: 0,
        version2: installed(2),
        independentVersion3: installed(3),
        allDefinitionOriginsContracted: true,
        allowedWholeDiagnosticPublications: allowed,
      },
      actual: {
        nativeGuard: {
          invalidDiagnostics: invalid,
          repairedDiagnostics: repaired,
          expectedInvalid: invalid,
        },
        initialDiagnostics: names.map((file) => publication(file, 1)),
        originalDiskAfterQueries: names.map((file) => ({ file, text: oracle.sources[file] })),
        applicationError: null,
        selectedApplicationQueryIndex: 0,
        appliedFiles: installed(2),
        independentGoldenFiles: installed(3),
        allDiagnosticPublications: allowed,
        observations: queries.map(({ allowedWholeDefinitionResponses, ...query }: Json) => ({
          ...query,
          definitionReply: allowedWholeDefinitionResponses[0],
        })),
      },
    };
    assert.doesNotThrow(() => assertWholeCase(packet, oracle, uris, false));
    for (let index = 0; index < 8; index++) {
      const observation = packet.actual.observations[index];
      const original = observation.definitionReply;
      for (const result of [
        null,
        [],
        { ...definition, uri: "file:///inert/Child.vue.ts" },
        { ...definition, range: { ...definition.range, end: definition.range.start } },
        { ...definition, extra: true },
      ]) {
        observation.definitionReply = reply(3 + index * 3, result);
        assert.throws(
          () => assertWholeCase(packet, oracle, uris, false),
          /whole contracted definition response/u,
        );
      }
      observation.definitionReply = queries[index].allowedWholeDefinitionResponses[1];
      assert.doesNotThrow(() => assertWholeCase(packet, oracle, uris, false));
      observation.definitionReply = original;
      observation.definitionContracted = false;
      assert.throws(() => assertWholeCase(packet, oracle, uris, false));
      observation.definitionContracted = true;
      declarationCursors++;
    }
    packet.allDefinitionOriginsContracted = false;
    assert.throws(() => assertWholeCase(packet, oracle, uris, false));
  }
  assert.equal(declarationCursors, 160);
});
