import { createRequire } from "node:module";
import path from "node:path";

import type {
  PatinaLintResult,
  PatinaRuleMeta,
  PatinaRuleOptions,
  PatinaSettings,
} from "./model.js";
import { loadBinding } from "./native.js";
import { getActivePreset } from "./settings.js";

const VUE_ROUTER_COMPONENTS = ["RouterLink", "RouterView"] as const;
const COMPONENT_REGISTRATION_RULE = "vue/require-component-registration";

export function lintPatina(
  source: string,
  filename: string,
  settings: PatinaSettings,
  enabledRules?: readonly string[],
  ruleOptions?: PatinaRuleOptions,
): PatinaLintResult {
  const resolvedRuleOptions = withVueRouterComponentGlobals(filename, ruleOptions);
  const { ruleOptions: cliRuleOptions, ...directRuleOptions } = resolvedRuleOptions ?? {};
  return loadBinding().lintPatinaSfc(source, {
    filename,
    locale: settings.locale,
    helpLevel: settings.helpLevel,
    preset: getActivePreset(settings),
    enabledRules: enabledRules ? [...enabledRules] : undefined,
    typeAware: settings.typeAware,
    corsaPath: settings.corsaPath,
    ruleOptionsJson: cliRuleOptions ? JSON.stringify(cliRuleOptions) : undefined,
    ...directRuleOptions,
  });
}

function withVueRouterComponentGlobals(
  filename: string,
  options: PatinaRuleOptions | undefined,
): PatinaRuleOptions | undefined {
  if (!isVueRouterResolvableFrom(filename)) return options;

  const ruleOptions = options?.ruleOptions ?? {};
  const registration = ruleOptions[COMPONENT_REGISTRATION_RULE] ?? {};
  return {
    ...options,
    ruleOptions: {
      ...ruleOptions,
      [COMPONENT_REGISTRATION_RULE]: {
        ...registration,
        globals: [...new Set([...(registration.globals ?? []), ...VUE_ROUTER_COMPONENTS])],
      },
    },
  };
}

function isVueRouterResolvableFrom(filename: string): boolean {
  try {
    createRequire(path.resolve(filename)).resolve("vue-router");
    return true;
  } catch {
    return false;
  }
}

export function getPatinaRules(): PatinaRuleMeta[] {
  return loadBinding().getPatinaRules();
}
