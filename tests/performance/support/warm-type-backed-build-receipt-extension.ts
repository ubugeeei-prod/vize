import { spawnSync } from "node:child_process";
import { isDeepStrictEqual } from "node:util";
import { sha256 } from "../../differential/sha256.ts";

export type ReceiptExtensionSide = "before" | "after";
export type ReceiptExtensionReader = (
  side: ReceiptExtensionSide,
  file: string,
) => Buffer | string | null;

// Authenticated published #8310 source, rather than hashes of a future retry.
export const receiptExtensionSources = {
  before: "b6d2e15e046f7f51e9cf0f78e19e8f726e99a086",
  reviewed: "78940693d715f3b2961a02e373f13dcefa06d38c",
} as const;

export const receiptCallerPaths = [
  ".github/actions/check-computed-inlay/run.sh",
  ".github/actions/check-vue-parity/action.yml",
  ".github/workflows/check.yml",
  ".github/workflows/lsp-current-baseline.yml",
  ".github/workflows/n8n-adoption.yml",
  ".github/workflows/nuxt3-module-build.yml",
  ".github/workflows/pr-source-checks.yml",
  "tests/_helpers/realworld-build-errors.ts",
  "tests/differential/compiler.mjs",
  "tests/differential/formatter-css-grouping.ts",
  "tests/differential/formatter-history-cli.ts",
  "tests/differential/formatter-sorting-config.mjs",
  "tests/differential/formatter.mjs",
  "tests/differential/lsp-report.ts",
  "tests/differential/lsp.ts",
  "tests/fixtures/typechecker/vite-plus-relative-tsconfig/provider.ts",
  "tests/fixtures/typechecker/vite-plus-relative-tsconfig/runtime.ts",
  "tests/performance/lsp-current-baseline.ts",
  "tests/performance/support/current-baseline/audit.ts",
  "tests/performance/support/warm-type-backed-build.ts",
  "tests/tooling/art-lint-public-boundaries.test.mjs",
  "tests/tooling/boolean-attribute-fix-cli.test.mjs",
  "tests/tooling/canon-slot-outlet-union-semantics.test.ts",
  "tests/tooling/cli-build-enum-bindings-7893.test.mjs",
  "tests/tooling/cli-build-error-oracles.test.ts",
  "tests/tooling/cli-build-whitespace-7880.test.ts",
  "tests/tooling/css-external-target-cli.test.mjs",
  "tests/tooling/css-global-compound-cli.test.mjs",
  "tests/tooling/css-global-ownership-cli.test.mjs",
  "tests/tooling/css-id-selector-ranges.test.mjs",
  "tests/tooling/css-utility-literal-tokens.test.mjs",
  "tests/tooling/differential-compiler.test.mjs",
  "tests/tooling/differential-formatter.test.mjs",
  "tests/tooling/formatter-continuation-prefix-width.test.mjs",
  "tests/tooling/formatter-css-continuation-indent.test.mjs",
  "tests/tooling/formatter-css-rule-layout.test.mjs",
  "tests/tooling/formatter-discovery.test.mjs",
  "tests/tooling/formatter-document-support.test.mjs",
  "tests/tooling/formatter-expression-width-semantics.test.mjs",
  "tests/tooling/formatter-hugged-interpolation-indent.test.mjs",
  "tests/tooling/formatter-json-comments.test.mjs",
  "tests/tooling/formatter-leading-root-comment.test.mjs",
  "tests/tooling/formatter-preserved-template-text.test.mjs",
  "tests/tooling/formatter-root-comment-attachment.test.mjs",
  "tests/tooling/formatter-sequence-directive-indent.test.mjs",
  "tests/tooling/formatter-template-parens.test.mjs",
  "tests/tooling/formatter-ts-generic-arrow.test.mjs",
  "tests/tooling/formatter-utf8-bom.test.mjs",
  "tests/tooling/lint-aria-hidden-static-focus.test.mjs",
  "tests/tooling/lint-component-registration-spans.test.mjs",
  "tests/tooling/lint-content-directives.test.mjs",
  "tests/tooling/lint-cross-file-browser-origin.test.mjs",
  "tests/tooling/lint-cross-file-ssr-context.test.mjs",
  "tests/tooling/lint-default-correctness-cli.test.mjs",
  "tests/tooling/lint-p0-originals.test.mjs",
  "tests/tooling/lint-schema-side-effects.test.ts",
  "tests/tooling/lint-ssr-script-setup.test.ts",
  "tests/tooling/lint-with-defaults-macro-ownership.test.mjs",
  "tests/tooling/lsp-bind-style-code-actions.test.ts",
  "tests/tooling/lsp-code-action-diagnostics.test.ts",
  "tests/tooling/lsp-native-attribute-hover-range.test.ts",
  "tests/tooling/lsp-session-capture.test.ts",
  "tests/tooling/lsp-source-build-binding.test.ts",
  "tests/tooling/nuxt-key-order-comma-cli.test.mjs",
  "tests/tooling/slot-style-fix-cli.test.mjs",
  "tests/tooling/support/lsp/differential-reference.ts",
  "tests/tooling/support/lsp/launch.ts",
  "tests/tooling/support/lsp/session-capture.ts",
  "tests/tooling/support/n8n-cli-config-acceptance.mjs",
  "tests/tooling/support/n8n-cli-config-semantic-acceptance.mjs",
  "tests/tooling/support/typecheck/cli-jsconfig-runtime.ts",
  "tools/benchmarks/scripts/vue-benchmarks-current-lint.mjs",
  "tools/support/compat/nuxt/javascript-workspace-cli.mjs",
] as const;

