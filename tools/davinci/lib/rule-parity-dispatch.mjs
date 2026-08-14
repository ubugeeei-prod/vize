// Trait hook inventories and the dispatch model, parsed from the real sources
// (rule.rs / markup.rs / linter/engine.rs), never hardcoded. Every anchor this
// stage asserts is a claim the artifact makes about path membership, so drift
// in the dispatch code fails the generator instead of silently skewing counts.

import { readFileSync } from "node:fs";
import path from "node:path";

import { byKey } from "./ordering.mjs";
import { PATINA_SRC } from "./rule-parity-paths.mjs";
import { matchBraceBlock, stripRustComments } from "./rule-parity-rust-text.mjs";
import { stripRust } from "./rust-source.mjs";

export function traitFnNames(stripped, traitName) {
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

export function loadDispatchModel() {
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
    return [...body.matchAll(/"([^"]+)"/g)].map((x) => x[1]).sort(byKey);
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
    .sort(byKey);

  return {
    templateHooks,
    markupHooks,
    semanticTemplateRules: readSet(ruleSets, "SEMANTIC_TEMPLATE_RULES"),
    sharedSfcDescriptorRules: readSet(ruleSets, "SHARED_SFC_DESCRIPTOR_RULES"),
    typeAwareRules,
  };
}
