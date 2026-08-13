#!/usr/bin/env node
// Rule-parity matrix generator (Davinci P0-8).
//
// Walks every rule source file under crates/vize_patina/src/rules/** and
// derives, per rule: its registration surface(s), whether it runs on the SFC
// (`Linter::lint_sfc`, the `lint()` family) and/or JSX (`Linter::lint_jsx`)
// paths, its vize_croquis usage (symbol-aware, per P0-7's use-declaration
// resolution), and a first-cut portability classification
// (neutral-core-candidate / vue-dialect-bound / container-bound) for charter
// #7's fairness metric. Hand-corrections live in
// davinci-road/plan/rule-parity-overrides.toml and are applied last.
//
// Usage:
//   node tools/davinci/rule-parity.mjs --write   # regenerate artifact
//   node tools/davinci/rule-parity.mjs --check   # diff against committed
//
// Node builtins only. Output is deterministic (stable sort everywhere,
// no timestamps, no absolute paths).

import { readFileSync, readdirSync, writeFileSync, existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { stripRust, findUseDecls, expandUseTree } from "./lib/rust-source.mjs";
import { formatTable } from "./lib/markdown.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const PATINA_SRC = path.join(repoRoot, "crates", "vize_patina", "src");
const RULES_DIR = path.join(PATINA_SRC, "rules");
const ARTIFACT_REL = "davinci-road/plan/rule-parity.md";
const ARTIFACT = path.join(repoRoot, ARTIFACT_REL);
const OVERRIDES_REL = "davinci-road/plan/rule-parity-overrides.toml";
const OVERRIDES = path.join(repoRoot, OVERRIDES_REL);
const REGEN_COMMAND = "node tools/davinci/rule-parity.mjs --write";

const META_KINDS = new Map([
  ["RuleMeta", "template-family"],
  ["ScriptRuleMeta", "script"],
  ["CssRuleMeta", "css"],
  ["MuseaRuleMeta", "musea"],
]);
const RULE_TRAITS = ["Rule", "MarkupRule", "ScriptRule", "CssRule", "MuseaRule"];
const CLASSIFICATIONS = ["neutral-core-candidate", "vue-dialect-bound", "container-bound"];

// ---------------------------------------------------------------------------
// Extra source pre-processing (comments stripped, string literals kept)
// ---------------------------------------------------------------------------

/**
 * Like stripRust, but keeps string/char literals: only comments are blanked.
 * Used where string contents are the signal (rule names, "v-…" literals).
 */
function stripRustComments(source) {
  const out = source.split("");
  const n = source.length;
  let i = 0;
  const blank = (from, to) => {
    for (let k = from; k < to; k++) if (out[k] !== "\n") out[k] = " ";
  };
  const skipString = (from) => {
    // Assumes source[from] is the opening quote of a plain/byte string.
    let j = from + 1;
    while (j < n && source[j] !== '"') j += source[j] === "\\" ? 2 : 1;
    return j + 1;
  };
  while (i < n) {
    const c = source[i];
    const c2 = source[i + 1];
    if (c === "/" && c2 === "/") {
      let j = i;
      while (j < n && source[j] !== "\n") j++;
      blank(i, j);
      i = j;
    } else if (c === "/" && c2 === "*") {
      let depth = 1;
      let j = i + 2;
      while (j < n && depth > 0) {
        if (source[j] === "/" && source[j + 1] === "*") {
          depth++;
          j += 2;
        } else if (source[j] === "*" && source[j + 1] === "/") {
          depth--;
          j += 2;
        } else {
          j++;
        }
      }
      blank(i, j);
      i = j;
    } else if (c === "r" || ((c === "b" || c === "c") && (c2 === "r" || c2 === '"'))) {
      let j = i;
      if (c === "b" || c === "c") j++;
      if (source[j] === "r") {
        j++;
        let hashes = 0;
        while (source[j] === "#") {
          hashes++;
          j++;
        }
        if (source[j] !== '"') {
          i++;
          continue;
        }
        j++;
        const closer = '"' + "#".repeat(hashes);
        const end = source.indexOf(closer, j);
        i = end === -1 ? n : end + closer.length;
      } else if (source[j] === '"') {
        i = skipString(j);
      } else {
        i++;
      }
    } else if (c === '"') {
      i = skipString(i);
    } else if (c === "'") {
      if (c2 === "\\") {
        let j = i + 2;
        while (j < n && source[j] !== "'") j++;
        i = j + 1;
      } else if (source[i + 2] === "'") {
        i += 3;
      } else {
        i++;
      }
    } else if (/[A-Za-z0-9_]/.test(c)) {
      let j = i;
      while (j < n && /[A-Za-z0-9_]/.test(source[j])) j++;
      i = j;
    } else {
      i++;
    }
  }
  return out.join("");
}

/** Collect the contents of "…" / r#"…"# / b"…" string literals in code text. */
function collectStringLiterals(code) {
  const literals = [];
  const n = code.length;
  let i = 0;
  while (i < n) {
    const c = code[i];
    const c2 = code[i + 1];
    if (c === "r" || ((c === "b" || c === "c") && (c2 === "r" || c2 === '"'))) {
      let j = i;
      if (c === "b" || c === "c") j++;
      if (code[j] === "r") {
        j++;
        let hashes = 0;
        while (code[j] === "#") {
          hashes++;
          j++;
        }
        if (code[j] !== '"') {
          i++;
          continue;
        }
        j++;
        const closer = '"' + "#".repeat(hashes);
        const end = code.indexOf(closer, j);
        literals.push(code.slice(j, end === -1 ? n : end));
        i = end === -1 ? n : end + closer.length;
      } else if (code[j] === '"') {
        let k = j + 1;
        while (k < n && code[k] !== '"') k += code[k] === "\\" ? 2 : 1;
        literals.push(code.slice(j + 1, k));
        i = k + 1;
      } else {
        i++;
      }
    } else if (c === '"') {
      let k = i + 1;
      while (k < n && code[k] !== '"') k += code[k] === "\\" ? 2 : 1;
      literals.push(code.slice(i + 1, k));
      i = k + 1;
    } else if (/[A-Za-z0-9_]/.test(c)) {
      let j = i;
      while (j < n && /[A-Za-z0-9_]/.test(code[j])) j++;
      i = j;
    } else {
      i++;
    }
  }
  return literals;
}

function matchBraceBlock(text, openIdx) {
  let depth = 0;
  for (let i = openIdx; i < text.length; i++) {
    if (text[i] === "{") depth++;
    else if (text[i] === "}") {
      depth--;
      if (depth === 0) return i;
    }
  }
  return -1;
}

/**
 * Find `impl <Trait> for <Type> { … }` blocks for the rule traits and list the
 * fns defined at the impl's top level, with each fn's body text.
 */
function findImplBlocks(stripped) {
  const blocks = [];
  const re = new RegExp(
    `\\bimpl\\s+(${RULE_TRAITS.join("|")})\\s+for\\s+([A-Za-z_][A-Za-z0-9_]*)`,
    "g",
  );
  let m;
  while ((m = re.exec(stripped)) !== null) {
    const open = stripped.indexOf("{", m.index + m[0].length);
    if (open === -1) continue;
    const close = matchBraceBlock(stripped, open);
    if (close === -1) continue;
    const body = stripped.slice(open + 1, close);
    const fns = new Map(); // fnName -> body text
    const fnRe = /\bfn\s+([a-z_][a-z0-9_]*)/g;
    let fm;
    while ((fm = fnRe.exec(body)) !== null) {
      // Only fns at the impl's own level: no unclosed brace before this fn.
      const before = body.slice(0, fm.index);
      let depth = 0;
      for (const ch of before) {
        if (ch === "{") depth++;
        else if (ch === "}") depth--;
      }
      if (depth !== 0) continue;
      const fnOpen = body.indexOf("{", fnRe.lastIndex);
      if (fnOpen === -1) continue;
      const fnClose = matchBraceBlock(body, fnOpen);
      if (fnClose === -1) continue;
      fns.set(fm[1], body.slice(fnOpen + 1, fnClose));
    }
    blocks.push({ trait: m[1], type: m[2], fns });
    re.lastIndex = close;
  }
  return blocks;
}

// ---------------------------------------------------------------------------
// Trait hook inventories (parsed from rule.rs / markup.rs, not hardcoded)
// ---------------------------------------------------------------------------

function traitFnNames(stripped, traitName) {
  const m = new RegExp(`\\btrait\\s+${traitName}\\b`).exec(stripped);
  if (!m) throw new Error(`trait ${traitName} not found`);
  const open = stripped.indexOf("{", m.index);
  const close = matchBraceBlock(stripped, open);
  const body = stripped.slice(open + 1, close);
  const names = [];
  const fnRe = /\bfn\s+([a-z_][a-z0-9_]*)/g;
  let fm;
  while ((fm = fnRe.exec(body)) !== null) {
    const before = body.slice(0, fm.index);
    let depth = 0;
    for (const ch of before) {
      if (ch === "{") depth++;
      else if (ch === "}") depth--;
    }
    if (depth === 0) names.push(fm[1]);
  }
  return names;
}

function loadDispatchModel() {
  const ruleRs = stripRust(readFileSync(path.join(PATINA_SRC, "rule.rs"), "utf8"));
  const markupRs = stripRust(readFileSync(path.join(PATINA_SRC, "markup.rs"), "utf8"));
  const engineRs = readFileSync(path.join(PATINA_SRC, "linter", "engine.rs"), "utf8");
  const engineStripped = stripRust(engineRs);

  const ruleFns = traitFnNames(ruleRs, "Rule");
  for (const anchor of ["meta", "as_markup_rule", "jsx_needs_lowering", "run_on_sfc"]) {
    if (!ruleFns.includes(anchor)) {
      throw new Error(`dispatch model drift: trait Rule lost fn ${anchor}; re-derive P0-8`);
    }
  }
  const templateHooks = ruleFns.filter(
    (f) => !["meta", "as_markup_rule", "jsx_needs_lowering", "run_on_sfc"].includes(f),
  );

  const markupFns = traitFnNames(markupRs, "MarkupRule");
  if (!markupFns.includes("name")) {
    throw new Error("dispatch model drift: trait MarkupRule lost fn name");
  }
  const markupHooks = markupFns.filter((f) => f !== "name");

  // lint_jsx dispatch anchors: the three-lane partition this matrix models.
  const jsxStart = /pub\s+fn\s+lint_jsx\b/.exec(engineStripped);
  if (!jsxStart) throw new Error("dispatch model drift: Linter::lint_jsx not found in engine.rs");
  const jsxOpen = engineStripped.indexOf("{", jsxStart.index);
  const jsxClose = matchBraceBlock(engineStripped, jsxOpen);
  const jsxBody = engineStripped.slice(jsxOpen + 1, jsxClose);
  for (const anchor of [
    "as_markup_rule",
    "jsx_needs_lowering",
    "legacy_keep_mask",
    "lint_jsx_over_ir",
    "lint_jsx_lowered_markup_root",
    "lint_jsx_fallback_root",
  ]) {
    if (!jsxBody.includes(anchor)) {
      throw new Error(`dispatch model drift: lint_jsx body lost anchor ${anchor}`);
    }
  }
  // The JSX path must not drive SFC hooks, the script/css block registries,
  // nor the corsa type-aware session; those absences are what make the
  // corresponding rows SFC-only below.
  for (const absent of ["run_on_sfc", "script_rules", "css_rules", "native_type_aware"]) {
    if (jsxBody.includes(absent)) {
      throw new Error(
        `dispatch model drift: lint_jsx body now references ${absent}; re-derive path membership`,
      );
    }
  }
  if ((engineStripped.match(/\.run_on_sfc\(/g) ?? []).length !== 1) {
    throw new Error("dispatch model drift: expected exactly one run_on_sfc dispatch in engine.rs");
  }
  // The SFC path must still drive the script/css block registries.
  for (const anchor of ["script_rules", "css_rules"]) {
    if (!engineStripped.includes(anchor)) {
      throw new Error(`dispatch model drift: engine.rs lost the ${anchor} dispatch`);
    }
  }
  if (engineStripped.includes("MuseaLinter")) {
    throw new Error(
      "dispatch model drift: MuseaLinter is now dispatched from engine.rs; musea rows are wrong",
    );
  }

  // Engine rule-name sets gating shared analysis work.
  const ruleSets = stripRustComments(
    readFileSync(path.join(PATINA_SRC, "linter", "engine", "rule_sets.rs"), "utf8"),
  );
  const readSet = (text, name) => {
    const m2 = new RegExp(`${name}\\s*:\\s*&\\[&str\\]\\s*=\\s*&\\[`).exec(text);
    if (!m2) throw new Error(`rule set ${name} not found`);
    const end = text.indexOf("]", m2.index + m2[0].length);
    const body = text.slice(m2.index + m2[0].length, end);
    return [...body.matchAll(/"([^"]+)"/g)].map((x) => x[1]).sort();
  };

  // Rules served by the corsa type-aware session (SFC-only, native-only): the
  // TYPE_AWARE_RULES list names const idents whose string values live in the
  // same file.
  const typeAwareRs = stripRustComments(
    readFileSync(path.join(PATINA_SRC, "linter", "native_type_aware.rs"), "utf8"),
  );
  const typeAwareConsts = new Map();
  for (const m2 of typeAwareRs.matchAll(/const\s+([A-Z_0-9]+)\s*:\s*&str\s*=\s*"([^"]+)"/g)) {
    typeAwareConsts.set(m2[1], m2[2]);
  }
  const typeAwareList = /TYPE_AWARE_RULES\s*:\s*&\[&str\]\s*=\s*&\[([^\]]*)\]/.exec(typeAwareRs);
  if (!typeAwareList) throw new Error("TYPE_AWARE_RULES not found in native_type_aware.rs");
  const typeAwareRules = [...typeAwareList[1].matchAll(/[A-Z_0-9]{2,}/g)]
    .map((m2) => {
      const value = typeAwareConsts.get(m2[0]);
      if (!value) throw new Error(`TYPE_AWARE_RULES const ${m2[0]} has no string value`);
      return value;
    })
    .sort();

  return {
    templateHooks,
    markupHooks,
    semanticTemplateRules: readSet(ruleSets, "SEMANTIC_TEMPLATE_RULES"),
    sharedSfcDescriptorRules: readSet(ruleSets, "SHARED_SFC_DESCRIPTOR_RULES"),
    typeAwareRules,
  };
}

