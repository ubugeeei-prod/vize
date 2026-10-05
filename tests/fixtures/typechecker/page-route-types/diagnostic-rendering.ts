import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export type DiagnosticRow = {
  file: string;
  line: number;
  column: number;
  code: number;
  message: string;
};

export type DiagnosticRenderCase = "define-page-unknown-id";

export type DiagnosticRenderContext = {
  renderCase?: DiagnosticRenderCase;
  sourcePath: string;
  source: string;
  providerArchiveSha256?: string;
  generatedRoutesSha256?: string;
};

const PAGE = "packages/playground-file-based/src/pages/users/[userId=int].vue";
const SOURCE_SHA = "fc7bc0530fc13ee5fcc67cba6f3e54b1a86280fdf80caf679e1fedbf58199844";
const PROVIDER_SHA = "5c0bf884438b9c58b1e926663e07572c80c5dcc8d0ddd57a32943a0161d8ee5f";
const ROUTES_SHA = "1a19bf3a6f143d7b7ee5c931d0f8da9a4e0a06f378278fc2615f99db746d95c8";

// Original full cf/37241714582 observation 073; two engine renders, not message parity.
// Provider index-BQLwgiyK.d.ts:1626,1637–1641,1674,1687–1714 owns the type.
// Original routes:38–51 registers thirteen custom parsers;811–817 permits only userId.
// These constants freeze each whole render; no sorting, expansion or normalization.
const NATIVE_MESSAGE =
  'Object literal may only specify known properties, and \'unknownId\' does not exist in type \'{ userId?: "date" | "month-valibot" | "month-zod" | "npm-org" | "semver" | "set" | "test-bool-q" | "test-color" | "test-csv" | "test-num" | "test-set" | "test-set-shape" | "version-range" | keyof ParamParsers_Native | undefined; }\'.';
const REFERENCE_MESSAGE =
  'Object literal may only specify known properties, and \'unknownId\' does not exist in type \'{ userId?: keyof ParamParsers_Native | "date" | "month-valibot" | "month-zod" | "npm-org" | "semver" | "set" | "test-bool-q" | "test-color" | "test-csv" | ... 4 more ... | undefined; }\'.';

/** Validate native rows for subsequent complete CLI/editor comparisons. */
export function comparePageDiagnostics(
  actual: DiagnosticRow[],
  reference: DiagnosticRow[],
  context: DiagnosticRenderContext,
): DiagnosticRow[] {
  if (context.renderCase === undefined) {
    assert.deepEqual(actual, reference);
    return actual;
  }

  assert.equal(context.renderCase, "define-page-unknown-id", "unknown diagnostic render case");
  assert.equal(context.sourcePath, PAGE, "the render contract belongs to the original page");
  assert.equal(
    createHash("sha256").update(context.source, "utf8").digest("hex"),
    SOURCE_SHA,
    "the render contract requires the exact observed authored source",
  );
  assert.equal(
    context.providerArchiveSha256,
    PROVIDER_SHA,
    "the complete provider must stay pinned",
  );
  assert.equal(
    context.generatedRoutesSha256,
    ROUTES_SHA,
    "the complete original map must stay pinned",
  );
  const location = { file: PAGE, line: 16, column: 7, code: 2353 };
  assert.deepEqual(actual, [{ ...location, message: NATIVE_MESSAGE }], "full native render vector");
  assert.deepEqual(
    reference,
    [{ ...location, message: REFERENCE_MESSAGE }],
    "full official-JS render vector",
  );
  return actual;
}
