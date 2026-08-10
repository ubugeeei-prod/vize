import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

import {
  compareSfcBlockStructure,
  compareTemplateAstEquivalence,
  templateAstSemanticSignature,
} from "./sfc-equivalence.ts";
import type { TemplateNode } from "./sfc-equivalence.ts";

const require = createRequire(import.meta.url);
const pug = require("pug") as {
  compile: (
    source: string,
    options: Record<string, unknown>,
  ) => ((locals: Record<string, unknown>) => string) & { dependencies: string[] };
  compileClientWithDependenciesTracked: (
    source: string,
    options: Record<string, unknown>,
  ) => { body: string; dependencies: string[] };
};
const lexPug = createRequire(require.resolve("pug"))("pug-lexer") as (
  source: string,
  options: { filename: string },
) => Array<{ type: string; val?: string | boolean; name?: string }>;
const {
  baseParse: parseVueTemplate,
  compile: compileVueTemplate,
  parserOptions: vueParserOptions,
} = require("@vue/compiler-dom") as {
  baseParse: (source: string, options: Record<string, unknown>) => TemplateNode;
  compile: (
    source: string,
    options: Record<string, unknown>,
  ) => {
    ast: TemplateNode;
    code: string;
    map?: unknown;
  };
  parserOptions: Record<string, unknown>;
};
const { parse: parseSfc } = require("@vue/compiler-sfc") as {
  parse: (
    source: string,
    options: { filename: string; sourceMap: boolean },
  ) => {
    descriptor: {
      template: {
        content: string;
        lang?: string;
        loc: { start: { offset: number }; end: { offset: number } };
      } | null;
    };
  };
};

const pugPackage = require("pug/package.json") as { version: string };
const compilerDomPackage = require("@vue/compiler-dom/package.json") as { version: string };
const compilerSfcPackage = require("@vue/compiler-sfc/package.json") as { version: string };

const PUG_INTEGRITY =
  "sha512-kFfq5mMzrS7+wrl5pLJzZEzemx34OQ0w4SARfhy/3yxTlhbstsudDwJzhf1hP02yHzbjoVMSXUj/Sz6RNfMyXg==";
const COMPILER_DOM_INTEGRITY =
  "sha512-k+bprkXxuqhVajgTx5mUHuir7TwQzUKOWR40ng1ncAqQRPnrLngGGgqVEEhOnTMlc8btHYVKmrP8s5Qyg0hvYA==";

if (pugPackage.version !== "3.0.4" || compilerDomPackage.version !== "3.5.35") {
  throw new Error(
    `Pug oracle package drift: pug=${pugPackage.version}, ` +
      `@vue/compiler-dom=${compilerDomPackage.version}`,
  );
}

const pugOptions = {
  compileDebug: false,
  doctype: "html",
  pretty: false,
  globals: [] as string[],
  filters: [] as string[],
};
const vueCompilerOptions = {
  mode: "module",
  prefixIdentifiers: true,
  hoistStatic: true,
  cacheHandlers: false,
  whitespace: "preserve",
  comments: true,
  sourceMap: true,
};

export const PUG_ORACLE_BASELINE = Object.freeze({
  pug: { package: "pug", version: pugPackage.version, integrity: PUG_INTEGRITY },
  vueCompiler: {
    package: "@vue/compiler-dom",
    version: compilerDomPackage.version,
    integrity: COMPILER_DOM_INTEGRITY,
  },
  sfcEnvelopeParser: { package: "@vue/compiler-sfc", version: compilerSfcPackage.version },
  dialectContext: "fixed-vue3",
  mapBasis: "preprocessed-html",
  authoredPugMapAvailable: false,
  executionPolicy: "static-pug-only-with-explicit-filters",
  pugOptions,
  vueCompilerOptions,
  optionsSha256: sha256(JSON.stringify({ pugOptions, vueCompilerOptions })),
});

type PugFilter = (source: string, options: Record<string, unknown>) => string;
export type PugOracleContext = {
  filename: string;
  displayFilename: string;
  basedir: string;
  filters?: Record<string, PugFilter>;
};