// ---------------------------------------------------------------------------
// Registration (which rules the dispatch paths can actually reach)
// ---------------------------------------------------------------------------

function walkRustFiles(dir) {
  const files = [];
  const visit = (d) => {
    for (const dirent of readdirSync(d, { withFileTypes: true }).sort((a, b) =>
      a.name < b.name ? -1 : a.name > b.name ? 1 : 0,
    )) {
      const full = path.join(d, dirent.name);
      if (dirent.isDirectory()) visit(full);
      else if (dirent.isFile() && dirent.name.endsWith(".rs")) files.push(full);
    }
  };
  visit(dir);
  return files;
}

/** Type names registered into a RuleRegistry anywhere in vize_patina/src. */
function collectRegisteredRuleTypes() {
  const types = new Set();
  const patterns = [
    // registry.register(Box::new(Type…)) / registry.add(Box::new(Type…))
    /\.\s*(?:register|add)\s*\(\s*Box::new\s*\(\s*((?:[A-Za-z_][A-Za-z0-9_]*::)*)([A-Z][A-Za-z0-9_]*)/g,
    // register_if_missing(registry, Box::new(Type…))
    /register_if_missing\s*\(\s*registry\s*,\s*Box::new\s*\(\s*((?:[A-Za-z_][A-Za-z0-9_]*::)*)([A-Z][A-Za-z0-9_]*)/g,
  ];
  for (const abs of walkRustFiles(PATINA_SRC)) {
    const stripped = stripRust(readFileSync(abs, "utf8"));
    for (const re of patterns) {
      re.lastIndex = 0;
      let m;
      while ((m = re.exec(stripped)) !== null) types.add(m[2]);
    }
  }
  return types;
}

/** Rule names in the built-in script registry the SFC path dispatches. */
function collectScriptRegistryNames() {
  const namesRs = stripRustComments(
    readFileSync(path.join(PATINA_SRC, "linter", "script_rules", "registry", "names.rs"), "utf8"),
  );
  const consts = new Map();
  for (const m of namesRs.matchAll(/const\s+([A-Z_0-9]+)\s*:\s*&str\s*=\s*"([^"]+)"/g)) {
    consts.set(m[1], m[2]);
  }
  const rulesRs = stripRustComments(
    readFileSync(path.join(PATINA_SRC, "linter", "script_rules", "registry", "rules.rs"), "utf8"),
  );
  const registered = new Set();
  for (const m of rulesRs.matchAll(/rule_name:\s*([A-Z_0-9]+)/g)) {
    const value = consts.get(m[1]);
    if (!value) throw new Error(`script registry rule_name const ${m[1]} not found in names.rs`);
    registered.add(value);
  }
  // Cross-check against the ALL list in names.rs (count only; both live there).
  const allList = /ALL_BUILTIN_SCRIPT_RULE_NAMES\s*:\s*&\[&str\]\s*=\s*&\[([^\]]*)\]/.exec(namesRs);
  const allCount = allList ? [...allList[1].matchAll(/[A-Z_0-9]+/g)].length : -1;
  return { registered, allCount };
}

/** Rule names in the built-in css registry the SFC path dispatches. */
function collectCssRegistryNames() {
  const cssRs = stripRustComments(
    readFileSync(path.join(PATINA_SRC, "linter", "css_rules.rs"), "utf8"),
  );
  const consts = new Map();
  for (const m of cssRs.matchAll(/const\s+(RULE_[A-Z_0-9]+)\s*:\s*&str\s*=\s*"([^"]+)"/g)) {
    consts.set(m[1], m[2]);
  }
  const allList = /ALL_BUILTIN_CSS_RULE_NAMES\s*:\s*&\[&str\]\s*=\s*&\[([^\]]*)\]/.exec(cssRs);
  if (!allList) throw new Error("ALL_BUILTIN_CSS_RULE_NAMES not found in css_rules.rs");
  const registered = new Set();
  for (const m of allList[1].matchAll(/RULE_[A-Z_0-9]+/g)) {
    const value = consts.get(m[0]);
    if (!value) throw new Error(`css registry const ${m[0]} has no string value`);
    registered.add(value);
  }
  return registered;
}

