// Pinned runtime for the Pug semantic oracle: the exact Pug and Vue compiler
// builds it speaks for, the compile options every side is hashed against, and
// the fail-closed guards that keep the oracle deterministic (static Pug only,
// explicit filters, content-addressed includes).
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

import type { TemplateNode } from "../sfc-equivalence.ts";

const require = createRequire(import.meta.url);
export const pug = require("pug") as {
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
export const {
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
export const { parse: parseSfc } = require("@vue/compiler-sfc") as {
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

export const pugOptions = {
  compileDebug: false,
  doctype: "html",
  pretty: false,
  globals: [] as string[],
  filters: [] as string[],
};
export const vueCompilerOptions = {
  mode: "module",
  prefixIdentifiers: true,
  hoistStatic: true,
  cacheHandlers: false,
  whitespace: "preserve",
  comments: true,
  sourceMap: true,
};

export function sha256(content: string | Buffer): string {
  return createHash("sha256").update(content).digest("hex");
}

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

export type PugFilter = (source: string, options: Record<string, unknown>) => string;
export type PugOracleContext = {
  filename: string;
  displayFilename: string;
  basedir: string;
  filters?: Record<string, PugFilter>;
};

export type Diagnostic = {
  code?: number | string;
  name?: string;
  message?: string;
  loc?: { start?: { line?: number; column?: number; offset?: number } };
};
export type CapturedDiagnostic = { severity: "error" | "warning"; value: Diagnostic };

/** Deterministic Unicode code-point ordering, independent of host ICU data. */
export function codePointCompare(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

/** Reject anything whose rendered HTML could depend on ambient state. */
export function assertStaticPug(
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
      // A quoted Pug string remains static even when its name is a Vue
      // directive; compiler-dom interprets the rendered attribute later.
      // Unquoted Pug expressions stay rejected because Pug would execute them.
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

/** Strip the common outer indentation so an opaque rebase is a no-op. */
export function canonicalRelativePug(source: string): string {
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

export function dependencyEvidence(
  dependencies: string[],
  basedir: string,
): Array<{ path: string; sha256: string }> {
  return [...new Set(dependencies)].sort(codePointCompare).map((dependency) => ({
    path: path.relative(basedir, dependency).split(path.sep).join("/"),
    sha256: sha256(fs.readFileSync(dependency)),
  }));
}

export function diagnosticSignatures(diagnostics: CapturedDiagnostic[]): string[] {
  return diagnostics.map(diagnosticSignature).sort(codePointCompare);
}

function diagnosticSignature(diagnostic: CapturedDiagnostic): string {
  return JSON.stringify([
    diagnostic.severity,
    diagnostic.value.code ?? diagnostic.value.name ?? "unknown",
    diagnostic.value.message ?? JSON.stringify(diagnostic.value),
  ]);
}

export function diagnosticLocation(diagnostic: CapturedDiagnostic): string {
  const start = diagnostic.value.loc?.start;
  return JSON.stringify([
    diagnostic.severity,
    start?.line ?? null,
    start?.column ?? null,
    start?.offset ?? null,
  ]);
}

export function diagnosticSummary(diagnostics: CapturedDiagnostic[]): string {
  return diagnosticSignatures(diagnostics).join(", ");
}

export function stableError(error: unknown, context: PugOracleContext): string {
  const message = error instanceof Error ? error.message : String(error);
  return message
    .replaceAll(context.filename, context.displayFilename)
    .replaceAll(context.basedir, "<fixture-root>")
    .split("\n")
    .slice(0, 3)
    .join(" ");
}
