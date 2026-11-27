import type {
  PatinaLintResult,
  PatinaRuleMeta,
  PatinaRuleOptions,
  PatinaSettings,
} from "./model.js";
import { loadBinding } from "./native.js";
import { ONE_SHOT_OXLINT_RUN, packageResolution } from "./run-scope.js";
import { getActivePreset } from "./settings.js";

const VUE_ROUTER_COMPONENTS = ["RouterLink", "RouterView"] as const;
const COMPONENT_REGISTRATION_RULE = "vue/require-component-registration";
const UNDEFINED_REFS_RULE = "vue/no-undefined-refs";

export function lintPatina(
  source: string,
  filename: string,
  settings: PatinaSettings,
  enabledRules?: readonly string[],
  ruleOptions?: PatinaRuleOptions,
): PatinaLintResult {
  const resolvedRuleOptions = withVueRouterComponentGlobals(filename, enabledRules, ruleOptions);
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
    // One Oxlint CLI run reads each imported type source once; every other host keeps reading fresh ones.
    runScopedTypeSources: ONE_SHOT_OXLINT_RUN,
    ...directRuleOptions,
  });
}

function withVueRouterComponentGlobals(
  filename: string,
  enabledRules: readonly string[] | undefined,
  options: PatinaRuleOptions | undefined,
): PatinaRuleOptions | undefined {
  if (enabledRules && !enabledRules.includes(COMPONENT_REGISTRATION_RULE)) return options;
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

/**
 * The host's `globals` name what a build step defines for every module, such
 * as the auto-imports `@vizejs/nuxt` lists. Templates read the same names.
 */
export function withHostGlobals(
  ruleName: string,
  options: PatinaRuleOptions | undefined,
  hostGlobals: Readonly<Record<string, unknown>> | undefined,
): PatinaRuleOptions | undefined {
  if (ruleName !== UNDEFINED_REFS_RULE || !hostGlobals) return options;
  const names = Object.keys(hostGlobals).filter((name) => hostGlobals[name] !== "off");
  if (names.length === 0) return options;

  const ruleOptions = options?.ruleOptions ?? {};
  const configured = ruleOptions[UNDEFINED_REFS_RULE]?.globals ?? [];
  return {
    ...options,
    ruleOptions: {
      ...ruleOptions,
      [UNDEFINED_REFS_RULE]: { globals: [...new Set([...configured, ...names])] },
    },
  };
}

const isVueRouterResolvableFrom = packageResolution("vue-router");

export function getPatinaRules(): PatinaRuleMeta[] {
  return loadBinding().getPatinaRules();
}