// ---------------------------------------------------------------------------
// Per-file rule extraction
// ---------------------------------------------------------------------------

function maskKeepNewlines(text) {
  return text.replace(/[^\n]/g, " ");
}

const CONTAINER_TOKEN_RE =
  /\b(SfcDescriptor|SfcBlock|SfcTemplateBlock|SfcScriptBlock|SfcStyleBlock|SfcParseOptions|parse_sfc|sfc_descriptor|sfc_template_descriptor|vize_atelier_sfc)\b/g;
const DIALECT_NODE_RE =
  /\b(DirectiveNode|DirectiveArgumentNode|ForNode|IfNode|IfBranchNode|InterpolationNode|MarkupDirective)\b|PropNode::Directive/g;
const DIALECT_SEMANTIC_RE =
  /\b(defineProps|defineEmits|defineModel|defineExpose|defineOptions|defineSlots|withDefaults|COMPILER_MACRO_NAMES|is_builtin_component|VueDialect|onBeforeMount|onMounted|onBeforeUpdate|onUpdated|onBeforeUnmount|onUnmounted|onActivated|onDeactivated|onServerPrefetch)\b/g;
const DIALECT_HOOKS = new Set(["check_directive", "check_for", "check_if", "check_interpolation"]);
const DIALECT_MARKUP_HOOKS = new Set(["enter_directive"]);
const DIALECT_NAME_PREFIXES = ["ecosystem", "nuxt", "petite-vue", "vapor"];

