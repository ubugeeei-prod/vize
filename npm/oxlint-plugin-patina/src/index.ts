import fs from "node:fs";
import { createRequire } from "node:module";

import {
  definePlugin,
  defineRule,
  type Context,
  type Diagnostic,
  type ESTree,
} from "@oxlint/plugins";

const require = createRequire(import.meta.url);

interface PatinaPosition {
  line: number;
  column: number;
  offset: number;
}

interface PatinaLocation {
  start: PatinaPosition;
  end: PatinaPosition;
}

interface PatinaDiagnostic {
  rule: string;
  severity: "error" | "warning";
  message: string;
  location: PatinaLocation;
  help: string | null;
}

interface PatinaLintResult {
  filename: string;
  errorCount: number;
  warningCount: number;
  diagnostics: PatinaDiagnostic[];
}

interface PatinaRuleMeta {
  name: string;
  description: string;
  category: string;
  fixable: boolean;
  defaultSeverity: "error" | "warning";
}

interface PatinaBinding {
  lintPatinaSfc(
    source: string,
    options?: {
      filename?: string;
      locale?: string;
      enabled_rules?: string[];
    },
  ): PatinaLintResult;
  getPatinaRules(): PatinaRuleMeta[];
}

interface LineColumn {
  line: number;
  column: number;
}

interface ScriptBlock {
  content: string;
  contentStart: LineColumn;
  contentEnd: LineColumn;
}

interface SingleScriptMap {
  block: ScriptBlock;
}

interface FileState {
  source: string;
  sourceLines: readonly string[];
  extractedScript: string;
  scriptMap: SingleScriptMap | null | undefined;
  allDiagnosticsByRule: Map<string, PatinaDiagnostic[]> | null;
  partialDiagnosticsByRule: Map<string, readonly PatinaDiagnostic[]>;
  requestedRules: Set<string>;
}

interface PatinaSettings {
  locale?: string;
}

let bindingCache: PatinaBinding | null = null;
const fileStateCache = new Map<string, FileState>();
const helpTextCache = new Map<string, string>();
const EMPTY_DIAGNOSTICS: readonly PatinaDiagnostic[] = [];

function loadBinding(): PatinaBinding {
  if (bindingCache) {
    return bindingCache;
  }

  const binding = require("@vizejs/native") as Partial<PatinaBinding>;
  if (typeof binding.lintPatinaSfc !== "function" || typeof binding.getPatinaRules !== "function") {
    throw new Error(
      "@vizejs/native does not expose the Patina Oxlint bridge. Rebuild or upgrade @vizejs/native.",
    );
  }

  bindingCache = binding as PatinaBinding;
  return bindingCache;
}

function isVueLikeFile(filename: string): boolean {
  return filename.endsWith(".vue");
}

function getPatinaSettings(context: Context): PatinaSettings {
  const settings = context.settings as Record<string, unknown>;
  const patina = settings.patina;
  if (typeof patina !== "object" || patina === null || Array.isArray(patina)) {
    return {};
  }

  const locale = (patina as Record<string, unknown>).locale;
  return typeof locale === "string" ? { locale } : {};
}

function getCacheKey(filename: string, settings: PatinaSettings): string {
  return `${filename}::${settings.locale ?? ""}`;
}

function getFileState(context: Context): FileState {
  const settings = getPatinaSettings(context);
  const cacheKey = getCacheKey(context.physicalFilename, settings);
  const cached = fileStateCache.get(cacheKey);
  if (cached) {
    return cached;
  }

  const state: FileState = {
    source: fs.readFileSync(context.physicalFilename, "utf8"),
    sourceLines: [],
    extractedScript: context.sourceCode.text,
    scriptMap: undefined,
    allDiagnosticsByRule: null,
    partialDiagnosticsByRule: new Map(),
    requestedRules: new Set(),
  };
  state.sourceLines = state.source.split(/\r\n?|\n/gu);
  fileStateCache.set(cacheKey, state);
  return state;
}

function indexDiagnosticsByRule(
  diagnostics: readonly PatinaDiagnostic[],
): Map<string, PatinaDiagnostic[]> {
  const grouped = new Map<string, PatinaDiagnostic[]>();

  for (const diagnostic of diagnostics) {
    const existing = grouped.get(diagnostic.rule);
    if (existing) {
      existing.push(diagnostic);
      continue;
    }

    grouped.set(diagnostic.rule, [diagnostic]);
  }

  return grouped;
}

