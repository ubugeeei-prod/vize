/**
 * Native binding loader for @vizejs/native.
 *
 * Provides lazy-loading of the native Rust-based parser and a JS fallback
 * for SFC analysis when the native `analyzeSfc` function is unavailable.
 */

import { createRequire } from "node:module";
import ts from "typescript";

import { extractWithDefaults } from "./with-defaults.js";

/** JSON defaults proven without evaluating component code. */
export interface SfcProp {
  name: string;
  type: string;
  required: boolean;
  default_value?: unknown;
  default_source?: string;
  unsupported_default?: boolean;
}

export interface SfcAnalysis {
  props: SfcProp[];
  emits: string[];
}

// Native binding types
export interface NativeBinding {
  parseArtStatusWarnings?: (source: string, options?: { filename?: string }) => string[];
  parseArt: (
    source: string,
    options?: { filename?: string },
  ) => {
    filename: string;
    metadata: {
      title: string;
      description?: string;
      component?: string;
      category?: string;
      tags: string[];
      status: string;
      order?: number;
      actionEvents?: string[];
    };
    variants: Array<{
      name: string;
      template: string;
      isDefault: boolean;
      skipVrt: boolean;
    }>;
    hasScriptSetup: boolean;
    hasScript: boolean;
    styleCount: number;
  };
  /** Compile a Vue SFC the same way the enclosing `.art.vue` file is compiled. */
  compileSfc: (
    source: string,
    options?: { filename?: string },
  ) => {
    code?: string;
    css?: string;
    errors?: Array<string | { message?: string }>;
  };
  scopeViteCssForPipeline?: (css: string, scopeId: string) => string;
  artToCsf: (
    source: string,
    options?: { filename?: string },
  ) => {
    code: string;
    filename: string;
  };
  generateArtPalette?: (
    source: string,
    artOptions?: { filename?: string },
    paletteOptions?: { infer_options?: boolean; group_by_type?: boolean },
  ) => {
    title: string;
    controls: Array<{
      name: string;
      control: string;
      default_value?: unknown;
      description?: string;
      required: boolean;
      options: Array<{ label: string; value: unknown }>;
      range?: { min: number; max: number; step?: number };
      group?: string;
    }>;
    groups: string[];
    json: string;
    typescript: string;
  };
  generateArtDoc?: (
    source: string,
    artOptions?: { filename?: string },
    docOptions?: {
      include_source?: boolean;
      include_templates?: boolean;
      include_metadata?: boolean;
    },
  ) => {
    markdown: string;
    filename: string;
    title: string;
    category?: string;
    variant_count: number;
  };
  parseDesignTokensFromPath?: (path: string) => unknown;
  buildDesignTokenMap?: (categories: unknown) => Record<string, unknown>;
  resolveDesignTokenReferences?: (categories: unknown) => unknown;
  flattenDesignTokenCategories?: (categories: unknown) => unknown[];
  generateDesignTokensMarkdown?: (categories: unknown, generatedAt?: string) => string;
  validateDesignTokenReference?: (
    tokenMap: Record<string, unknown>,
    reference: string,
    selfPath?: string,
  ) => { valid: boolean; error?: string };
  findDependentDesignTokens?: (tokenMap: Record<string, unknown>, targetPath: string) => string[];
  analyzeSfc?: (
    source: string,
    options?: { filename?: string; profile?: boolean; cache?: boolean },
  ) => SfcAnalysis;
}

// Lazy-load native binding
let native: NativeBinding | null = null;

export function loadNative(): NativeBinding {
  if (native) return native;

  const require = createRequire(import.meta.url);
  try {
    native = require("@vizejs/native") as NativeBinding;
    return native;
  } catch (e) {
    throw new Error(
      `Failed to load @vizejs/native. Make sure it's installed and built:\n${String(e)}`,
    );
  }
}

/**
 * JS-based fallback for SFC analysis when native `analyzeSfc` is not available.
 * Reads direct type-literal props and preserves the existing emits/defaults contract.
 */