type Diagnostic = {
  code?: number | string;
  name?: string;
  message?: string;
  loc?: { start?: { line?: number; column?: number; offset?: number } };
};
type CapturedDiagnostic = { severity: "error" | "warning"; value: Diagnostic };

type SideEvidence = {
  sourceSha256: string;
  pugBodySha256: string | null;
  relativePugSha256: string | null;
  preprocessedHtmlSha256: string | null;
  normalizedRenderSha256: string | null;
  renderCodeSha256: string | null;
  diagnosticsSha256: string | null;
  diagnosticLocationsSha256: string | null;
  sourceMapSha256: string | null;
  templateOffsets: { start: number; end: number } | null;
  dependencies: Array<{ path: string; sha256: string }>;
  error: string | null;
};

export type PugOracleEvidence = {
  contextSha256: string;
  pristine: SideEvidence;
  formatted: SideEvidence;
  sourceMapMoved: boolean | null;
  templateOffsetsMoved: boolean | null;
};

export type PugOracleComparison = {
  differences: string[];
  baselineUsable: boolean;
  evidence: PugOracleEvidence;
};

export function isPugSfc(source: string, filename: string): boolean {
  const template = parseSfc(source, { filename, sourceMap: false }).descriptor.template;
  return template?.lang?.toLowerCase() === "pug";
}

export function comparePugTemplateEquivalence(
  original: string,
  formatted: string,
  context: PugOracleContext,
): PugOracleComparison {
  const differences = compareSfcBlockStructure(original, formatted, context.filename);
  const pristine = compileSide(original, context);
  const after = compileSide(formatted, context);
  const baselineUsable = pristine.compiled != null && pristine.diagnostics.length === 0;

  if (pristine.error != null) differences.push(`pristine Pug baseline failed: ${pristine.error}`);
  if (pristine.diagnostics.length > 0) {
    differences.push(`pristine Vue baseline failed: ${diagnosticSummary(pristine.diagnostics)}`);
  }
  if (after.error != null) differences.push(`formatted Pug baseline failed: ${after.error}`);
  if (after.diagnostics.length > 0) {
    differences.push(`formatted Vue baseline failed: ${diagnosticSummary(after.diagnostics)}`);
  }

  if (pristine.compiled != null && after.compiled != null) {
    if (pristine.evidence.relativePugSha256 !== after.evidence.relativePugSha256) {
      differences.push("relative authored Pug bytes changed");
    }
    const beforeDiagnostics = diagnosticSignatures(pristine.diagnostics);
    const afterDiagnostics = diagnosticSignatures(after.diagnostics);
    if (JSON.stringify(beforeDiagnostics) !== JSON.stringify(afterDiagnostics)) {
      differences.push(
        `Vue compiler diagnostics changed: ${JSON.stringify(beforeDiagnostics)} -> ` +
          JSON.stringify(afterDiagnostics),
      );
    }
    const dependencyBefore = JSON.stringify(pristine.evidence.dependencies);
    const dependencyAfter = JSON.stringify(after.evidence.dependencies);
    if (dependencyBefore !== dependencyAfter) {
      differences.push(`Pug dependencies changed: ${dependencyBefore} -> ${dependencyAfter}`);
    }
    differences.push(
      ...compareTemplateAstEquivalence(pristine.semanticAst!, after.semanticAst!).map(
        (difference) => `Pug render ${difference}`,
      ),
    );
  }

  return {
    differences,
    baselineUsable,
    evidence: {
      contextSha256: sha256(
        JSON.stringify({
          displayFilename: context.displayFilename,
          basedirMode: "registry-project-root",
          filters: Object.entries(context.filters ?? {})
            .map(([name, filter]) => [name, sha256(String(filter))])
            .sort(([left], [right]) => left.localeCompare(right)),
          baseline: PUG_ORACLE_BASELINE.optionsSha256,
        }),
      ),
      pristine: pristine.evidence,
      formatted: after.evidence,
      sourceMapMoved:
        pristine.evidence.sourceMapSha256 == null || after.evidence.sourceMapSha256 == null
          ? null
          : pristine.evidence.sourceMapSha256 !== after.evidence.sourceMapSha256,
      templateOffsetsMoved:
        pristine.evidence.templateOffsets == null || after.evidence.templateOffsets == null
          ? null
          : JSON.stringify(pristine.evidence.templateOffsets) !==
            JSON.stringify(after.evidence.templateOffsets),
    },
  };
}

