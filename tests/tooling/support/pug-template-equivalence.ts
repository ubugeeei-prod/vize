// Semantic oracle for `<template lang="pug">` blocks. Pug is preprocessed with
// the pinned Pug build and the resulting HTML is compiled with the pinned Vue
// compiler on both sides, so a formatting change may rebase outer indentation
// but must not move a single byte of authored Pug or change the compiled
// program. The oracle is fixed-Vue-3 and maps against preprocessed HTML; it
// makes no authored-Pug source-map claim. See ./pug/oracle-runtime.ts for the
// pinned toolchain and the fail-closed static-Pug guards.
import fs from "node:fs";

import {
  PUG_ORACLE_BASELINE,
  assertStaticPug,
  canonicalRelativePug,
  codePointCompare,
  compileVueTemplate,
  dependencyEvidence,
  diagnosticLocation,
  diagnosticSignatures,
  diagnosticSummary,
  parseSfc,
  parseVueTemplate,
  pug,
  pugOptions,
  sha256,
  stableError,
  vueCompilerOptions,
  vueParserOptions,
} from "./pug/oracle-runtime.ts";
import type { CapturedDiagnostic, Diagnostic, PugOracleContext } from "./pug/oracle-runtime.ts";
import {
  compareSfcBlockStructure,
  compareTemplateAstEquivalence,
  templateAstSemanticSignature,
} from "./sfc-equivalence.ts";
import type { TemplateNode } from "./sfc-equivalence.ts";
import { findSfcOpeningTagEnd } from "./sfc-opening-tag.ts";

export { PUG_ORACLE_BASELINE } from "./pug/oracle-runtime.ts";
export type { PugOracleContext } from "./pug/oracle-runtime.ts";

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

export function isPugSfc(source: string, _filename: string): boolean {
  const openingTag = findTopLevelTemplateOpeningTag(source);
  if (openingTag == null) return false;
  const lang = /(?:^|\s)lang\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+))/iu.exec(openingTag);
  return (lang?.[1] ?? lang?.[2] ?? lang?.[3] ?? "").toLowerCase() === "pug";
}

/** Read only top-level SFC opening tags, without invoking a dialect parser. */
function findTopLevelTemplateOpeningTag(source: string): string | null {
  let offset = 0;
  while (offset < source.length) {
    const start = source.indexOf("<", offset);
    if (start < 0) return null;
    if (source.startsWith("<!--", start)) {
      const end = source.indexOf("-->", start + 4);
      offset = end < 0 ? source.length : end + 3;
      continue;
    }
    const nameMatch = /^<([A-Za-z][\w-]*)\b/u.exec(source.slice(start));
    if (nameMatch == null) {
      offset = start + 1;
      continue;
    }
    const end = findSfcOpeningTagEnd(source, start + nameMatch[0].length);
    if (end < 0) return null;
    const name = nameMatch[1].toLowerCase();
    const openingTag = source.slice(start, end + 1);
    if (name === "template") return openingTag;
    if (!/\/\s*>$/u.test(openingTag)) {
      const closePattern = new RegExp(`</${name}\\s*>`, "iu");
      const remainder = source.slice(end + 1);
      const close = closePattern.exec(remainder);
      offset = close == null ? source.length : end + 1 + close.index + close[0].length;
    } else {
      offset = end + 1;
    }
  }
  return null;
}

export function comparePugTemplateEquivalence(
  original: string,
  formatted: string,
  context: PugOracleContext,
): PugOracleComparison {
  const differences = compareSfcBlockStructure(original, formatted, context.filename);
  const pristine = compileSide(original, context);
  const after = compileSide(formatted, context);
  const pristineErrors = diagnosticsOfSeverity(pristine.diagnostics, "error");
  const afterErrors = diagnosticsOfSeverity(after.diagnostics, "error");
  const baselineUsable =
    pristine.compiled != null && pristine.error == null && pristineErrors.length === 0;

  if (pristine.error != null) differences.push(`pristine Pug baseline failed: ${pristine.error}`);
  if (pristineErrors.length > 0) {
    differences.push(`pristine Vue baseline failed: ${diagnosticSummary(pristineErrors)}`);
  }
  if (after.error != null) differences.push(`formatted Pug baseline failed: ${after.error}`);
  if (afterErrors.length > 0) {
    differences.push(`formatted Vue baseline failed: ${diagnosticSummary(afterErrors)}`);
  }

  if (pristine.compiled != null && after.compiled != null) {
    if (pristine.evidence.relativePugSha256 !== after.evidence.relativePugSha256) {
      differences.push("relative authored Pug bytes changed");
    }
    const beforeWarnings = diagnosticSignatures(
      diagnosticsOfSeverity(pristine.diagnostics, "warning"),
    );
    const afterWarnings = diagnosticSignatures(diagnosticsOfSeverity(after.diagnostics, "warning"));
    if (JSON.stringify(beforeWarnings) !== JSON.stringify(afterWarnings)) {
      differences.push(
        `Vue compiler warnings changed: ${JSON.stringify(beforeWarnings)} -> ` +
          JSON.stringify(afterWarnings),
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
            .sort(([left], [right]) => codePointCompare(left, right)),
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

function diagnosticsOfSeverity(
  diagnostics: CapturedDiagnostic[],
  severity: CapturedDiagnostic["severity"],
): CapturedDiagnostic[] {
  return diagnostics.filter((diagnostic) => diagnostic.severity === severity);
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
    JSON.stringify(diagnostics.map(diagnosticLocation).sort(codePointCompare)),
  );

  return { compiled, semanticAst, diagnostics, evidence, error: evidence.error };
}
