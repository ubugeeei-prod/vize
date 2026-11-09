// Per-file rule extraction: META identity, impl/hook shape, symbol-aware
// croquis usage (P0-7's use-declaration resolution), and the lexical signals
// the portability classification is derived from.

import { readFileSync } from "node:fs";
import path from "node:path";

import { byKey } from "./ordering.ts";
import { RULES_DIR } from "./rule-parity-paths.ts";
import type { DispatchModel } from "./rule-parity-dispatch.ts";
import {
  collectStringLiterals,
  findImplBlocks,
  matchBraceBlock,
  stripRustComments,
} from "./rule-parity-rust-text.ts";
import type { ImplBlock } from "./rule-parity-rust-text.ts";
import { expandUseTree, findUseDecls, maskKeepNewlines, stripRust } from "./rust-source.ts";

const CONTAINER_TOKEN_RE =
  /\b(SfcDescriptor|SfcBlock|SfcTemplateBlock|SfcScriptBlock|SfcStyleBlock|SfcParseOptions|parse_sfc|sfc_descriptor|sfc_template_descriptor|vize_atelier_sfc)\b/g;
const DIALECT_NODE_RE =
  /\b(DirectiveNode|DirectiveArgumentNode|ForNode|IfNode|IfBranchNode|InterpolationNode|MarkupDirective)\b|PropNode::Directive/g;
const DIALECT_SEMANTIC_RE =
  /\b(defineProps|defineEmits|defineModel|defineExpose|defineOptions|defineSlots|withDefaults|COMPILER_MACRO_NAMES|is_builtin_component|VueDialect|onBeforeMount|onMounted|onBeforeUpdate|onUpdated|onBeforeUnmount|onUnmounted|onActivated|onDeactivated|onServerPrefetch)\b/g;
const DIALECT_HOOKS = new Set(["check_directive", "check_for", "check_if", "check_interpolation"]);
const DIALECT_MARKUP_HOOKS = new Set(["enter_directive"]);
export const DIALECT_NAME_PREFIXES = ["ecosystem", "nuxt", "petite-vue", "vapor"];

export type HookModel = Pick<DispatchModel, "templateHooks" | "markupHooks">;

export interface RuleSignals {
  container: Set<string>;
  dialect: Set<string>;
}

export interface ParsedRuleFile {
  abs: string;
  rel: string;
  source: string;
  meta: { kind: string; name: string } | null;
  impls: ImplBlock[];
  ruleImplTypes: string[];
  hooks: Set<string>;
  markupHookSet: Set<string>;
  asMarkup: boolean;
  jsxLowered: boolean;
  scriptUsesAst: boolean;
  scriptUsesTemplateAst: boolean;
  croquisItems: Map<string, number>;
  ctxSites: number;
  signals: RuleSignals;
  kind?: "test" | "module" | "helper";
}

function lexicalScope(source: string, end: number): string {
  const opens: number[] = [];
  for (let at = 0; at < end; at += 1) {
    if ("{([".includes(source[at])) opens.push(at);
    else if ("})]".includes(source[at])) opens.pop();
  }
  return opens.join(",");
}

function expressionEnd(source: string, start: number, end: number, delimiter: string): number {
  let depth = 0;
  for (let at = start; at < end; at += 1) {
    const token = source[at];
    if (depth === 0 && token === delimiter) return at;
    if ("{([".includes(token)) depth += 1;
    else if ("})]".includes(token)) depth -= 1;
  }
  return end;
}