function compileSide(
  source: string,
  context: PugOracleContext,
): {
  compiled: { ast: TemplateNode; code: string; map?: unknown } | null;
  semanticAst: TemplateNode | null;
  diagnostics: CapturedDiagnostic[];
  evidence: SideEvidence;
  error: string | null;
} {
  const evidence: SideEvidence = {
    sourceSha256: sha256(source),
    pugBodySha256: null,
    relativePugSha256: null,
    preprocessedHtmlSha256: null,
    normalizedRenderSha256: null,
    renderCodeSha256: null,
    diagnosticsSha256: null,
    diagnosticLocationsSha256: null,
    sourceMapSha256: null,
    templateOffsets: null,
    dependencies: [],
    error: null,
  };
  const diagnostics: CapturedDiagnostic[] = [];
  let compiled: { ast: TemplateNode; code: string; map?: unknown } | null = null;
  let semanticAst: TemplateNode | null = null;

  try {
    const template = parseSfc(source, {
      filename: context.filename,
      sourceMap: true,
    }).descriptor.template;
    if (template == null || template.lang?.toLowerCase() !== "pug") {
      throw new Error('expected one <template lang="pug"> block');
    }
    evidence.pugBodySha256 = sha256(template.content);
    evidence.relativePugSha256 = sha256(canonicalRelativePug(template.content));
    evidence.templateOffsets = {
      start: template.loc.start.offset,
      end: template.loc.end.offset,
    };
    const filters = context.filters ?? {};
    const compileOptions = {
      filename: context.filename,
      basedir: context.basedir,
      doctype: pugOptions.doctype,
      compileDebug: pugOptions.compileDebug,
      pretty: pugOptions.pretty,
      globals: pugOptions.globals,
      filters,
    };
    assertStaticPug(template.content, context.filename, filters);
    const tracked = pug.compileClientWithDependenciesTracked(template.content, compileOptions);
    for (const dependency of tracked.dependencies) {
      assertStaticPug(fs.readFileSync(dependency, "utf8"), dependency, filters);
    }
    const render = pug.compile(template.content, compileOptions);
    const html = render(Object.create(null) as Record<string, unknown>);
    evidence.preprocessedHtmlSha256 = sha256(html);
    evidence.dependencies = dependencyEvidence(render.dependencies, context.basedir);
    semanticAst = parseVueTemplate(html, vueParserOptions);
    compiled = compileVueTemplate(html, {
      ...vueCompilerOptions,
      filename: context.displayFilename,
      onError: (diagnostic: Diagnostic) =>
        diagnostics.push({ severity: "error", value: diagnostic }),
      onWarn: (diagnostic: Diagnostic) =>
        diagnostics.push({ severity: "warning", value: diagnostic }),
    });
    evidence.normalizedRenderSha256 = sha256(templateAstSemanticSignature(semanticAst));
    evidence.renderCodeSha256 = sha256(compiled.code);
    evidence.sourceMapSha256 = sha256(JSON.stringify(compiled.map ?? null));
  } catch (error) {
    evidence.error = stableError(error, context);
  }

  evidence.diagnosticsSha256 = sha256(JSON.stringify(diagnosticSignatures(diagnostics)));
  evidence.diagnosticLocationsSha256 = sha256(
    JSON.stringify(diagnostics.map(diagnosticLocation).sort()),
  );

  return { compiled, semanticAst, diagnostics, evidence, error: evidence.error };
}

function canonicalRelativePug(source: string): string {
  const lines = source.replaceAll("\r\n", "\n").split("\n");
  while (lines.length > 0 && isBlankLine(lines[0])) lines.shift();
  while (lines.length > 0 && isBlankLine(lines.at(-1)!)) lines.pop();
  const nonBlank = lines.filter((line) => !isBlankLine(line));
  if (nonBlank.length === 0) return "";
  let common = leadingWhitespace(nonBlank[0]);
  for (const line of nonBlank.slice(1)) {
    const limit = Math.min(common.length, leadingWhitespace(line).length);
    let index = 0;
    while (index < limit && line[index] === common[index]) index += 1;
    common = common.slice(0, index);
  }
  return lines.map((line) => (isBlankLine(line) ? "" : line.slice(common.length))).join("\n");
}