function parseRuleFile(abs, model) {
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
    const rAlt = [...receivers].sort().join("|");
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

// ---------------------------------------------------------------------------
// Overrides sidecar
// ---------------------------------------------------------------------------

function parseOverrides() {
  if (!existsSync(OVERRIDES)) {
    throw new Error(`${OVERRIDES_REL} is missing; commit it (empty is fine)`);
  }
  const overrides = new Map(); // rule name -> { classification, reason }
  let current = null;
  const lines = readFileSync(OVERRIDES, "utf8").split("\n");
  for (const [idx, rawLine] of lines.entries()) {
    const line = rawLine.replace(/(^|\s)#.*$/, "").trim();
    if (line === "") continue;
    const header = /^\[overrides\."([^"]+)"\]$/.exec(line);
    if (header) {
      current = { classification: null, reason: null };
      if (overrides.has(header[1])) {
        throw new Error(`${OVERRIDES_REL}:${idx + 1}: duplicate override for ${header[1]}`);
      }
      overrides.set(header[1], current);
      continue;
    }
    const kv = /^([a-z_]+)\s*=\s*"([^"]*)"$/.exec(line);
    if (!kv || current === null) {
      throw new Error(
        `${OVERRIDES_REL}:${idx + 1}: unrecognized line (schema: [overrides."rule"], classification = "…", reason = "…")`,
      );
    }
    if (kv[1] === "classification") {
      if (!CLASSIFICATIONS.includes(kv[2])) {
        throw new Error(`${OVERRIDES_REL}:${idx + 1}: invalid classification "${kv[2]}"`);
      }
      current.classification = kv[2];
    } else if (kv[1] === "reason") {
      current.reason = kv[2];
    } else {
      throw new Error(`${OVERRIDES_REL}:${idx + 1}: unknown key "${kv[1]}"`);
    }
  }
  for (const [name, o] of overrides) {
    if (!o.classification || !o.reason) {
      throw new Error(`${OVERRIDES_REL}: override for ${name} needs classification and reason`);
    }
  }
  return overrides;
}

// ---------------------------------------------------------------------------
// Matrix assembly
// ---------------------------------------------------------------------------

function byKey(a, b) {
  return a < b ? -1 : a > b ? 1 : 0;
}

