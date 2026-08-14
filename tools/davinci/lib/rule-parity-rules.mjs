// Per-file rule extraction: META identity, impl/hook shape, symbol-aware
// croquis usage (P0-7's use-declaration resolution), and the lexical signals
// the portability classification is derived from.

import { readFileSync } from "node:fs";
import path from "node:path";

import { byKey } from "./ordering.mjs";
import { RULES_DIR } from "./rule-parity-paths.mjs";
import {
  collectStringLiterals,
  findImplBlocks,
  matchBraceBlock,
  stripRustComments,
} from "./rule-parity-rust-text.mjs";
import { expandUseTree, findUseDecls, maskKeepNewlines, stripRust } from "./rust-source.mjs";

const CONTAINER_TOKEN_RE =
  /\b(SfcDescriptor|SfcBlock|SfcTemplateBlock|SfcScriptBlock|SfcStyleBlock|SfcParseOptions|parse_sfc|sfc_descriptor|sfc_template_descriptor|vize_atelier_sfc)\b/g;
const DIALECT_NODE_RE =
  /\b(DirectiveNode|DirectiveArgumentNode|ForNode|IfNode|IfBranchNode|InterpolationNode|MarkupDirective)\b|PropNode::Directive/g;
const DIALECT_SEMANTIC_RE =
  /\b(defineProps|defineEmits|defineModel|defineExpose|defineOptions|defineSlots|withDefaults|COMPILER_MACRO_NAMES|is_builtin_component|VueDialect|onBeforeMount|onMounted|onBeforeUpdate|onUpdated|onBeforeUnmount|onUnmounted|onActivated|onDeactivated|onServerPrefetch)\b/g;
const DIALECT_HOOKS = new Set(["check_directive", "check_for", "check_if", "check_interpolation"]);
const DIALECT_MARKUP_HOOKS = new Set(["enter_directive"]);
export const DIALECT_NAME_PREFIXES = ["ecosystem", "nuxt", "petite-vue", "vapor"];

export function parseRuleFile(abs, model) {
  const raw = readFileSync(abs, "utf8");
  const stripped = stripRust(raw);
  const code = stripRustComments(raw); // strings kept, comments blanked

  // --- meta ---------------------------------------------------------------
  const metaMatch =
    /static\s+META\s*:\s*(RuleMeta|ScriptRuleMeta|CssRuleMeta|MuseaRuleMeta)\s*=/.exec(code);
  let meta = null;
  if (metaMatch) {
    const open = code.indexOf("{", metaMatch.index);
    const close = matchBraceBlock(code, open);
    const body = code.slice(open + 1, close);
    const nameMatch = /name:\s*"([^"]+)"/.exec(body);
    if (!nameMatch) throw new Error(`META without name in ${abs}`);
    meta = { kind: metaMatch[1], name: nameMatch[1] };
  }

  // --- impl blocks + hook sets ---------------------------------------------
  const impls = findImplBlocks(stripped);
  const ruleImplTypes = impls.filter((b) => b.trait === "Rule").map((b) => b.type);
  const hooks = new Set();
  const markupHookSet = new Set();
  let asMarkup = false;
  let jsxLowered = false;
  let scriptUsesAst = false;
  let scriptUsesTemplateAst = false;
  for (const block of impls) {
    if (block.trait === "Rule") {
      for (const [fn, body] of block.fns) {
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
  const itemAliases = new Map(); // localName -> item
  const moduleAliases = new Map(); // localName -> module path under vize_croquis
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
  const croquisItems = new Map(); // item path -> sites
  const bumpItem = (item, count) => {
    if (count > 0) croquisItems.set(item, (croquisItems.get(item) ?? 0) + count);
  };
  let text = masked;
  text = text.replace(/\bvize_croquis((?:::[A-Za-z_][A-Za-z0-9_]*)+)/g, (whole, rest) => {
    bumpItem(rest.slice(2), 1);
    return " ".repeat(whole.length);
  });
  for (const [local, modulePath] of moduleAliases) {
    if (local === "__glob__") continue;
    const re = new RegExp(`(?<![A-Za-z0-9_:.])${local}::([A-Za-z_][A-Za-z0-9_]*)`, "g");
    text = text.replace(re, (whole, member) => {
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
  const receivers = new Set();
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
  const signals = { container: new Set(), dialect: new Set() };
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
    rel: path.relative(RULES_DIR, abs).split(path.sep).join("/"),
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
