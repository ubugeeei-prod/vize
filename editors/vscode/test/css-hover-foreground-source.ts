import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { isDeepStrictEqual } from "node:util";

export type SourceTreeEntry = Readonly<{ path: string; mode: string; type: string; oid: string }>;
type SourceLayer = Readonly<{
  name: string;
  reviewedRevision: string;
  sourceSha256: string;
  inverseSha256: string;
  changes: readonly Readonly<{ path: string; before: SourceTreeEntry; after: SourceTreeEntry }>[];
}>;
type CheckoutComposition = Readonly<{
  entries: readonly SourceTreeEntry[];
  layer: SourceLayer | null;
}>;

export const typedSourcePath = "crates/vize_maestro/src/server/handlers/typed.rs";
export const originalTypedSourceSha256 =
  "a287bbc3060de2a5c01963fe82962501b5f688091c912ec10e97adc402e6859a";
const reviewedTypedSourceSha256 =
  "752179a706971407290c7144720bbcbc63261031bf6b5378bdb8cc9427b601ac";
const before: SourceTreeEntry = Object.freeze({
  path: typedSourcePath,
  mode: "100644",
  type: "blob",
  oid: "d378281fbcda6dda2f550e21fa3ccde3905d8a7d",
});
const after = Object.freeze({
  ...before,
  oid: "a4b1e81960029f488921f917e46589f1d7655da9",
});
const hash = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");
const blob = (bytes: Uint8Array): string =>
  createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");

const hoverHeader = `    async fn hover_request_with_preparation(
        &self,
        params: HoverParams,
        preparation: Option<(&tower_lsp::lsp_types::Url, &std::sync::atomic::AtomicBool)>,
    ) -> Result<Option<Hover>> {
        #[cfg(not(feature = "native"))]
        let _ = preparation;
`;
const oldHoverHeader =
  "    pub(super) async fn hover_request(&self, params: HoverParams) -> Result<Option<Hover>> {\n";
const completionHeader = `    async fn completion_request_with_preparation(
        &self,
        params: CompletionParams,
        preparation: Option<(&tower_lsp::lsp_types::Url, &std::sync::atomic::AtomicBool)>,
    ) -> Result<Option<CompletionResponse>> {
        #[cfg(not(feature = "native"))]
        let _ = preparation;
`;
const oldCompletionHeader = `    pub(super) async fn completion_request(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
`;
const contextAttachment = `        #[cfg(feature = "native")]
        let ctx = ctx.with_foreground_preparation(preparation);
`;
const documentDispatch = `
impl super::super::project_contexts::DocumentServer<'_> {
    pub(super) async fn hover_request(&self, params: HoverParams) -> Result<Option<Hover>> {
        MaestroServer::hover_request_with_preparation(self, params, self.foreground_preparation())
            .await
    }

    pub(super) async fn completion_request(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        MaestroServer::completion_request_with_preparation(
            self,
            params,
            self.foreground_preparation(),
        )
        .await
    }
}
`;

function replaceExactly(
  source: string,
  selected: string,
  replacement: string,
  count: number,
): string {
  const pieces = source.split(selected);
  assert.equal(pieces.length, count + 1, "foreign foreground source structure");
  return pieces.join(replacement);
}

// Authenticate the complete current body, perform the finite source inverse,
// then authenticate its complete original body. No CSS code is normalized.
export function invertForegroundPreparationSource(bytes: unknown): Buffer {
  assert.ok(Buffer.isBuffer(bytes), "complete committed source bytes required");
  assert.equal(hash(bytes), reviewedTypedSourceSha256, "unreviewed typed source bytes");
  assert.equal(blob(bytes), after.oid, "unreviewed typed source blob");
  let original = replaceExactly(bytes.toString("utf8"), hoverHeader, oldHoverHeader, 1);
  original = replaceExactly(original, completionHeader, oldCompletionHeader, 1);
  original = replaceExactly(original, contextAttachment, "", 2);
  original = replaceExactly(original, documentDispatch, "", 1);
  const restored = Buffer.from(original);
  assert.equal(hash(restored), originalTypedSourceSha256, "whole typed inverse differs");
  assert.equal(blob(restored), before.oid, "whole original typed blob differs");
  return restored;
}

// This changes only the expected current-checkout composition. The caller must
// still compare its ENTIRE actual owned tree with the returned entries.
export function composeForegroundPreparationCheckout(
  expected: readonly SourceTreeEntry[],
  actual: readonly SourceTreeEntry[],
  readSource: (name: string) => unknown,
): CheckoutComposition {
  assert.equal(new Set(expected.map((row) => row.path)).size, expected.length);
  assert.equal(new Set(actual.map((row) => row.path)).size, actual.length);
  const original = expected.find((row) => row.path === typedSourcePath);
  const current = actual.find((row) => row.path === typedSourcePath);
  if (!original || !current) return { entries: expected, layer: null };
  assert.deepEqual(original, before, "foreign original typed row");
  if (isDeepStrictEqual(current, before)) return { entries: expected, layer: null };
  assert.deepEqual(current, after, "foreign current typed row");
  invertForegroundPreparationSource(readSource(typedSourcePath));
  return {
    entries: expected.map((row) => (row.path === typedSourcePath ? current : row)),
    layer: {
      name: "same-document-foreground-preparation-11347",
      reviewedRevision: "1248e5a903c6de56f2e4630650a048744d135edf",
      sourceSha256: reviewedTypedSourceSha256,
      inverseSha256: originalTypedSourceSha256,
      changes: [{ path: typedSourcePath, before, after }],
    },
  };
}

const compareStrings = (a: string, b: string): number => (a < b ? -1 : a > b ? 1 : 0);

export function assertExactCssTree(
  actual: readonly SourceTreeEntry[],
  expected: readonly SourceTreeEntry[],
): void {
  const current = new Map(actual.map((entry) => [entry.path, entry]));
  const reviewed = new Map(expected.map((entry) => [entry.path, entry]));
  assert.equal(current.size, actual.length, "duplicate actual tree path");
  assert.equal(reviewed.size, expected.length, "duplicate reviewed tree path");
  const differences = [...new Set([...current.keys(), ...reviewed.keys()])]
    .sort(compareStrings)
    .filter((name) => !isDeepStrictEqual(current.get(name), reviewed.get(name)))
    .map((name) => ({
      path: name,
      actual: current.get(name) ?? null,
      expected: reviewed.get(name) ?? null,
    }));
  assert.deepEqual(
    differences,
    [],
    "current crates tree differs from reviewed blob/mode composition",
  );
}
