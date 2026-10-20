// Matrix assembly: joins the dispatch model, the registration surfaces, the
// per-file extraction and the overrides sidecar into one rule table. File
// accounting is reconciled here (every `.rs` file is a rule, a module
// organizer, a `*_tests.rs` companion, or a helper) and a rule trait impl
// without a META is a hard failure, so the file model cannot drift silently.
// A source-proven private SFC callback contributes to its registered parent.

import { walkRustFiles } from "./crates.ts";
import { loadDispatchModel } from "./rule-parity-dispatch.ts";
import { parseOverrides } from "./rule-parity-overrides.ts";
import { META_KINDS, OVERRIDES_REL, RULES_DIR } from "./rule-parity-paths.ts";
import { collectPrivateScriptCallbacks } from "./rule-parity-private-script.ts";
import type { PrivateScripts } from "./rule-parity-private-script.ts";
import {
  collectCssRegistryNames,
  collectRegisteredRuleTypes,
  collectScriptRegistryNames,
} from "./rule-parity-registry.ts";
import { DIALECT_NAME_PREFIXES, parseRuleFile } from "./rule-parity-rules.ts";
import type { ParsedRuleFile, RuleSignals } from "./rule-parity-rules.ts";
import { applyTiers } from "./rule-parity-tiers.ts";

export interface MatrixRule {
  name: string;
  family: string | undefined;
  file: string;
  surfaces: string[];
  sfc: string;
  sfcDetail: string;
  jsx: string;
  jsxLane: string;
  croquis: Map<string, number>;
  ctxSites: number;
  registered: boolean;
  classification: string | null;
  signals: RuleSignals;
  overridden: boolean;
  overrideReason: string | null;
  tier?: string;
}

export type Matrix = ReturnType<typeof buildMatrix>;

export function buildMatrix() {
  const model = loadDispatchModel();
  const registeredTypes = collectRegisteredRuleTypes();
  const scriptRegistry = collectScriptRegistryNames();
  const cssRegistry = collectCssRegistryNames();
  const overrides = parseOverrides();

  const files = walkRustFiles(RULES_DIR).map((abs) => parseRuleFile(abs, model));
  const { privateScripts, ruleFiles, nonRuleFiles } = accountRuleFiles(files, registeredTypes);

  const rules = new Map<string, MatrixRule>();
  for (const f of ruleFiles) {
    const meta = f.meta!;
    if (rules.has(meta.name)) {
      throw new Error(
        `duplicate rule name ${meta.name} (${rules.get(meta.name)!.file} and ${f.rel})`,
      );
    }
    const family = META_KINDS.get(meta.kind);
    const rule: MatrixRule = {
      name: meta.name,
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
      const callback = privateScripts.byParent.get(f.rel)?.callback;
      const templateHookSet = [...f.hooks].filter((h) => h !== "run_on_sfc");
      const corsa = model.typeAwareRules.includes(meta.name);
      if (templateHookSet.length > 0) rule.surfaces.push("template-ast");
      if (f.hooks.has("run_on_sfc")) rule.surfaces.push("sfc-source");
      if (f.impls.some((b) => b.trait === "MarkupRule")) rule.surfaces.push("markup-facade");
      if (corsa) rule.surfaces.push("type-aware-corsa");
      if (callback) {
        rule.surfaces.push(callback.scriptUsesAst ? "script-oxc" : "script-source");
        f.signals.container.add("hook:as_sfc_script_rule");
      }
      if (rule.surfaces.length === 0) rule.surfaces.push("none");
      rule.registered = f.ruleImplTypes.some((t) => registeredTypes.has(t));
      if (rule.registered) {
        rule.sfc = "yes";
        const mechanisms: string[] = [];
        if (templateHookSet.length > 0) mechanisms.push("template-visitor");
        if (f.hooks.has("run_on_sfc")) mechanisms.push("sfc-hooks");
        if (corsa) mechanisms.push("corsa");
        if (callback) mechanisms.push("script-blocks");
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
      rule.registered = scriptRegistry.registered.has(meta.name);
      if (rule.registered) {
        rule.sfc = "yes";
        rule.sfcDetail = "script-blocks";
        rule.jsx = "yes";
        rule.jsxLane = "script";
      }
    } else if (family === "css") {
      rule.surfaces.push("css-text");
      rule.registered = cssRegistry.has(meta.name);
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
      model.sharedSfcDescriptorRules.includes(meta.name);
    if (family === "musea") f.signals.container.add("musea Art-file blocks");
    if (model.sharedSfcDescriptorRules.includes(meta.name)) {
      f.signals.container.add("SHARED_SFC_DESCRIPTOR_RULES");
    }
    const prefix = meta.name.split("/")[0];
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
  applyTiers(rules);

  return {
    model,
    files,
    ruleFiles,
    nonRuleFiles,
    rules,
    scriptRegistry,
    cssRegistry,
    overrides,
    privateScripts,
  };
}

export function accountRuleFiles(
  files: readonly ParsedRuleFile[],
  registeredTypes: ReadonlySet<string>,
): {
  privateScripts: PrivateScripts;
  ruleFiles: ParsedRuleFile[];
  nonRuleFiles: ParsedRuleFile[];
} {
  const privateScripts = collectPrivateScriptCallbacks(files, registeredTypes);
  // A META declares one rule identity; a proven private callback adds no rule.
  const ruleFiles = files.filter((f) => f.meta !== null);
  const nonRuleFiles = files.filter((f) => f.meta === null);
  for (const f of nonRuleFiles) {
    if (f.impls.some((b) => b.trait !== "MarkupRule") && !privateScripts.byCallback.has(f.rel)) {
      throw new Error(`rule trait impl without META in ${f.rel}; the file model is stale`);
    }
    const stem = f.rel.replace(/\.rs$/, "");
    if (f.rel.endsWith("_tests.rs")) f.kind = "test";
    else if (files.some((g) => g.rel.startsWith(stem + "/"))) f.kind = "module";
    else f.kind = "helper";
  }
  return { privateScripts, ruleFiles, nonRuleFiles };
}