const oldProvider = "tests/differential/build-receipt.mjs";
const provider = "tests/differential/build-receipt.ts";
const primitive = "tests/differential/sha256.ts";
const harness = "tests/differential/harness.mjs";
const checkTask = "tools/config/vite-plus/tasks/check.ts";
const project = "tsconfig.source-build-receipt.json";
const companion = "docs/davinci/decisions/2026-10-08-native-typescript-build-receipts.md";
export const receiptAssertionDigests = {
  "tests/tooling/differential-evidence-upload.test.ts":
    "1a689d970b3d47fb287a4c1903bcf5afcfb92bb51f53830a1f0eb6f33310edd9",
  "tests/tooling/github-workflows-source-selection.test.ts":
    "db6ec62d12cb9c39d3af2f80d91168201828fe56983863f382916e9d15f607ef",
  "tests/tooling/github-workflows-tooling-receipts.test.ts":
    "426b4b47985ca24e18cae99056c16b5dadc2bc53f71fa2f0620d0a4ae2f72fc4",
} as const;
// These two new implementation files are separately reviewed, finite scope.
// They have no fictional published789 body or self-referential future hash.
export const receiptQualificationPaths = [
  "tests/performance/support/warm-type-backed-build-receipt-extension.ts",
  "tests/tooling/source-build-receipt-extension.test.ts",
] as const;

export const receiptWitnessDigests = {
  "tests/_fixtures/tooling/source-build-receipt-extension/published-bodies.json.gz":
    "62ed8f4c8e0ebcbe0149f641bc1b836837c91b80953c3b774a6833e18605c7a9",
  "tests/_fixtures/tooling/source-build-receipt-extension/provenance.json":
    "e30d48c74b5146bc308765d37f12a4365dfbdcbf0a19c93afaec96dac3cdde41",
} as const;

export const receiptCompanionAppendix = `
The commit above records the original authoring boundary; genuine replays retain
that separate move-only boundary before typing.

The initial exact789 source run rejected the reviewed extension as an unqualified
harness delta before any original 400 provider execution. A finite source
qualifier now checks all 73 complete caller bodies against the sole \`.mjs\` to
\`.ts\` token change, authenticated original/current provider and hash-helper
bytes, exact primitive extraction, scoped project and check-command addition,
and three command assertions with their original checks.
Only the complete reviewed closure may enter the existing original 400 campaign;
its inputs, recipes, oracles and budgets remain unchanged. Failed run
37764288919/job113267998705 remains retained; no performance credit or retry
waiver follows from this source-extension admission.
`;

const sourceHashes = {
  originalProvider: "61267d16a5add01aa7dc18f534bc7f0eae346bd7cfab6719f0db065b96b69e91",
  currentProvider: "06043b859abd70e09923055f89fd6d0d6c7edd8edcc18a828f896f7faab055dd",
  primitive: "2edd1fc61e667219e59283a7ee9c65c4040b3f803f063df76ecdedac9df7197d",
  originalHarness: "ff3bb2fe68378b361e0e7f5eee7dd6d73b0074ce15ff211777bbbd765ed12219",
  originalCheckTask: "a70d603acbbb7bdb9983116a58fec9ba9a41100dafba19f8cf045857af0b99df",
  companion: "bff995cbc34e4e8909c6f6230b41a4dd314175f79b0f3177afddba77364dd15b",
} as const;

const scopedProject = {
  extends: "./tsconfig.node.json",
  compilerOptions: { composite: false, incremental: false },
  include: [],
  files: [provider, primitive, ...receiptQualificationPaths],
};

function text(value: Buffer | string | null): string | null {
  if (value === null) return null;
  if (typeof value === "string") return value;
  const decoded = value.toString("utf8");
  return Buffer.from(decoded).equals(value) ? decoded : null;
}

function digest(value: string | null): string | null {
  return value === null ? null : sha256(value);
}

export interface QualifiedReceiptExtension {
  authority: "complete-reviewed-source-build-receipt-extension";
  originalSource: string;
  reviewedSource: string;
  completeCallers: number;
  qualifiedFinitePaths: string[];
}