export function analyzeSfcFallback(source: string, _options?: { filename?: string }): SfcAnalysis {
  try {
    const props: SfcProp[] = [];
    const emits: string[] = [];

    // Extract the <script setup> block
    const scriptSetupMatch = source.match(/<script\s+[^>]*setup[^>]*>([\s\S]*?)<\/script>/);
    if (!scriptSetupMatch) {
      // Try regular <script> block
      const scriptMatch = source.match(/<script[^>]*>([\s\S]*?)<\/script>/);
      if (!scriptMatch) return { props: [], emits: [] };
    }
    const scriptContent = scriptSetupMatch?.[1] || "";

    const withDefaults = extractWithDefaults(scriptContent);
    for (const { name, type, optional } of directTypeLiteralProps(scriptContent)) {
      const defaultValue = withDefaults.get(name);
      props.push({
        name,
        type,
        required: !optional,
        ...(defaultValue !== undefined ? fallbackDefault(defaultValue) : {}),
      });
    }

    // Extract defineEmits
    const emitsMatch = scriptContent.match(/defineEmits\s*<\s*\{([\s\S]*?)\}>/);
    if (emitsMatch) {
      const emitsBody = emitsMatch[1];
      const emitRegex = /(\w+)\s*:/g;
      let match;
      while ((match = emitRegex.exec(emitsBody)) !== null) {
        emits.push(match[1]);
      }
    }

    return { props, emits };
  } catch {
    return { props: [], emits: [] };
  }
}

function directTypeLiteralProps(
  scriptContent: string,
): Array<{ name: string; type: string; optional: boolean }> {
  const source = ts.createSourceFile("props.ts", scriptContent, ts.ScriptTarget.Latest, true);
  const findLiteral = (node: ts.Node): ts.TypeLiteralNode | undefined => {
    if (
      ts.isCallExpression(node) &&
      ts.isIdentifier(node.expression) &&
      node.expression.text === "defineProps"
    ) {
      const argument = node.typeArguments?.[0];
      if (argument && ts.isTypeLiteralNode(argument)) return argument;
    }
    return ts.forEachChild(node, findLiteral);
  };
  const literal = findLiteral(source);
  return (
    literal?.members.flatMap((member) => {
      if (!ts.isPropertySignature(member) || !member.type) return [];
      let name = member.name;
      if (ts.isComputedPropertyName(name)) {
        if (!ts.isStringLiteral(name.expression)) return [];
        name = name.expression;
      }
      if (!ts.isIdentifier(name) && !ts.isStringLiteral(name) && !ts.isNumericLiteral(name))
        return [];
      return [
        { name: name.text, type: member.type.getText(source), optional: !!member.questionToken },
      ];
    }) ?? []
  );
}

/** Preserve the JSON contract when an older native binary requires the typed fallback. */
function fallbackDefault(
  source: string,
): Pick<SfcProp, "default_value" | "default_source" | "unsupported_default"> {
  const file = ts.createSourceFile(
    "default.ts",
    `const value = (${source});`,
    ts.ScriptTarget.Latest,
    true,
  );
  const statement = file.statements[0];
  let value =
    statement && ts.isVariableStatement(statement)
      ? statement.declarationList.declarations[0]?.initializer
      : undefined;
  while (
    value &&
    (ts.isParenthesizedExpression(value) ||
      ts.isAsExpression(value) ||
      ts.isSatisfiesExpression(value))
  )
    value = value.expression;
  if (value && (ts.isStringLiteral(value) || ts.isNoSubstitutionTemplateLiteral(value)))
    return { default_value: value.text };
  if (value && ts.isNumericLiteral(value) && Number.isFinite(Number(value.text)))
    return { default_value: Number(value.text) };
  if (value?.kind === ts.SyntaxKind.TrueKeyword) return { default_value: true };
  if (value?.kind === ts.SyntaxKind.FalseKeyword) return { default_value: false };
  if (value?.kind === ts.SyntaxKind.NullKeyword) return { default_value: null };
  if (value && ts.isPrefixUnaryExpression(value) && ts.isNumericLiteral(value.operand)) {
    const number = Number(value.operand.text);
    if (value.operator === ts.SyntaxKind.MinusToken) return { default_value: -number };
    if (value.operator === ts.SyntaxKind.PlusToken) return { default_value: number };
  }
  return { default_source: source, unsupported_default: true };
}
