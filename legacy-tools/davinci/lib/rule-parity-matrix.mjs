// Matrix assembly: joins the dispatch model, the registration surfaces, the
// per-file extraction and the overrides sidecar into one rule table. File
// accounting is reconciled here (every `.rs` file is a rule, a module
// organizer, a `*_tests.rs` companion, or a helper) and a rule trait impl
// without a META is a hard failure, so the file model cannot drift silently.

import { walkRustFiles } from "./crates.mjs";
import { loadDispatchModel } from "./rule-parity-dispatch.mjs";
import { parseOverrides } from "./rule-parity-overrides.mjs";
import { META_KINDS, OVERRIDES_REL, RULES_DIR } from "./rule-parity-paths.mjs";
import {
  collectCssRegistryNames,
  collectRegisteredRuleTypes,
  collectScriptRegistryNames,
} from "./rule-parity-registry.mjs";
import { DIALECT_NAME_PREFIXES, parseRuleFile } from "./rule-parity-rules.mjs";

export function buildMatrix() {
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