function lintPatina(
  source: string,
  filename: string,
  settings: PatinaSettings,
  enabledRules?: readonly string[],
): PatinaLintResult {
  return loadBinding().lintPatinaSfc(source, {
    filename,
    locale: settings.locale,
    enabled_rules: enabledRules ? [...enabledRules] : undefined,
  });
}

function getDiagnosticsForRule(
  context: Context,
  state: FileState,
  ruleName: string,
): readonly PatinaDiagnostic[] {
  if (state.allDiagnosticsByRule) {
    return state.allDiagnosticsByRule.get(ruleName) ?? EMPTY_DIAGNOSTICS;
  }

  const cached = state.partialDiagnosticsByRule.get(ruleName);
  if (cached) {
    return cached;
  }

  const settings = getPatinaSettings(context);

  if (state.requestedRules.size === 0) {
    state.requestedRules.add(ruleName);
    const diagnostics = lintPatina(state.source, context.physicalFilename, settings, [
      ruleName,
    ]).diagnostics;
    const indexedDiagnostics = indexDiagnosticsByRule(diagnostics);
    const ruleDiagnostics = indexedDiagnostics.get(ruleName) ?? EMPTY_DIAGNOSTICS;
    state.partialDiagnosticsByRule.set(ruleName, ruleDiagnostics);
    return ruleDiagnostics;
  }

  state.requestedRules.add(ruleName);
  state.allDiagnosticsByRule = indexDiagnosticsByRule(
    lintPatina(state.source, context.physicalFilename, settings).diagnostics,
  );
  state.partialDiagnosticsByRule.clear();
  return state.allDiagnosticsByRule.get(ruleName) ?? EMPTY_DIAGNOSTICS;
}

function getScriptMap(state: FileState): SingleScriptMap | null {
  if (state.scriptMap !== undefined) {
    return state.scriptMap;
  }

  state.scriptMap = createSingleScriptMap(state.source, state.extractedScript);
  return state.scriptMap;
}

function createSingleScriptMap(source: string, extractedScript: string): SingleScriptMap | null {
  if (!extractedScript) {
    return null;
  }

  const blocks = extractScriptBlocks(source);
  if (blocks.length !== 1) {
    return null;
  }

  const [block] = blocks;
  return block.content === extractedScript ? { block } : null;
}

function createLineStartOffsets(source: string): number[] {
  const lineStarts = [0];

  for (let index = 0; index < source.length; index += 1) {
    if (source.charCodeAt(index) === 10) {
      lineStarts.push(index + 1);
    }
  }

  return lineStarts;
}

function offsetToLineColumn(lineStarts: readonly number[], offset: number): LineColumn {
  let low = 0;
  let high = lineStarts.length - 1;

  while (low <= high) {
    const middle = (low + high) >> 1;
    if (lineStarts[middle] <= offset) {
      low = middle + 1;
      continue;
    }

    high = middle - 1;
  }

  const lineIndex = Math.max(0, low - 1);
  return {
    line: lineIndex + 1,
    column: offset - lineStarts[lineIndex] + 1,
  };
}

function extractScriptBlocks(source: string): ScriptBlock[] {
  const blocks: ScriptBlock[] = [];
  const lineStarts = createLineStartOffsets(source);
  const scriptTag = /<script\b[^>]*>/giu;
  let match: RegExpExecArray | null;

  while ((match = scriptTag.exec(source)) !== null) {
    const openTagEnd = match.index + match[0].length;
    const closeTagStart = source.indexOf("</script>", openTagEnd);
    if (closeTagStart === -1) {
      break;
    }

    blocks.push({
      content: source.slice(openTagEnd, closeTagStart),
      contentStart: offsetToLineColumn(lineStarts, openTagEnd),
      contentEnd: offsetToLineColumn(lineStarts, closeTagStart),
    });
    scriptTag.lastIndex = closeTagStart + "</script>".length;
  }

  return blocks;
}

function compareLineColumn(left: LineColumn, right: LineColumn): number {
  if (left.line !== right.line) {
    return left.line - right.line;
  }

  return left.column - right.column;
}

function toScriptPosition(position: LineColumn, contentStart: LineColumn): LineColumn {
  if (position.line === contentStart.line) {
    return {
      line: 1,
      column: position.column - contentStart.column + 1,
    };
  }

  return {
    line: position.line - contentStart.line + 1,
    column: position.column,
  };
}