function buildMatrix() {
  const model = loadDispatchModel();
  const registeredTypes = collectRegisteredRuleTypes();
  const scriptRegistry = collectScriptRegistryNames();
  const cssRegistry = collectCssRegistryNames();
  const overrides = parseOverrides();

  const files = walkRustFiles(RULES_DIR).map((abs) => parseRuleFile(abs, model));

  // File accounting: a rule file is a file with a META static (each declares
  // exactly one rule identity; preset variants share the file's single META).
  const ruleFiles = files.filter((f) => f.meta !== null);
  const nonRuleFiles = files.filter((f) => f.meta === null);
  for (const f of nonRuleFiles) {
    if (f.impls.some((b) => b.trait !== "MarkupRule")) {
      throw new Error(`rule trait impl without META in ${f.rel}; the file model is stale`);
    }
    const stem = f.rel.replace(/\.rs$/, "");
    if (f.rel.endsWith("_tests.rs")) f.kind = "test";
    else if (files.some((g) => g.rel.startsWith(stem + "/"))) f.kind = "module";
    else f.kind = "helper";
  }

  const rules = new Map();
  for (const f of ruleFiles) {
    if (rules.has(f.meta.name)) {
      throw new Error(
        `duplicate rule name ${f.meta.name} (${rules.get(f.meta.name).file} and ${f.rel})`,
      );
    }
    const family = META_KINDS.get(f.meta.kind);
    const rule = {
      name: f.meta.name,
      family,
      file: f.rel,
      surfaces: [],
      sfc: "no",
      sfcDetail: "",
      jsx: "no",
      jsxLane: "none",
      croquis: f.croquisItems,
      ctxSites: f.ctxSites,
      registered: false,
      classification: null,
      signals: f.signals,
      overridden: false,
      overrideReason: null,
    };

    if (family === "template-family") {
      const templateHookSet = [...f.hooks].filter((h) => h !== "run_on_sfc");
      const corsa = model.typeAwareRules.includes(f.meta.name);
      if (templateHookSet.length > 0) rule.surfaces.push("template-ast");
      if (f.hooks.has("run_on_sfc")) rule.surfaces.push("sfc-source");
      if (f.impls.some((b) => b.trait === "MarkupRule")) rule.surfaces.push("markup-facade");
      if (corsa) rule.surfaces.push("type-aware-corsa");
      if (rule.surfaces.length === 0) rule.surfaces.push("none");
      rule.registered = f.ruleImplTypes.some((t) => registeredTypes.has(t));
      if (rule.registered) {
        rule.sfc = "yes";
        const mechanisms = [];
        if (templateHookSet.length > 0) mechanisms.push("template-visitor");
        if (f.hooks.has("run_on_sfc")) mechanisms.push("sfc-hooks");
        if (corsa) mechanisms.push("corsa");
        rule.sfcDetail = mechanisms.length > 0 ? mechanisms.join("+") : "no-op-hooks";
        if (f.asMarkup) {
          rule.jsx = "yes";
          rule.jsxLane = f.jsxLowered ? "ir-lowered" : "ir";
        } else if (templateHookSet.length > 0) {
          rule.jsx = "yes";
          rule.jsxLane = "fallback";
        } else {
          rule.jsx = "no";
          rule.jsxLane = "no-jsx-hooks";
        }
      }
    } else if (family === "script") {
      rule.surfaces.push(f.scriptUsesAst ? "script-oxc" : "script-source");
      if (f.scriptUsesTemplateAst) rule.surfaces.push("template-ast");
      rule.registered = scriptRegistry.registered.has(f.meta.name);
      if (rule.registered) {
        rule.sfc = "yes";
        rule.sfcDetail = "script-blocks";
      }
    } else if (family === "css") {
      rule.surfaces.push("css-text");
      rule.registered = cssRegistry.has(f.meta.name);
      if (rule.registered) {
        rule.sfc = "yes";
        rule.sfcDetail = "style-blocks";
      }
    } else if (family === "musea") {
      rule.surfaces.push("musea-blocks");
      rule.registered = false; // MuseaLinter is its own surface, not lint()/lint_jsx()
    }

    // Classification: container > dialect > neutral, then overrides.
    const containerSignal =
      f.signals.container.size > 0 ||
      family === "musea" ||
      model.sharedSfcDescriptorRules.includes(f.meta.name);
    if (family === "musea") f.signals.container.add("musea Art-file blocks");
    if (model.sharedSfcDescriptorRules.includes(f.meta.name)) {
      f.signals.container.add("SHARED_SFC_DESCRIPTOR_RULES");
    }
    const prefix = f.meta.name.split("/")[0];
    if (DIALECT_NAME_PREFIXES.includes(prefix)) f.signals.dialect.add(`prefix:${prefix}/`);
    const dialectSignal = f.signals.dialect.size > 0;
    rule.classification = containerSignal
      ? "container-bound"
      : dialectSignal
        ? "vue-dialect-bound"
        : "neutral-core-candidate";

    rules.set(rule.name, rule);
  }

  for (const [name, o] of overrides) {
    const rule = rules.get(name);
    if (!rule) throw new Error(`${OVERRIDES_REL}: override for unknown rule "${name}"`);
    rule.overridden = rule.classification !== o.classification;
    rule.classification = o.classification;
    rule.overrideReason = o.reason;
  }

  return { model, files, ruleFiles, nonRuleFiles, rules, scriptRegistry, cssRegistry, overrides };
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

function renderArtifact(matrix) {
  const { model, files, ruleFiles, nonRuleFiles, rules } = matrix;
  const rows = [...rules.values()].sort((a, b) => byKey(a.name, b.name));

  const count = (pred) => rows.filter(pred).length;
  const familyCounts = new Map();
  for (const kind of META_KINDS.values()) {
    familyCounts.set(
      kind,
      count((r) => r.family === kind),
    );
  }
  const surfaceCounts = new Map();
  for (const r of rows) {
    for (const s of r.surfaces) surfaceCounts.set(s, (surfaceCounts.get(s) ?? 0) + 1);
  }
  const sfcSet = rows.filter((r) => r.sfc === "yes");
  const jsxSet = rows.filter((r) => r.jsx === "yes");
  const both = rows.filter((r) => r.sfc === "yes" && r.jsx === "yes");
  const sfcOnly = rows.filter((r) => r.sfc === "yes" && r.jsx !== "yes");
  const jsxOnly = rows.filter((r) => r.sfc !== "yes" && r.jsx === "yes");
  const neither = rows.filter((r) => r.sfc !== "yes" && r.jsx !== "yes");
  const laneCounts = new Map();
  for (const r of rows) {
    if (r.jsxLane !== "none") laneCounts.set(r.jsxLane, (laneCounts.get(r.jsxLane) ?? 0) + 1);
  }
  const croquisUsers = rows.filter((r) => r.croquis.size > 0 || r.ctxSites > 0);
  const classCounts = new Map();
  for (const c of CLASSIFICATIONS)
    classCounts.set(
      c,
      count((r) => r.classification === c),
    );
  const overriddenRows = rows.filter((r) => r.overrideReason !== null);
  const unregistered = rows.filter((r) => !r.registered && r.family !== "musea");

  const moduleFiles = nonRuleFiles.filter((f) => f.kind === "module");
  const testFiles = nonRuleFiles.filter((f) => f.kind === "test");
  const helperFiles = nonRuleFiles.filter((f) => f.kind === "helper");

  const croquisCell = (r) => {
    const parts = [];
    if (r.croquis.size > 0) {
      const items = [...r.croquis.keys()].sort(byKey);
      const shown = items.slice(0, 3).map((i) => `\`${i}\``);
      if (items.length > 3) shown.push(`+${items.length - 3}`);
      const sites = [...r.croquis.values()].reduce((a, b) => a + b, 0);
      parts.push(`direct ${sites}: ${shown.join(", ")}`);
    }
    if (r.ctxSites > 0) parts.push(`ctx ${r.ctxSites}`);
    return parts.length > 0 ? parts.join("; ") : "—";
  };
  const jsxCell = (r) => {
    if (r.jsx === "yes") return `yes (${r.jsxLane})`;
    if (r.jsxLane === "no-jsx-hooks") return "no (no JSX-reachable hooks)";
    return "no";
  };
  const sfcCell = (r) => (r.sfc === "yes" ? `yes (${r.sfcDetail})` : "no");

  const lines = [];
  lines.push("<!-- GENERATED FILE — do not edit by hand.");
  lines.push(`     Regenerate: ${REGEN_COMMAND}`);
  lines.push("     Verify:     node tools/davinci/rule-parity.mjs --check");
  lines.push("     Generator:  tools/davinci/rule-parity.mjs");
  lines.push(`     Overrides:  ${OVERRIDES_REL} (hand-corrections; applied last) -->`);
  lines.push("");
  lines.push("# Rule-parity matrix (SFC × JSX)");
  lines.push("");
  lines.push(
    "Per-rule registration surface, SFC/JSX path membership, croquis usage, and" +
      " a first-cut portability classification for every lint rule under" +
      " `crates/vize_patina/src/rules/**` — the classification substrate for" +
      " charter #7's fairness metric (one rule corpus serving SFC and JSX at" +
      " parity) and the fact-adoption target in charter #35 (Davinci P0-8).",
  );
  lines.push("");

  lines.push("## How each column is derived");
  lines.push("");
  lines.push(
    "- **rule identity** — the `name` field of the file's `static META:" +
      " RuleMeta | ScriptRuleMeta | CssRuleMeta | MuseaRuleMeta`. Every rule" +
      " file declares exactly one META; a file with several `impl … for` preset" +
      " variants sharing that META (e.g. `HtmlSelfClosing` +" +
      " `HtmlSelfClosingNuxt`) is one rule.",
  );
  lines.push(
    "- **surfaces** — from the traits the file implements and the hooks each" +
      " impl overrides: `template-ast` (any `Rule` template-visitor hook — the" +
      " hook list is parsed from `trait Rule` in `src/rule.rs`, currently " +
      model.templateHooks.map((h) => "`" + h + "`").join(", ") +
      "), `sfc-source` (`run_on_sfc`), `markup-facade` (`impl MarkupRule`," +
      " the zero-copy IR in `src/markup.rs`), `type-aware-corsa` (listed in" +
      " `TYPE_AWARE_RULES` in `linter/native_type_aware.rs` — served by the" +
      " corsa type-checker session, native-only), `script-oxc` /" +
      " `script-source` (`impl ScriptRule`, split on `uses_ast`/" +
      "`check_program`), `css-text` (`impl CssRule`), `musea-blocks` (musea" +
      " family).",
  );
  lines.push(
    "- **lint() (SFC path)** — a template-family rule runs iff one of its impl" +
      " types is registered into a `RuleRegistry` (every `register…(Box::new(…))`" +
      " site across `vize_patina/src` is collected mechanically); `lint_sfc`" +
      " then drives `run_on_sfc` plus the template visitor over every" +
      " registered rule — and, for `TYPE_AWARE_RULES`, the corsa session" +
      " (`no-op-hooks` cells cannot occur: a hook-less rule reaches SFC only" +
      " via corsa, shown as `corsa`). Script rules run iff listed in" +
      " `linter/script_rules/registry` (dispatched per `<script>` block; the" +
      " same registry also serves plain-JS `lint_script` and inline" +
      " `<script>` in `lint_standalone_html`). CSS rules run iff listed in" +
      " `ALL_BUILTIN_CSS_RULE_NAMES` (dispatched per `<style>` block). Musea" +
      " rules run only under `MuseaLinter` (Art files) — neither path here.",
  );
  lines.push(
    "- **lint_jsx() (JSX path)** — derived from the three-lane partition in" +
      " `Linter::lint_jsx` (`linter/engine.rs`): `ir` = overrides" +
      " `as_markup_rule` → runs over the zero-cost OXC projection;" +
      " `ir-lowered` = additionally `jsx_needs_lowering() == true` → runs over" +
      " the lowered relief AST via the markup visitor; `fallback` = no markup" +
      " projection but has template-visitor hooks → runs over the lowered" +
      " relief AST via the legacy visitor (`legacy_keep_mask`). A registered" +
      " rule with no markup projection and no template-visitor hook —" +
      " `run_on_sfc`-only rules and corsa-only rules — is dispatched but can" +
      " never fire (`lint_jsx` calls neither `run_on_sfc` nor the corsa" +
      " session) — shown as `no (no JSX-reachable hooks)`. Script/CSS" +
      " registries, the corsa session, and `MuseaLinter` are never invoked" +
      " from `lint_jsx`; the generator asserts those absences (and the lane" +
      " anchors) against the current dispatch source and fails on drift.",
  );
  lines.push(
    "- **croquis** — symbol-aware, per P0-7's use-declaration resolution:" +
      " `use vize_croquis::…` aliases (braces, `as`, module aliases) are" +
      " resolved per file and reference sites counted in comment/string-" +
      "stripped code (`direct N`), plus `.analysis()` / `.has_analysis()`" +
      " calls on receivers typed `LintContext` / `MarkupContext` /" +
      " `MarkupDocument` — the lane croquis facts reach rules through" +
      " (`ctx N`). No raw text matching.",
  );
  lines.push(
    "- **classification** — heuristic, precedence container > dialect >" +
      " neutral: **container-bound** if the file touches SFC block structure" +
      " (`SfcDescriptor`/`Sfc*Block`/`parse_sfc`/`sfc_descriptor`/" +
      "`vize_atelier_sfc`), implements `run_on_sfc`, appears in the engine's" +
      " `SHARED_SFC_DESCRIPTOR_RULES`, or is a musea rule (Art-file blocks);" +
      " **vue-dialect-bound** if it references directive-specific node kinds" +
      " (`DirectiveNode`, `ForNode`, `IfNode`, `InterpolationNode`," +
      " `MarkupDirective`, `PropNode::Directive`, …), implements a" +
      " directive-shaped hook (`check_directive`/`check_for`/`check_if`/" +
      '`check_interpolation`/`enter_directive`), contains a `"v-…"` string' +
      " literal, references Vue-only semantics (compiler macros, lifecycle" +
      " names, `is_builtin_component`, `VueDialect`), or belongs to a" +
      " Vue-ecosystem name prefix (" +
      DIALECT_NAME_PREFIXES.map((p) => "`" + p + "/`").join(", ") +
      "); otherwise **neutral-core-candidate**. Overrides from" +
      ` \`${OVERRIDES_REL}\` are applied last; overridden rows are marked \`*\`.`,
  );
  lines.push("");
  lines.push("### Heuristic limitations");
  lines.push("");
  lines.push(
    "- Signals are lexical. A rule that _mentions_ a directive node in shared" +
      " helper code is classified dialect-bound even if its core check is" +
      " neutral; conversely Vue semantics reached through helpers in _other_" +
      " files are invisible. Record corrections in the overrides sidecar, not" +
      " here.",
  );
  lines.push(
    "- `check_interpolation`/`InterpolationNode` count as dialect signals" +
      " (mustache syntax) even though interpolation has a JSX analogue; the" +
      " markup-facade `enter_interpolation`/`enter_conditional`/`enter_list`" +
      " hooks count as neutral (they abstract both syntaxes).",
  );
  lines.push(
    "- Path membership is dispatch-shape, not runtime effect: a rule that" +
      " data-gates on `ctx.analysis()` (see `SEMANTIC_TEMPLATE_RULES`) may" +
      " no-op on JSX when the engine computes no analysis, and a `fallback`" +
      " rule checking Vue-only syntax simply never matches in JSX — both still" +
      " _run_ there. `#[cfg]`-gated registrations (e.g. type-aware rules," +
      " native-only) count as registered.",
  );
  lines.push("- Macro-generated code is invisible to source parsing.");
  lines.push("");

  lines.push("## File accounting");
  lines.push("");
  lines.push(`- \`.rs\` files under \`crates/vize_patina/src/rules/**\`: **${files.length}**`);
  lines.push(
    `- rule-defining files (exactly one \`static META\` each): **${ruleFiles.length}**` +
      ` → **${rows.length} rules**`,
  );
  lines.push(
    `- non-rule files: **${nonRuleFiles.length}** — ${moduleFiles.length} module organizers` +
      ` (a \`<name>.rs\` with a \`<name>/\` directory beside it), ${testFiles.length}` +
      ` \`*_tests.rs\` companions, ${helperFiles.length} helper/data files (rule submodules,` +
      ` shared tables, private utilities)`,
  );
  lines.push("");

  lines.push("## Summary");
  lines.push("");
  lines.push(`- **total rules: ${rows.length}**`);
  lines.push("- by family: " + [...familyCounts.entries()].map(([k, v]) => `${k} ${v}`).join(", "));
  lines.push(
    "- by surface (a rule can have several): " +
      [...surfaceCounts.entries()]
        .sort((a, b) => byKey(a[0], b[0]))
        .map(([k, v]) => `\`${k}\` ${v}`)
        .join(", "),
  );
  lines.push(
    `- path membership: SFC \`lint_sfc\` ${sfcSet.length} · JSX \`lint_jsx\` ${jsxSet.length}` +
      ` · **SFC∩JSX ${both.length}** · SFC-only ${sfcOnly.length} · JSX-only ${jsxOnly.length}` +
      ` · neither ${neither.length} (${count((r) => r.family === "musea")} musea +` +
      ` ${unregistered.length} unregistered)`,
  );
  lines.push(
    "- JSX lanes: " +
      [...laneCounts.entries()]
        .sort((a, b) => byKey(a[0], b[0]))
        .map(([k, v]) => `\`${k}\` ${v}`)
        .join(", ") +
      " — `ir` + `ir-lowered` is the markup-facade migration list" +
      ` (${(laneCounts.get("ir") ?? 0) + (laneCounts.get("ir-lowered") ?? 0)} =` +
      ` ${surfaceCounts.get("markup-facade") ?? 0} \`markup-facade\` rules)`,
  );
  lines.push(
    `- classification: ` +
      CLASSIFICATIONS.map((c) => `${c} **${classCounts.get(c)}**`).join(" · ") +
      ` (${overriddenRows.length} overridden)`,
  );
  lines.push(
    `- croquis adoption: **${croquisUsers.length}** rules touch vize_croquis` +
      ` (${count((r) => r.croquis.size > 0)} direct imports,` +
      ` ${count((r) => r.ctxSites > 0)} via context analysis)`,
  );
  lines.push("");

  lines.push("## Full table");
  lines.push("");
  lines.push("Sorted by rule name. File paths are relative to `crates/vize_patina/src/rules/`.");
  lines.push("");
  lines.push(
    formatTable(
      [
        "rule",
        "family",
        "file",
        "surfaces",
        "lint() (SFC)",
        "lint_jsx()",
        "croquis",
        "classification",
      ],
      ["left", "left", "left", "left", "left", "left", "left", "left"],
      rows.map((r) => [
        `\`${r.name}\``,
        r.family,
        `\`${r.file}\``,
        r.surfaces.join(", "),
        sfcCell(r),
        jsxCell(r),
        croquisCell(r),
        `${r.classification}${r.overrideReason !== null ? " \\*" : ""}`,
      ]),
    ).trimEnd(),
  );
  lines.push("");

  lines.push("## Overrides applied");
  lines.push("");
  if (overriddenRows.length === 0) {
    lines.push(`None. Hand-corrections go in \`${OVERRIDES_REL}\`, never into this file.`);
  } else {
    lines.push(
      formatTable(
        ["rule", "classification", "reason"],
        ["left", "left", "left"],
        overriddenRows.map((r) => [`\`${r.name}\``, r.classification, r.overrideReason]),
      ).trimEnd(),
    );
  }
  lines.push("");

  lines.push("## Cross-checks");
  lines.push("");
  if (unregistered.length > 0) {
    lines.push(
      "- Rules defined but registered on no dispatch path (dead or host-only" +
        " until wired): " +
        unregistered.map((r) => `\`${r.name}\``).join(", "),
    );
  } else {
    lines.push("- Every non-musea rule is registered on at least one dispatch path.");
  }
  const engineSetGaps = [];
  for (const [setName, set] of [
    ["SEMANTIC_TEMPLATE_RULES", model.semanticTemplateRules],
    ["SHARED_SFC_DESCRIPTOR_RULES", model.sharedSfcDescriptorRules],
    ["TYPE_AWARE_RULES", model.typeAwareRules],
  ]) {
    for (const name of set) {
      const r = rules.get(name);
      if (!r) engineSetGaps.push(`\`${setName}\` names unknown rule \`${name}\``);
      else if (!r.registered) {
        engineSetGaps.push(`\`${setName}\` names \`${name}\`, which no preset registers`);
      }
    }
  }
  if (engineSetGaps.length > 0) {
    lines.push(
      "- Engine rule-name sets referencing rules outside the registered set" +
        " (gate entries that can never activate): " +
        engineSetGaps.join("; "),
    );
  } else {
    lines.push("- Every engine rule-name set entry resolves to a registered rule.");
  }
  const semanticNoCroquis = model.semanticTemplateRules.filter((name) => {
    const r = rules.get(name);
    return !r || (r.croquis.size === 0 && r.ctxSites === 0);
  });
  lines.push(
    "- `SEMANTIC_TEMPLATE_RULES` (engine-side croquis gate, `linter/engine/rule_sets.rs`)" +
      ` lists ${model.semanticTemplateRules.length} rules; ` +
      (semanticNoCroquis.length === 0
        ? "all of them show croquis usage above."
        : "these show no croquis usage above (disagreement, not reconciled): " +
          semanticNoCroquis.map((n) => `\`${n}\``).join(", ")),
  );
  const gatedNames = new Set(model.semanticTemplateRules);
  const usersOutsideGate = croquisUsers.filter(
    (r) => r.family === "template-family" && r.ctxSites > 0 && !gatedNames.has(r.name),
  );
  lines.push(
    "- Context-lane croquis users outside that gate (their template pass runs" +
      " without analysis unless another path supplies it): " +
      (usersOutsideGate.length === 0
        ? "none."
        : usersOutsideGate.map((r) => `\`${r.name}\``).join(", ")),
  );
  lines.push(
    `- Script registry: ${matrix.scriptRegistry.registered.size} dispatch entries vs` +
      ` ${matrix.scriptRegistry.allCount} names in \`ALL_BUILTIN_SCRIPT_RULE_NAMES\`` +
      (matrix.scriptRegistry.registered.size === matrix.scriptRegistry.allCount
        ? " (agree)."
        : " (**disagree** — investigate)."),
  );
  lines.push("");
  return lines.join("\n");
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

function generate() {
  return renderArtifact(buildMatrix());
}

function main() {
  const mode = process.argv[2];
  if (mode !== "--write" && mode !== "--check") {
    console.error("usage: node tools/davinci/rule-parity.mjs --write | --check");
    process.exit(2);
  }
  const generated = generate();
  if (mode === "--write") {
    writeFileSync(ARTIFACT, generated);
    console.log(`wrote ${ARTIFACT_REL}`);
    return;
  }
  if (!existsSync(ARTIFACT)) {
    console.error(`stale: ${ARTIFACT_REL} does not exist. Regenerate with: ${REGEN_COMMAND}`);
    process.exit(1);
  }
  const committed = readFileSync(ARTIFACT, "utf8");
  if (committed === generated) {
    console.log(`${ARTIFACT_REL} is up to date`);
    return;
  }
  const committedLines = committed.split("\n");
  const generatedLines = generated.split("\n");
  let firstDiff = -1;
  const max = Math.max(committedLines.length, generatedLines.length);
  for (let i = 0; i < max; i++) {
    if (committedLines[i] !== generatedLines[i]) {
      firstDiff = i;
      break;
    }
  }
  const committedSet = new Set(committedLines);
  const generatedSet = new Set(generatedLines);
  const removed = committedLines.filter((l) => !generatedSet.has(l)).length;
  const added = generatedLines.filter((l) => !committedSet.has(l)).length;
  console.error(`stale: ${ARTIFACT_REL} drifted from the current sources.`);
  console.error(
    `  first differing line: ${firstDiff + 1} (committed ${committedLines.length} lines, regenerated ${generatedLines.length})`,
  );
  if (firstDiff >= 0) {
    console.error(`  - ${(committedLines[firstDiff] ?? "<eof>").slice(0, 160)}`);
    console.error(`  + ${(generatedLines[firstDiff] ?? "<eof>").slice(0, 160)}`);
  }
  console.error(`  lines only in committed: ${removed}, only in regenerated: ${added}`);
  console.error(`  Regenerate with: ${REGEN_COMMAND}`);
  process.exit(1);
}

main();