function literalRuleName(expression: string): string | null {
  if (!/^"(?:[^"\\]|\\(?:[nrt0\\"']|x[\da-fA-F]{2}|u\{[\da-fA-F_]+\}))*"$/su.test(expression)) {
    return null;
  }
  return expression
    .slice(1, -1)
    .replace(
      /\\(u\{([\da-fA-F_]+)\}|x([\da-fA-F]{2})|.)/gu,
      (_, escape: string, unicode: string | undefined, ascii: string | undefined) => {
        if (unicode !== undefined)
          return String.fromCodePoint(Number.parseInt(unicode.replaceAll("_", ""), 16));
        if (ascii !== undefined) return String.fromCodePoint(Number.parseInt(ascii, 16));
        return (
          ({ n: "\n", r: "\r", t: "\t", "0": "\0" } as Record<string, string>)[escape] ?? escape
        );
      },
    );
}

function metaRuleName(
  code: string,
  stripped: string,
  open: number,
  close: number,
  metaAt: number,
  abs: string,
): string {
  const scope = lexicalScope(stripped, open + 1);
  const fields = [...stripped.slice(open + 1, close).matchAll(/\bname\s*:/g)]
    .map((match) => ({ at: open + 1 + match.index, length: match[0].length }))
    .filter(({ at }) => lexicalScope(stripped, at) === scope);
  if (fields.length !== 1) throw new Error(`META without one name in ${abs}`);
  const start = fields[0].at + fields[0].length;
  const end = expressionEnd(stripped, start, close, ",");
  const expression = code.slice(start, end).trim();
  const literal = literalRuleName(expression);
  if (literal !== null && literal.length > 0) return literal;
  if (!/^[A-Z][A-Z0-9_]*$/.test(expression))
    throw new Error(`META name is not a literal or local string constant in ${abs}`);

  const owner = lexicalScope(stripped, metaAt);
  const declarations = [
    ...stripped.matchAll(new RegExp(`\\bconst\\s+${expression}\\b`, "g")),
  ].filter((match) => lexicalScope(stripped, match.index) === owner);
  if (declarations.length !== 1)
    throw new Error(`META name requires one local const ${expression} in ${abs}`);
  const at = declarations[0].index;
  const initializer = new RegExp(
    `^const\\s+${expression}\\s*:\\s*&(?:'static\\s+)?str\\s*=\\s*`,
  ).exec(code.slice(at));
  if (!initializer)
    throw new Error(`META name const ${expression} is not a literal &str in ${abs}`);
  const valueStart = at + initializer[0].length;
  const valueEnd = expressionEnd(stripped, valueStart, stripped.length, ";");
  const value = literalRuleName(code.slice(valueStart, valueEnd).trim());
  if (value === null || value.length === 0)
    throw new Error(`META name const ${expression} is not a literal &str in ${abs}`);
  return value;
}

export function parseRuleFile(abs: string, model: HookModel): ParsedRuleFile {
  return parseRuleSource(
    readFileSync(abs, "utf8"),
    path.relative(RULES_DIR, abs).split(path.sep).join("/"),
    model,
    abs,
  );
}

export function parseRuleSource(
  raw: string,
  rel: string,
  model: HookModel,
  abs: string = rel,
): ParsedRuleFile {
  const stripped = stripRust(raw);
  const code = stripRustComments(raw); // strings kept, comments blanked

  // --- meta ---------------------------------------------------------------
  const metaMatch =
    /\bstatic\s+META\s*:\s*(RuleMeta|ScriptRuleMeta|CssRuleMeta|MuseaRuleMeta)\s*=/.exec(stripped);
  let meta: ParsedRuleFile["meta"] = null;
  if (metaMatch) {
    const open = stripped.indexOf("{", metaMatch.index + metaMatch[0].length);
    const close = matchBraceBlock(stripped, open);
    if (open === -1 || close === -1) throw new Error(`META without a complete body in ${abs}`);
    meta = {
      kind: metaMatch[1],
      name: metaRuleName(code, stripped, open, close, metaMatch.index, abs),
    };
  }

  // --- impl blocks + hook sets ---------------------------------------------
  const impls = findImplBlocks(stripped);
  const ruleImplTypes = impls.filter((b) => b.trait === "Rule").map((b) => b.type);
  const hooks = new Set<string>();
  const markupHookSet = new Set<string>();
  let asMarkup = false;
  let jsxLowered = false;
  let scriptUsesAst = false;
  let scriptUsesTemplateAst = false;
  for (const block of impls) {
    if (block.trait === "Rule") {
      for (const [fn, body] of block.fns) {
        if (fn === "as_sfc_script_rule") continue; // SFC capability, never a template/JSX hook.
        if (fn === "as_markup_rule" && /\bSome\b/.test(body)) asMarkup = true;
        else if (fn === "jsx_needs_lowering" && /\btrue\b/.test(body)) jsxLowered = true;
        else if (fn === "run_on_sfc" || model.templateHooks.includes(fn)) hooks.add(fn);
      }
    } else if (block.trait === "MarkupRule") {
      for (const fn of block.fns.keys()) {
        if (model.markupHooks.includes(fn)) markupHookSet.add(fn);
      }
    } else if (block.trait === "ScriptRule") {
      for (const [fn, body] of block.fns) {
        if (fn === "uses_ast" && /\btrue\b/.test(body)) scriptUsesAst = true;
        if (fn === "check_program") scriptUsesAst = true;
        if (fn === "uses_template_ast" && /\btrue\b/.test(body)) scriptUsesTemplateAst = true;
      }
    }
  }

  // --- croquis usage (symbol-aware, P0-7-style) -----------------------------
  const decls = findUseDecls(stripped);
  let masked = stripped;
  for (const d of decls) {
    masked =
      masked.slice(0, d.start) +
      maskKeepNewlines(stripped.slice(d.start, d.end)) +
      masked.slice(d.end);
  }
  const itemAliases = new Map<string, string>(); // localName -> item
  const moduleAliases = new Map<string, string>(); // localName -> module path under vize_croquis
  for (const d of decls) {
    for (const entry of expandUseTree(d.body)) {
      if (entry.segments[0] !== "vize_croquis") continue;
      if (entry.glob) {
        moduleAliases.set("__glob__", entry.segments.slice(1).join("::"));
        continue;
      }
      const last = entry.segments[entry.segments.length - 1];
      const local = entry.alias ?? last;
      if (entry.self || entry.segments.length === 1) {
        moduleAliases.set(local, entry.segments.slice(1).join("::"));
      } else {
        itemAliases.set(local, entry.segments.slice(1).join("::"));
        if (/^[a-z_]/.test(last)) moduleAliases.set(local, entry.segments.slice(1).join("::"));
      }
    }
  }
  const croquisItems = new Map<string, number>(); // item path -> sites
  const bumpItem = (item: string, count: number): void => {
    if (count > 0) croquisItems.set(item, (croquisItems.get(item) ?? 0) + count);
  };
  let text = masked;
  text = text.replace(
    /\bvize_croquis((?:::[A-Za-z_][A-Za-z0-9_]*)+)/g,
    (whole: string, rest: string) => {
      bumpItem(rest.slice(2), 1);
      return " ".repeat(whole.length);
    },
  );
  for (const [local, modulePath] of moduleAliases) {
    if (local === "__glob__") continue;
    const re = new RegExp(`(?<![A-Za-z0-9_:.])${local}::([A-Za-z_][A-Za-z0-9_]*)`, "g");
    text = text.replace(re, (whole: string, member: string) => {
      bumpItem(modulePath === "" ? member : `${modulePath}::${member}`, 1);
      return " ".repeat(whole.length);
    });
  }
  for (const [local, item] of itemAliases) {
    const re = new RegExp(`(?<![A-Za-z0-9_:.'])${local}(?![A-Za-z0-9_!])`, "g");
    bumpItem(item, (text.match(re) ?? []).length);
  }

  // Analysis-product access through the lint context: `.analysis()` /
  // `.has_analysis()` on receivers typed LintContext / MarkupContext /
  // MarkupDocument (the context lane croquis facts flow through).
  const receivers = new Set<string>();
  const annRe =
    /\b([a-z_][a-z0-9_]*)\s*:\s*&(?:'[a-z_][A-Za-z0-9_]*\s*)?(?:mut\s+)?(?:LintContext|MarkupContext|MarkupDocument)\b/g;
  for (const m of masked.matchAll(annRe)) receivers.add(m[1]);
  let ctxSites = 0;
  if (receivers.size > 0) {
    const rAlt = [...receivers].sort(byKey).join("|");
    const callRe = new RegExp(
      `(?<![A-Za-z0-9_])(?:${rAlt})\\s*(?:\\.\\s*lint\\(\\)\\s*)?\\.\\s*(?:analysis|has_analysis)\\s*\\(`,
      "g",
    );
    ctxSites = (masked.match(callRe) ?? []).length;
  }

  // --- classification signals ----------------------------------------------
  const signals: RuleSignals = { container: new Set(), dialect: new Set() };
  for (const m of code.matchAll(CONTAINER_TOKEN_RE)) signals.container.add(m[1] ?? m[0]);
  for (const m of code.matchAll(DIALECT_NODE_RE))
    signals.dialect.add(m[1] ?? "PropNode::Directive");
  for (const m of code.matchAll(DIALECT_SEMANTIC_RE)) signals.dialect.add(m[1]);
  for (const hook of hooks) if (DIALECT_HOOKS.has(hook)) signals.dialect.add(`hook:${hook}`);
  for (const hook of markupHookSet) {
    if (DIALECT_MARKUP_HOOKS.has(hook)) signals.dialect.add(`hook:${hook}`);
  }
  if (hooks.has("run_on_sfc")) signals.container.add("hook:run_on_sfc");
  for (const literal of collectStringLiterals(code)) {
    if (/(?:^|[^A-Za-z0-9_])v-[a-z]/.test(literal)) {
      signals.dialect.add('"v-…" literal');
      break;
    }
  }

  return {
    abs,
    rel,
    source: stripped,
    meta,
    impls,
    ruleImplTypes,
    hooks,
    markupHookSet,
    asMarkup,
    jsxLowered,
    scriptUsesAst,
    scriptUsesTemplateAst,
    croquisItems,
    ctxSites,
    signals,
  };
}
