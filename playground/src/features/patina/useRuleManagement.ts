import { ref, computed, type Ref } from "vue";
import type { LintPreset, LintRule } from "../../wasm/index";

const STORAGE_KEY = "vize-patina-rules-config";
const DEFAULT_PRESET: LintPreset = "happy-path";
const KNOWN_PRESETS = new Set<LintPreset>(["happy-path", "opinionated", "essential", "nuxt"]);

export function useRuleManagement(rules: Ref<LintRule[]>, lint: () => void) {
  const selectedPreset = ref<LintPreset>(DEFAULT_PRESET);
  const enabledRules = ref<Set<string>>(new Set());
  const severityOverrides = ref<Map<string, "error" | "warning" | "off">>(new Map());

  // Rule filtering state
  const selectedCategory = ref<string>("all");
  const searchQuery = ref("");

  const categories = computed(() => {
    const cats = new Set(rules.value.map((r) => r.category));
    return ["all", ...Array.from(cats).sort()];
  });

  const filteredRules = computed(() => {
    return rules.value.filter((rule) => {
      const matchesCategory =
        selectedCategory.value === "all" || rule.category === selectedCategory.value;
      const matchesSearch =
        searchQuery.value === "" ||
        rule.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
        rule.description.toLowerCase().includes(searchQuery.value.toLowerCase());
      return matchesCategory && matchesSearch;
    });
  });

  function normalizePreset(value: unknown): LintPreset {
    return typeof value === "string" && KNOWN_PRESETS.has(value as LintPreset)
      ? (value as LintPreset)
      : DEFAULT_PRESET;
  }

  /** Load saved rule configuration from localStorage */
  function loadRuleConfig() {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved) {
        const config = JSON.parse(saved);
        selectedPreset.value = normalizePreset(config.selectedPreset);
        enabledRules.value = new Set(config.enabledRules || []);
        severityOverrides.value = new Map(Object.entries(config.severityOverrides || {}));
      }
    } catch (e) {
      console.warn("Failed to load rule config:", e);
    }
  }

  /** Save rule configuration to localStorage */
  function saveRuleConfig() {
    try {
      const config = {
        selectedPreset: selectedPreset.value,
        enabledRules: Array.from(enabledRules.value),
        severityOverrides: Object.fromEntries(severityOverrides.value),
      };
      localStorage.setItem(STORAGE_KEY, JSON.stringify(config));
    } catch (e) {
      console.warn("Failed to save rule config:", e);
    }
  }

  /** Initialize all rules as enabled when rules are loaded */
  function initializeRuleState() {
    if (rules.value.length === 0) {
      return;
    }

    if (enabledRules.value.size === 0) {
      applyPreset(selectedPreset.value, false);
      return;
    }

    enabledRules.value = new Set(
      Array.from(enabledRules.value).filter((name) =>
        rules.value.some((rule) => rule.name === name),
      ),
    );
    saveRuleConfig();
  }

  /** Apply a built-in rule preset */
  function applyPreset(preset: LintPreset, shouldLint = true) {
    selectedPreset.value = preset;
    enabledRules.value = new Set(
      rules.value.filter((rule) => rule.presets.includes(preset)).map((rule) => rule.name),
    );
    saveRuleConfig();
    if (shouldLint) {
      lint();
    }
  }

  /** Toggle rule enabled state */
  function toggleRule(ruleName: string) {
    if (enabledRules.value.has(ruleName)) {
      enabledRules.value.delete(ruleName);
    } else {
      enabledRules.value.add(ruleName);
    }
    saveRuleConfig();
    lint();
  }

  /** Toggle all rules in a category */
  function toggleCategory(category: string, enabled: boolean) {
    const categoryRules = rules.value.filter((r) => r.category === category);
    categoryRules.forEach((rule) => {
      if (enabled) {
        enabledRules.value.add(rule.name);
      } else {
        enabledRules.value.delete(rule.name);
      }
    });
    saveRuleConfig();
    lint();
  }

  /** Enable all rules */
  function enableAllRules() {
    selectedPreset.value = "opinionated";
    rules.value.forEach((rule) => {
      enabledRules.value.add(rule.name);
    });
    saveRuleConfig();
    lint();
  }

  /** Disable all rules */
  function disableAllRules() {
    enabledRules.value.clear();
    saveRuleConfig();
    lint();
  }

  /** Check if all rules in a category are enabled */
  function isCategoryFullyEnabled(category: string): boolean {
    const categoryRules = rules.value.filter((r) => r.category === category);
    return categoryRules.every((rule) => enabledRules.value.has(rule.name));
  }

  /** Check if some (but not all) rules in a category are enabled */
  function isCategoryPartiallyEnabled(category: string): boolean {
    const categoryRules = rules.value.filter((r) => r.category === category);
    const enabledCount = categoryRules.filter((rule) => enabledRules.value.has(rule.name)).length;
    return enabledCount > 0 && enabledCount < categoryRules.length;
  }

  return {
    selectedPreset,
    enabledRules,
    severityOverrides,
    selectedCategory,
    searchQuery,
    categories,
    filteredRules,
    loadRuleConfig,
    saveRuleConfig,
    initializeRuleState,
    applyPreset,
    toggleRule,
    toggleCategory,
    enableAllRules,
    disableAllRules,
    isCategoryFullyEnabled,
    isCategoryPartiallyEnabled,
  };
}
