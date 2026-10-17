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
import { targets } from "../../tools/support/release/public_acceptance/lsp_source.ts";
import { independentPublicObservationLaw } from "./support/release-public-independent-observation-law.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
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
