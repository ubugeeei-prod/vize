import { createRequire } from "node:module";

import { compareSfcEquivalence, sfcSemanticSignature } from "./sfc-equivalence.ts";
import type { SfcDialect } from "./sfc-baseline-routes.ts";
import {
  assertNoCompilerErrors,
  blockSignature,
  createBaselineProvenance,
  normalizeCompilerMessages,
  semanticSha256,
} from "./sfc-baseline-signatures.ts";
import type { SfcDescriptor } from "./sfc-baseline-signatures.ts";
import { vue2RenderSignature, vue27RenderCodeSignature } from "./vue2-render-signature.ts";

const require = createRequire(import.meta.url);

export type BaselineFailureStage =
  | "adapter-load"
  | "sfc-parse"
  | "template-compile"
  | "semantic-normalize";

export type BaselineFailure = {
  side: "original" | "formatted";
  stage: BaselineFailureStage;
  message: string;
};

export type SfcBaselineProvenance = {
  id: string;
  dialect: SfcDialect;
  package: string | null;
  version: string | null;
  entrySha256: string | null;
  normalization: string;
  options: Record<string, unknown>;
};

export type SfcBaselineComparison = {
  verdict: "equivalent" | "semantic-diff" | "baseline-unusable";
  reasonCode: string | null;
  differences: string[];
  failure: BaselineFailure | null;
  beforeSemanticSha256: string | null;
  afterSemanticSha256: string | null;
  baseline: SfcBaselineProvenance;
};

type SideResult =
  | { ok: true; signature: string }
  | { ok: false; stage: BaselineFailureStage; message: string };
type LoadedBaseline =
  | { compile: (source: string, filename: string) => SideResult; provenance: SfcBaselineProvenance }
  | { failure: Omit<BaselineFailure, "side">; provenance: SfcBaselineProvenance };
const baselineCache = new Map<SfcDialect, LoadedBaseline>();

export function compareSfcWithDialectBaseline(
  original: string,
  formatted: string,
  filename: string,
  dialect: SfcDialect,
): SfcBaselineComparison {
  const baseline = loadBaseline(dialect);
  if ("failure" in baseline) {
    return failedComparison("original", baseline.failure, baseline.provenance);
  }
  const before = baseline.compile(original, filename);
  if (!before.ok) return failedComparison("original", before, baseline.provenance);
  const after = baseline.compile(formatted, filename);
  if (!after.ok) return failedComparison("formatted", after, baseline.provenance);
  const beforeHash = semanticSha256(before.signature);
  const afterHash = semanticSha256(after.signature);
  const differences = dialect === "3" ? compareSfcEquivalence(original, formatted, filename) : [];
  if (beforeHash !== afterHash || differences.length > 0) {
    return {
      verdict: "semantic-diff",
      reasonCode: "semantic-signature-changed",
      differences:
        differences.length > 0
          ? differences
          : [`semantic signature changed: ${beforeHash} -> ${afterHash}`],
      failure: null,
      beforeSemanticSha256: beforeHash,
      afterSemanticSha256: afterHash,
      baseline: baseline.provenance,
    };
  }
  return {
    verdict: "equivalent",
    reasonCode: null,
    differences: [],
    failure: null,
    beforeSemanticSha256: beforeHash,
    afterSemanticSha256: afterHash,
    baseline: baseline.provenance,
  };
}

export function getSfcBaselineProvenance(dialect: SfcDialect): SfcBaselineProvenance {
  return loadBaseline(dialect).provenance;
}

function loadBaseline(dialect: SfcDialect): LoadedBaseline {
  const cached = baselineCache.get(dialect);
  if (cached != null) return cached;
  let loaded: LoadedBaseline;
  try {
    loaded = selectBaseline(dialect);
  } catch (error) {
    loaded = unsupportedBaseline(dialect, error);
  }
  baselineCache.set(dialect, loaded);
  return loaded;
}

function selectBaseline(dialect: SfcDialect): LoadedBaseline {
  switch (dialect) {
    case "2":
      return loadVue26Baseline();
    case "2.7":
      return loadVue27Baseline();
    case "3":
      return loadVue3Baseline();
    case "0.10":
    case "0.11":
    case "1":
      return unsupportedBaseline(dialect);
  }
}

