import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { fixtureLanguageId, loadLspManifest } from "../differential/lsp-manifest.ts";
import { inspectLspObservation } from "../differential/lsp-report.ts";
import type { FixtureData } from "../differential/lsp-types.ts";
import { referenceObservation, rewriteFrames } from "./support/lsp/differential-reference.ts";
import { freshVueSessionBinding } from "./support/lsp/fresh-vue-project.ts";
import { root } from "./support/lsp/paths.ts";
import type { PublishedSessionBinding } from "./support/lsp/session-process.ts";

test("authored LSP profiles preserve the historical default and reject unsupported values", () => {
  const loaded = loadLspManifest(path.join(root, "tests/_fixtures/differential/lsp/manifest.json"));
  assert.equal(fixtureLanguageId(loaded.cases[0].data), "vue");
  assert.deepEqual(
    loaded.cases
      .filter((fixture) => fixture.data.languageId !== undefined)
      .map((fixture) => [fixture.id, fixture.data.languageId]),
    [
      ["lsp/regression/jsx-intrinsic-hover-10521-jsx", "javascriptreact"],
      ["lsp/regression/jsx-intrinsic-hover-10521-tsx", "typescriptreact"],
    ],
    "historical rows retain their original didOpen profiles",
  );
  for (const languageId of ["vue", "javascriptreact", "typescriptreact"] as const) {
    const fixture = { ...loaded.cases[0], data: { ...loaded.cases[0].data, languageId } };
    assert.equal(fixtureLanguageId(fixture.data), languageId);
    assert.deepEqual(inspectLspObservation(fixture, referenceObservation(fixture)), [
      { requestId: 2, state: "equal" },
    ]);
    const changed = referenceObservation(fixture);
    rewriteFrames(changed, "client", (messages) => {
      const params = messages[2].params as { textDocument: { languageId: string } };
      params.textDocument.languageId = "typescript";
    });
    assert.throws(() => inspectLspObservation(fixture, changed));
  }
  for (const languageId of [null, false, 0, "", "typescript", "javascript", "pug", "Vue"]) {
    assert.throws(
      () => fixtureLanguageId({ languageId } as unknown as FixtureData),
      /unsupported authored LSP document profile/,
    );
  }
});

test("fresh Vue sessions retain source custody unless publication authority is explicit", () => {
  const binary = path.join(root, "target/ci/vize");
  assert.deepEqual(freshVueSessionBinding(binary, undefined, false), { repoRoot: root, binary });
  assert.deepEqual(freshVueSessionBinding(binary, undefined, true), { repoRoot: root, binary });
  const published: PublishedSessionBinding = {
    authority: "published-release",
    receiptPath: "/owned/installation.json",
    receiptSha256: "a".repeat(64),
    captureRoot: "/owned/capture",
  };
  assert.deepEqual(freshVueSessionBinding(binary, published, false), {
    repoRoot: root,
    binary,
    published,
  });
  assert.throws(
    () => freshVueSessionBinding(binary, published, true),
    /published authority cannot satisfy requested source-built custody/,
  );
});
