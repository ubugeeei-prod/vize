// Demand gates for Croquis products with no external consumer (P4-4a).
//
// A product stays in the summary's "no external consumers" section only when
// it names a gate here. The gate is the switch that keeps the product off
// the hot path until something demands it. `--check` fails when an orphan
// has no gate, or a gate names a product that is not an orphan.

/** @type {Record<string, readonly string[]>} */
export const GATES = {
  "summary-on-demand": ["AnalysisStats", "CroquisStats"],
  "script-parse": [
    "ImportStatementInfo",
    "InvalidExport",
    "OptionKey",
    "OptionsDescriptor",
    "ReExportForward",
    "ReExportInfo",
  ],
  "semantic-snapshot": [
    "SemanticBindingSnapshot",
    "SemanticComponentUsageSnapshot",
    "SemanticEventListenerSnapshot",
    "SemanticInjectSnapshot",
    "SemanticPassedPropSnapshot",
    "SemanticProvideSnapshot",
    "SemanticReactiveSourceSnapshot",
    "SemanticReactivityLossSnapshot",
    "SemanticScopeBindingSnapshot",
    "SemanticScopeSnapshot",
    "SemanticSlotUsageSnapshot",
    "SemanticSourceRange",
    "SemanticTemplateExpressionSnapshot",
    "UnusedTemplateVar",
  ],
  ComponentUsages: ["Croquis.component_usages", "Croquis.used_components"],
  analyze_hoisting: ["Croquis.hoists", "HoistTracker"],
  "symbol-table": ["Croquis.symbols", "Symbol", "SymbolFlags", "SymbolId", "SymbolTable"],
  UndefinedRefs: ["Croquis.undefined_refs"],
  track_usage: ["Croquis.used_directives"],
  "template-facts": ["ComponentRegistration", "ElementIdInfo", "TemplateInfo"],
  "effect-graph": ["EffectGraph"],
  "provide-inject": ["ProvideInjectTracker", "Croquis.provide_inject"],
  "race-conditions": ["RaceConditionTracker", "Croquis.race_conditions"],
  "reactivity-overlay": [
    "ReactivityEffectEdgeOverlay",
    "ReactivityEffectGraphOverlay",
    "ReactivityLossOverlay",
    "ReactivityOverlay",
    "ReactivityOverlaySummary",
    "ReactivitySourceOverlay",
  ],
  analyze_template_scopes: [
    "BindingFlags",
    "BlockKind",
    "BlockScopeData",
    "CallbackScopeData",
    "ClientOnlyScopeData",
    "ClosureScopeData",
    "ExternalModuleScopeData",
    "ImportedExport",
    "JsGlobalScopeData",
    "JsRuntime",
    "PARAM_INLINE_CAP",
    "ParamNames",
    "ParentScopes",
    "ScriptSetupScopeData",
    "Span",
    "UniversalScopeData",
    "VueGlobalScopeData",
  ],
  "setup-context": ["SetupContextTracker"],
  "script-types": ["TypeResolver"],
};

const gateByProduct = new Map();
for (const [gate, ids] of Object.entries(GATES)) {
  for (const id of ids) {
    if (gateByProduct.has(id)) {
      throw new Error(`croquis gate ${id} is listed under ${gateByProduct.get(id)} and ${gate}`);
    }
    gateByProduct.set(id, gate);
  }
}

export function gateOf(productId) {
  return gateByProduct.get(productId) ?? null;
}

export function gatedProductIds() {
  return [...gateByProduct.keys()];
}