function loadVue26Baseline() {
  const packageName = "vue-sfc-compiler-2-6";
  const entry = require.resolve(`${packageName}/build.js`);
  const packageJson = require(`${packageName}/package.json`) as { version: string };
  const compiler = require(entry) as {
    parseComponent: (source: string, options: object) => SfcDescriptor;
    compile: (
      source: string,
      options: object,
    ) => { render: string; staticRenderFns: string[]; errors: unknown[]; tips: unknown[] };
  };
  const compileOptions = { comments: true, outputSourceRange: true, whitespace: "condense" };
  const provenance = createBaselineProvenance(
    "vue2.6",
    "2",
    "vue-template-compiler",
    packageJson.version,
    entry,
    "vue2-render-v1",
    { parse: { pad: false }, compile: compileOptions },
  );
  return {
    provenance,
    compile: (source: string, filename: string): SideResult =>
      compileLegacySide(source, filename, compiler.parseComponent, (template) => {
        const result = compiler.compile(template, compileOptions);
        assertNoCompilerErrors(result.errors);
        return {
          render: vue2RenderSignature(result.render, result.staticRenderFns),
          tips: normalizeCompilerMessages(result.tips),
        };
      }),
  };
}

function loadVue27Baseline() {
  const packageName = "vue-sfc-compiler-2-7";
  const entry = require.resolve(packageName);
  const packageJson = require(`${packageName}/package.json`) as { version: string };
  const compiler = require(entry) as {
    parseComponent: (source: string, options: object) => SfcDescriptor;
    compileTemplate: (options: object) => { code: string; errors: unknown[]; tips: unknown[] };
  };
  const options = {
    isProduction: true,
    prettify: false,
    compilerOptions: { comments: true, outputSourceRange: true, whitespace: "condense" },
  };
  const provenance = createBaselineProvenance(
    "vue2.7",
    "2.7",
    "@vue/compiler-sfc",
    packageJson.version,
    entry,
    "vue2-render-v1",
    { parse: { pad: false }, compile: options },
  );
  return {
    provenance,
    compile: (source: string, filename: string): SideResult =>
      compileLegacySide(source, filename, compiler.parseComponent, (template) => {
        const result = compiler.compileTemplate({ source: template, filename, ...options });
        assertNoCompilerErrors(result.errors);
        return {
          render: vue27RenderCodeSignature(result.code),
          tips: normalizeCompilerMessages(result.tips),
        };
      }),
  };
}

function loadVue3Baseline() {
  const packageName = "@vue/compiler-sfc";
  const entry = require.resolve(packageName);
  const packageJson = require(`${packageName}/package.json`) as { version: string };
  const compiler = require(entry) as {
    parse: (source: string, options: object) => { descriptor: SfcDescriptor; errors: unknown[] };
  };
  const provenance = createBaselineProvenance(
    "vue3",
    "3",
    packageName,
    packageJson.version,
    entry,
    "vue3-template-ast-v1",
    { sourceMap: false },
  );
  return {
    provenance,
    compile(source: string, filename: string): SideResult {
      try {
        const parsed = compiler.parse(source, { filename, sourceMap: false });
        assertNoCompilerErrors(parsed.errors);
        return { ok: true, signature: sfcSemanticSignature(source, filename) };
      } catch (error) {
        return failure("sfc-parse", error);
      }
    },
  };
}

function compileLegacySide(
  source: string,
  filename: string,
  parse: (source: string, options: object) => SfcDescriptor,
  compile: (template: string) => unknown,
): SideResult {
  let descriptor: SfcDescriptor;
  try {
    descriptor = parse(source, { filename, pad: false });
  } catch (error) {
    return failure("sfc-parse", error);
  }
  try {
    const render =
      descriptor.template?.content == null ? null : compile(descriptor.template.content);
    return { ok: true, signature: JSON.stringify([blockSignature(descriptor), render]) };
  } catch (error) {
    return failure("template-compile", error);
  }
}

function unsupportedBaseline(dialect: SfcDialect, error?: unknown) {
  const provenance: SfcBaselineProvenance = {
    id: `unsupported-vue-${dialect}`,
    dialect,
    package: null,
    version: null,
    entrySha256: null,
    normalization: "unavailable",
    options: {},
  };
  return {
    provenance,
    failure: {
      stage: "adapter-load" as const,
      message:
        error == null
          ? `no official formatter baseline adapter is registered for Vue ${dialect}`
          : error instanceof Error
            ? error.message
            : (JSON.stringify(error) ?? "unknown adapter load failure"),
    },
  };
}

function failedComparison(
  side: "original" | "formatted",
  failureInput: Omit<BaselineFailure, "side">,
  baseline: SfcBaselineProvenance,
): SfcBaselineComparison {
  const failure = { side, ...failureInput };
  return {
    verdict: side === "original" ? "baseline-unusable" : "semantic-diff",
    reasonCode: side === "original" ? "original-baseline-unusable" : "formatted-baseline-unusable",
    differences: [`${failure.stage}: ${failure.message}`],
    failure,
    beforeSemanticSha256: null,
    afterSemanticSha256: null,
    baseline,
  };
}

function failure(stage: BaselineFailureStage, error: unknown): SideResult {
  return { ok: false, stage, message: error instanceof Error ? error.message : String(error) };
}