function leadingWhitespace(line: string): string {
  return line.match(/^[\t ]*/)?.[0] ?? "";
}

function isBlankLine(line: string): boolean {
  return /^[\t ]*$/.test(line);
}

function assertStaticPug(
  source: string,
  filename: string,
  filters: Record<string, PugFilter>,
): void {
  for (const token of lexPug(source, { filename })) {
    if (token.type === "filter") {
      if (typeof token.val !== "string" || !Object.hasOwn(filters, token.val)) {
        throw new Error(`Pug filter ${String(token.val)} has no explicit deterministic oracle`);
      }
      continue;
    }
    if (token.type === "attribute") {
      if (token.val === true || (typeof token.val === "string" && isQuotedLiteral(token.val))) {
        continue;
      }
      throw new Error(`executable Pug attribute ${token.name ?? "unknown"} is not oracle-safe`);
    }
    if (
      (token.type === "code" || token.type === "interpolated-code") &&
      typeof token.val === "string" &&
      isQuotedLiteral(token.val)
    ) {
      continue;
    }
    if (executablePugTokens.has(token.type)) {
      throw new Error(`executable Pug token ${token.type} is not oracle-safe`);
    }
  }
}

const executablePugTokens = new Set([
  "&attributes",
  "call",
  "case",
  "code",
  "conditional",
  "each",
  "eachOf",
  "else",
  "else-if",
  "if",
  "interpolated-code",
  "interpolation",
  "mixin",
  "mixin-block",
  "unless",
  "when",
  "while",
]);

function isQuotedLiteral(value: string): boolean {
  if (value.length < 2) return false;
  const quote = value[0];
  if ((quote !== '"' && quote !== "'") || value.at(-1) !== quote) return false;
  let escaped = false;
  for (let index = 1; index < value.length - 1; index += 1) {
    const character = value[index];
    if (escaped) {
      escaped = false;
    } else if (character === "\\") {
      escaped = true;
    } else if (character === quote) {
      return false;
    }
  }
  return !escaped;
}

function dependencyEvidence(
  dependencies: string[],
  basedir: string,
): Array<{ path: string; sha256: string }> {
  return [...new Set(dependencies)]
    .sort((left, right) => left.localeCompare(right))
    .map((dependency) => ({
      path: path.relative(basedir, dependency).split(path.sep).join("/"),
      sha256: sha256(fs.readFileSync(dependency)),
    }));
}

function diagnosticSignatures(diagnostics: CapturedDiagnostic[]): string[] {
  return diagnostics.map(diagnosticSignature).sort((left, right) => left.localeCompare(right));
}

function diagnosticSignature(diagnostic: CapturedDiagnostic): string {
  return JSON.stringify([
    diagnostic.severity,
    diagnostic.value.code ?? diagnostic.value.name ?? "unknown",
    diagnostic.value.message ?? JSON.stringify(diagnostic.value),
  ]);
}

function diagnosticLocation(diagnostic: CapturedDiagnostic): string {
  const start = diagnostic.value.loc?.start;
  return JSON.stringify([
    diagnostic.severity,
    start?.line ?? null,
    start?.column ?? null,
    start?.offset ?? null,
  ]);
}

function diagnosticSummary(diagnostics: CapturedDiagnostic[]): string {
  return diagnosticSignatures(diagnostics).join(", ");
}

function stableError(error: unknown, context: PugOracleContext): string {
  const message = error instanceof Error ? error.message : String(error);
  return message
    .replaceAll(context.filename, context.displayFilename)
    .replaceAll(context.basedir, "<fixture-root>")
    .split("\n")
    .slice(0, 3)
    .join(" ");
}

function sha256(content: string | Buffer): string {
  return createHash("sha256").update(content).digest("hex");
}