function mapToScriptLoc(
  diagnostic: PatinaDiagnostic,
  scriptMap: SingleScriptMap | null,
): Diagnostic["loc"] | null {
  if (!scriptMap) {
    return null;
  }

  const { block } = scriptMap;
  if (
    compareLineColumn(diagnostic.location.start, block.contentStart) < 0 ||
    compareLineColumn(diagnostic.location.end, block.contentEnd) > 0
  ) {
    return null;
  }

  return {
    start: toScriptPosition(diagnostic.location.start, block.contentStart),
    end: toScriptPosition(diagnostic.location.end, block.contentStart),
  };
}

function formatPatinaMessage(
  diagnostic: PatinaDiagnostic,
  useMappedLocation: boolean,
  sourceSnippet: string | null,
): string {
  const locationPrefix = useMappedLocation
    ? ""
    : `Actual Vue location: line ${diagnostic.location.start.line}, column ${diagnostic.location.start.column}\n`;
  const snippetSection = sourceSnippet ? `Source:\n${indentBlock(sourceSnippet, "  ")}\n` : "";
  const helpSuffix = diagnostic.help
    ? `\nHelp:\n${indentBlock(formatHelpText(diagnostic.help), "  ")}`
    : "";
  return `${locationPrefix}${snippetSection}${diagnostic.message}${helpSuffix}`;
}

function formatHelpText(help: string): string {
  const cached = helpTextCache.get(help);
  if (cached) {
    return cached;
  }

  const plainText = help
    .replace(/\r\n?/gu, "\n")
    .replace(/^\s*```[^\n]*$/gmu, "")
    .replace(/\*\*(.*?)\*\*/gu, "$1")
    .replace(/__(.*?)__/gu, "$1")
    .replace(/`([^`\n]+)`/gu, "$1")
    .replace(/^\s*[-*]\s+/gmu, "- ")
    .replace(/\n{3,}/gu, "\n\n")
    .trim();

  helpTextCache.set(help, plainText);
  return plainText;
}

function indentBlock(text: string, indent: string): string {
  return text
    .split("\n")
    .map((line) => `${indent}${line}`)
    .join("\n");
}

function createSourceSnippet(
  sourceLines: readonly string[],
  diagnostic: PatinaDiagnostic,
): string | null {
  const line = sourceLines[diagnostic.location.start.line - 1];
  if (line === undefined) {
    return null;
  }

  const startColumn = Math.max(1, diagnostic.location.start.column);
  const endColumn =
    diagnostic.location.start.line === diagnostic.location.end.line
      ? Math.max(startColumn + 1, diagnostic.location.end.column)
      : startColumn + 1;
  const caretWidth = Math.max(1, endColumn - startColumn);
  const lineNumber = String(diagnostic.location.start.line);
  const gutter = `${lineNumber} | `;
  const caretIndent = " ".repeat(gutter.length + startColumn - 1);

  return `${gutter}${line}\n${caretIndent}${"^".repeat(caretWidth)}`;
}

function createOxlintDiagnostic(
  _node: ESTree.Program,
  diagnostic: PatinaDiagnostic,
  state: FileState,
): Diagnostic {
  const scriptMap = getScriptMap(state);
  const loc = mapToScriptLoc(diagnostic, scriptMap);
  const message = formatPatinaMessage(
    diagnostic,
    loc !== null,
    loc ? null : createSourceSnippet(state.sourceLines, diagnostic),
  );

  return {
    loc: loc ?? {
      start: { line: 1, column: 1 },
      end: { line: 1, column: 1 },
    },
    message,
  };
}

function createPatinaRule(ruleMeta: PatinaRuleMeta) {
  return defineRule({
    meta: {
      type: ruleMeta.defaultSeverity === "error" ? "problem" : "suggestion",
      docs: {
        description: ruleMeta.description,
      },
    },
    createOnce(context) {
      return {
        Program(node) {
          if (!isVueLikeFile(context.filename)) {
            return;
          }

          const state = getFileState(context);
          const diagnostics = getDiagnosticsForRule(context, state, ruleMeta.name);
          if (diagnostics.length === 0) {
            return;
          }

          const scriptMap = getScriptMap(state);
          for (const diagnostic of diagnostics) {
            void scriptMap;
            context.report(createOxlintDiagnostic(node, diagnostic, state));
          }
        },
      };
    },
  });
}

const patinaRules = Object.fromEntries(
  loadBinding()
    .getPatinaRules()
    .map((ruleMeta) => [ruleMeta.name, createPatinaRule(ruleMeta)]),
);

export default definePlugin({
  meta: {
    name: "patina",
  },
  rules: patinaRules,
});
