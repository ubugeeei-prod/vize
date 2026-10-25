// Trait hook inventories and the dispatch model, parsed from the real sources
// (rule.rs / markup/rule.rs / linter/engine.rs), never hardcoded. Every anchor this
// stage asserts is a claim the artifact makes about path membership, so drift
// in the dispatch code fails the generator instead of silently skewing counts.

import { readFileSync } from "node:fs";
import path from "node:path";

import { byKey } from "./ordering.ts";
import { PATINA_SRC } from "./rule-parity-paths.ts";
import { loadPlainTypeDispatch } from "./rule-parity-plain-types.ts";
import type { PlainTypeDispatch } from "./rule-parity-plain-types.ts";
import { matchBraceBlock, stripRustComments } from "./rule-parity-rust-text.ts";
import { stripRust } from "./rust-source.ts";

export interface DispatchModel {
  templateHooks: string[];
  markupHooks: string[];
  semanticTemplateRules: string[];
  sharedSfcDescriptorRules: string[];
  typeAwareRules: string[];
  plainTypes: PlainTypeDispatch;
}

export function traitFnNames(stripped: string, traitName: string): string[] {
  const m = new RegExp(`\\btrait\\s+${traitName}\\b`).exec(stripped);
  if (!m) throw new Error(`trait ${traitName} not found`);
  const open = stripped.indexOf("{", m.index);
  const close = matchBraceBlock(stripped, open);
  const body = stripped.slice(open + 1, close);
  const names: string[] = [];
  const fnRe = /\bfn\s+([a-z_][a-z0-9_]*)/g;
  let fm: RegExpExecArray | null;
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

/** Verify the real markup phase and its public type-completion wrapper. */
export function validateJsxDispatch(source: string): void {
  const engine = stripRust(source);
  const body = (name: string, publicOnly = false): string => {
    const pattern = new RegExp(`\\b${publicOnly ? "pub\\s+" : ""}fn\\s+${name}\\b`, "gu");
    const matches = [...engine.matchAll(pattern)];
    if (matches.length !== 1) {
      throw new Error(`dispatch model drift: expected one ${name} function`);
    }
    const open = engine.indexOf("{", matches[0].index);
    return engine.slice(open + 1, matchBraceBlock(engine, open));
  };
  const wrapper = body("lint_jsx", true);
  let markup = wrapper;
  if (/\.\s*lint_jsx_without_types\s*\(/u.test(wrapper)) {
    const delegate =
      /let\s+mut\s+result\s*=\s*self\s*\.\s*lint_jsx_without_types\s*\(\s*source\s*,\s*filename\s*,\s*lang\s*,?\s*\)/gu;
    const append =
      /super\s*::\s*native_type_aware\s*::\s*plain_module\s*::\s*append_plain_module_type_diagnostics\s*\(\s*self\s*,\s*source\s*,\s*filename\s*,\s*&\s*mut\s+result\s*,?\s*\)/gu;
    const finish = /self\s*\.\s*apply_diagnostic_policy\s*\(\s*result\s*\)/gu;
    const delegates = [...wrapper.matchAll(delegate)];
    const appends = [...wrapper.matchAll(append)];
    const finishes = [...wrapper.matchAll(finish)];
    if (
      delegates.length !== 1 ||
      appends.length !== 1 ||
      finishes.length !== 1 ||
      (wrapper.match(/\.\s*lint_jsx_without_types\s*\(/gu) ?? []).length !== 1 ||
      (wrapper.match(/\bappend_plain_module_type_diagnostics\s*\(/gu) ?? []).length !== 1 ||
      !(delegates[0].index < appends[0].index && appends[0].index < finishes[0].index)
    ) {
      throw new Error(
        "dispatch model drift: JSX wrapper must prepare, append plain types and finish exactly once in order",
      );
    }
    const remaining = wrapper.replace(append, "plain_type_completion");
    for (const forbidden of [
      "run_on_sfc",
      "css_rules",
      "native_type_aware",
      "CorsaTypeAwareSession",
      "with_corsa_session",
    ]) {
      if (remaining.includes(forbidden)) {
        throw new Error(`dispatch model drift: JSX wrapper gained unsupported ${forbidden}`);
      }
    }
    markup = body("lint_jsx_without_types");
  }
  for (const anchor of [
    "as_markup_rule",
    "jsx_needs_lowering",
    "legacy_keep_mask",
    "lint_jsx_over_ir",
    "lint_jsx_lowered_markup_root",
    "lint_jsx_fallback_root",
  ]) {
    if (!markup.includes(anchor)) {
      throw new Error(`dispatch model drift: JSX markup phase lost anchor ${anchor}`);
    }
  }
  if ((markup.match(/\.\s*lint_jsx_script\s*\(/gu) ?? []).length !== 1) {
    throw new Error("dispatch model drift: JSX markup phase must invoke script rules exactly once");
  }
  // These are markup-phase exclusions. The public wrapper deliberately invokes
  // plain-module type completion; this is not a claim that all JSX is Corsa-free.
  for (const absent of [
    "run_on_sfc",
    "css_rules",
    "native_type_aware",
    "CorsaTypeAwareSession",
    "with_corsa_session",
  ]) {
    if (markup.includes(absent)) {
      throw new Error(
        `dispatch model drift: JSX markup phase now references ${absent}; re-derive path membership`,
      );
    }
  }
}

export function loadDispatchModel(): DispatchModel {
  const ruleRs = stripRust(readFileSync(path.join(PATINA_SRC, "rule.rs"), "utf8"));
  const markupRs = stripRust(readFileSync(path.join(PATINA_SRC, "markup", "rule.rs"), "utf8"));
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

  validateJsxDispatch(engineRs);
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
  const readSet = (text: string, name: string): string[] => {
    const m2 = new RegExp(`${name}\\s*:\\s*&\\[&str\\]\\s*=\\s*&\\[`).exec(text);
    if (!m2) throw new Error(`rule set ${name} not found`);
    const end = text.indexOf("]", m2.index + m2[0].length);
    const body = text.slice(m2.index + m2[0].length, end);
    return [...body.matchAll(/"([^"]+)"/g)].map((x) => x[1]).sort(byKey);
  };

  // Rules served by the native type-aware session: the
  // TYPE_AWARE_RULES list names const idents whose string values live in the
  // same file.
  const typeAwareRs = stripRustComments(
    readFileSync(path.join(PATINA_SRC, "linter", "native_type_aware.rs"), "utf8"),
  );
  const typeAwareConsts = new Map<string, string>();
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
    plainTypes: loadPlainTypeDispatch(typeAwareConsts),
  };
}