/** Preserve the old classifier; qualify this complete extension or nothing. */
export function qualifyBuildReceiptExtension(
  changed: readonly string[],
  alreadyQualified: ReadonlySet<string>,
  readBody: ReceiptExtensionReader,
): QualifiedReceiptExtension | null {
  if (new Set(changed).size !== changed.length) return null;
  const required = [
    ...receiptCallerPaths,
    ...Object.keys(receiptAssertionDigests),
    provider,
    primitive,
    harness,
    checkTask,
    project,
    companion,
    ...receiptQualificationPaths,
    ...Object.keys(receiptWitnessDigests),
  ];
  const finite = new Set([...required, oldProvider]);
  if (required.some((file) => !changed.includes(file))) return null;
  if (changed.some((file) => !finite.has(file) && !alreadyQualified.has(file))) return null;
  const bodies = new Map<string, Buffer | string | null>();
  const read = (side: ReceiptExtensionSide, file: string) => {
    const key = `${side}:${file}`;
    if (!bodies.has(key)) bodies.set(key, readBody(side, file));
    return bodies.get(key)!;
  };
  const body = (side: ReceiptExtensionSide, file: string) => text(read(side, file));
  for (const file of receiptCallerPaths) {
    const before = body("before", file);
    const after = body("after", file);
    if (before === null || after === null || !before.includes("build-receipt.mjs")) return null;
    if (before.replaceAll("build-receipt.mjs", "build-receipt.ts") !== after) return null;
  }
  for (const [file, expected] of Object.entries(receiptAssertionDigests)) {
    const before = body("before", file);
    if (before === null || digest(before) !== expected) return null;
    if (before.replaceAll("build-receipt\\.mjs", "build-receipt\\.ts") !== body("after", file))
      return null;
  }
  if (
    digest(body("before", oldProvider)) !== sourceHashes.originalProvider ||
    read("after", oldProvider) !== null ||
    read("before", provider) !== null ||
    digest(body("after", provider)) !== sourceHashes.currentProvider ||
    read("before", primitive) !== null ||
    digest(body("after", primitive)) !== sourceHashes.primitive
  )
    return null;
  const oldHarness = body("before", harness);
  if (digest(oldHarness) !== sourceHashes.originalHarness || oldHarness === null) return null;
  const extractedHarness = oldHarness
    .replace('import { createHash } from "node:crypto";', 'import { sha256 } from "./sha256.ts";')
    .replace(
      'export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");',
      'export { sha256 } from "./sha256.ts";',
    );
  if (body("after", harness) !== extractedHarness) return null;
  const oldCheck = body("before", checkTask);
  if (digest(oldCheck) !== sourceHashes.originalCheckTask || oldCheck === null) return null;
  const checkAnchor =
    '  "node tools/support/typescript/check-project.ts tsconfig.ci-gates.json",\n';
  const appendedCheck = oldCheck.replace(
    checkAnchor,
    checkAnchor +
      '  "node tools/support/typescript/check-project.ts tsconfig.source-build-receipt.json",\n',
  );
  if (body("after", checkTask) !== appendedCheck) return null;
  const currentProject = body("after", project);
  if (read("before", project) !== null || currentProject === null) return null;
  try {
    const parsed: unknown = JSON.parse(currentProject);
    if (!isDeepStrictEqual(parsed, scopedProject)) return null;
  } catch {
    return null;
  }
  const currentCompanion = body("after", companion);
  if (read("before", companion) !== null || currentCompanion === null) return null;
  const publishedCompanion = currentCompanion.endsWith(receiptCompanionAppendix)
    ? currentCompanion.slice(0, -receiptCompanionAppendix.length)
    : currentCompanion;
  if (digest(publishedCompanion) !== sourceHashes.companion) return null;
  for (const file of receiptQualificationPaths)
    if (read("before", file) !== null || !body("after", file)) return null;
  for (const [file, expected] of Object.entries(receiptWitnessDigests)) {
    const after = read("after", file);
    if (read("before", file) !== null || after === null || sha256(after) !== expected) return null;
  }
  return {
    authority: "complete-reviewed-source-build-receipt-extension",
    originalSource: receiptExtensionSources.before,
    reviewedSource: receiptExtensionSources.reviewed,
    completeCallers: receiptCallerPaths.length,
    qualifiedFinitePaths: [...required, oldProvider],
  };
}

/** Read actual immutable Git bytes; leave the old classifier every other path. */
export function unqualifiedReceiptPaths(
  changed: readonly string[],
  alreadyQualified: ReadonlySet<string>,
  driverRoot: string,
  before: string,
  after: string,
): readonly string[] {
  const oldQualified = new Set([
    ...alreadyQualified,
    ...changed.filter((file) =>
      /^tests\/performance\/support\/warm-type-backed-[a-z-]+\.ts$/u.test(file),
    ),
  ]);
  const qualified = qualifyBuildReceiptExtension(changed, oldQualified, (side, file) => {
    const revision = side === "before" ? before : after;
    const result = spawnSync("git", ["show", `${revision}:${file}`], { cwd: driverRoot });
    if (!result.error && result.signal === null && result.status === 0) return result.stdout;
    if (!result.error && result.signal === null && result.status === 128) return null;
    throw Object.assign(
      new Error(`Git receipt body read failed: ${result.stderr?.toString("utf8") ?? ""}`),
      {
        status: result.status,
        signal: result.signal,
        error: result.error,
        stdout: result.stdout,
        stderr: result.stderr,
      },
    );
  });
  if (!qualified) return changed;
  const finite = new Set(qualified.qualifiedFinitePaths);
  return changed.filter((file) => !finite.has(file));
}
