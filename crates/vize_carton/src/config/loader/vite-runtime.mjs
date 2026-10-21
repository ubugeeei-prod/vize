// This dependency-free projection is also embedded by the native config loader.
import { resolve } from "node:path";

// Generated validation subset of npm/cli/schemas/vize.config.schema.json.
// The canonical Vue options reuse every existing native section except retired config inheritance.
// The source law compares every retained schema rule with the authoritative native schema.
const vueOptionSchema = JSON.parse(
  '{"type":"object","properties":{"name":{"type":"string"},"compiler":{"$ref":"#/definitions/CompilerConfig"},"experimentals":{"type":"object","additionalProperties":{"oneOf":[{"type":"boolean"},{"type":"object","additionalProperties":true},{"type":"null"}]}},"dialect":{"type":"string","enum":["vue","petite-vue"]},"vite":{"$ref":"#/definitions/VitePluginConfig"},"musea":{"$ref":"#/definitions/MuseaConfig"},"typeChecker":{"$ref":"#/definitions/TypeCheckerConfig"},"linter":{"$ref":"#/definitions/LinterConfig"},"formatter":{"$ref":"#/definitions/FormatterConfig"},"languageServer":{"$ref":"#/definitions/LanguageServerConfig"},"globalTypes":{"$ref":"#/definitions/GlobalTypesConfig"},"files":{"type":"array","items":{"type":"string"}},"ignores":{"type":"array","items":{"type":"string"}},"basePath":{"type":"string"},"lib":{"$ref":"#/definitions/LibConfig"},"compile":{"$ref":"#/definitions/CompilerConfig"},"typeCheck":{"$ref":"#/definitions/TypeCheckerConfig"},"fmt":{"$ref":"#/definitions/FormatterConfig"},"lint":{"type":"object","required":["jsPlugins","plugins","rules","settings"],"properties":{"jsPlugins":{"type":"array","items":{"type":"string"}},"plugins":{"type":"array","items":{"type":"string"}},"rules":{"type":"object","additionalProperties":true},"settings":{"type":"object","additionalProperties":true},"ignorePatterns":{"type":"array","items":{"type":"string"}},"overrides":{"type":"array"}},"additionalProperties":true},"version":{"$ref":"#/definitions/VueVersion"},"entries":{"type":"array","items":{"type":"object","properties":{"name":{"type":"string"},"basePath":{"type":"string"},"files":{"type":"array","items":{"type":"string"}},"ignores":{"type":"array","items":{"type":"string"}},"compiler":{"$ref":"#/definitions/CompilerConfig"},"linter":{"$ref":"#/definitions/LinterConfig"}},"additionalProperties":false}}},"additionalProperties":false,"definitions":{"CompilerConfig":{"type":"object","properties":{"mode":{"type":"string","enum":["module","function"]},"vapor":{"type":"boolean"},"jsxMode":{"type":"string","enum":["vdom","vapor"]},"jsxCompat":{"type":"string","enum":["native","babel"]},"customRenderer":{"type":"boolean"},"customElements":{"type":"array","items":{"type":"string"}},"ssr":{"type":"boolean"},"templateSyntax":{"type":"string","enum":["standard","strict","quirks"]},"whitespace":{"type":"string","enum":["condense","preserve","vue2-line-breaks","vue2-preserve"]},"sourceMap":{"type":"boolean"},"prefixIdentifiers":{"type":"boolean"},"hoistStatic":{"type":"boolean"},"cacheHandlers":{"type":"boolean"},"isTs":{"type":"boolean"},"scriptExt":{"type":"string","enum":["ts","js"]},"runtimeModuleName":{"type":"string"},"runtimeGlobalName":{"type":"string"},"compatibility":{"$ref":"#/definitions/CompilerCompatibilityConfig"}},"additionalProperties":false},"CompilerCompatibilityConfig":{"type":"object","properties":{"vueVersion":{"$ref":"#/definitions/VueVersion"},"hostCompiler":{"type":"boolean"},"scriptSetupInStandalone":{"type":"boolean"},"optionsApiVapor":{"type":"boolean"},"nuxtVersion":{"type":"integer","enum":[2,3,4]},"webpackVersion":{"type":"integer","enum":[4,5]}},"additionalProperties":false},"VueVersion":{"type":"string","enum":["3","2.7","2","1","0.11","0.10"]},"VitePluginConfig":{"type":"object","properties":{"include":{"oneOf":[{"type":"string"},{"type":"array","items":{"type":"string"}}]},"exclude":{"oneOf":[{"type":"string"},{"type":"array","items":{"type":"string"}}]},"scanPatterns":{"type":"array","items":{"type":"string"}},"ignorePatterns":{"type":"array","items":{"type":"string"}}},"additionalProperties":false},"MuseaConfig":{"type":"object","properties":{"include":{"type":"array","items":{"type":"string"}},"exclude":{"type":"array","items":{"type":"string"}},"basePath":{"type":"string"},"storybookCompat":{"type":"boolean"},"inlineArt":{"type":"boolean"},"vrt":{"$ref":"#/definitions/MuseaVrtConfig"},"a11y":{"$ref":"#/definitions/MuseaA11yConfig"},"autogen":{"$ref":"#/definitions/MuseaAutogenConfig"}},"additionalProperties":false},"MuseaVrtConfig":{"type":"object","properties":{"threshold":{"type":"number"},"outDir":{"type":"string"},"viewports":{"type":"array","items":{"$ref":"#/definitions/MuseaViewport"}}},"additionalProperties":false},"MuseaViewport":{"type":"object","properties":{"width":{"type":"integer"},"height":{"type":"integer"},"name":{"type":"string"}},"additionalProperties":false,"required":["width","height"]},"MuseaA11yConfig":{"type":"object","properties":{"enabled":{"type":"boolean"},"rules":{"type":"object","additionalProperties":{"type":"boolean"}}},"additionalProperties":false},"MuseaAutogenConfig":{"type":"object","properties":{"enabled":{"type":"boolean"},"maxVariants":{"type":"integer"}},"additionalProperties":false},"TypeCheckerConfig":{"type":"object","properties":{"enabled":{"type":"boolean"},"strict":{"type":"boolean"},"checkProps":{"type":"boolean"},"checkEmits":{"type":"boolean"},"checkTemplateBindings":{"type":"boolean"},"checkReactivity":{"type":"boolean"},"checkSetupContext":{"type":"boolean"},"checkInvalidExports":{"type":"boolean"},"checkFallthroughAttrs":{"type":"boolean"},"optionsApi":{"type":"boolean"},"legacyVue2":{"type":"boolean"},"jsxTypecheck":{"type":"boolean"},"tsconfig":{"type":"string"},"corsaPath":{"type":"string"},"tsgoPath":{"type":"string"},"tsrsPath":{"type":"string"},"globalsFile":{"type":"string"},"servers":{"type":"integer","minimum":1,"maximum":32},"lspRequestTimeoutMs":{"type":"integer","minimum":1}},"additionalProperties":false},"LinterConfig":{"type":"object","properties":{"enabled":{"type":"boolean"},"preset":{"type":"string","enum":["general-recommended","general_recommended","generalRecommended","happy-path","happy_path","happy","default","recommended","opinionated","strict","all","essential","incremental","ecosystem","nuxt"]},"typeAware":{"type":"boolean"},"crossFile":{"type":"boolean"},"crossFileTree":{"type":"boolean"},"crossFileComplexity":{"type":"boolean"},"strictReactivity":{"type":"boolean"},"maxWarnings":{"type":"integer","minimum":0},"rules":{"type":"object","additionalProperties":{"type":"string","enum":["off","warn","error"]}},"ruleOptions":{"$ref":"#/definitions/LintRuleOptions"},"categories":{"type":"object","properties":{"correctness":{"type":"string","enum":["off","warn","error"]},"suspicious":{"type":"string","enum":["off","warn","error"]},"style":{"type":"string","enum":["off","warn","error"]},"perf":{"type":"string","enum":["off","warn","error"]},"a11y":{"type":"string","enum":["off","warn","error"]},"security":{"type":"string","enum":["off","warn","error"]}},"additionalProperties":false}},"additionalProperties":false},"LintRuleOptions":{"type":"object","properties":{"script/define-macros-order":{"type":"object","properties":{"order":{"type":"array","items":{"type":"string","minLength":1},"uniqueItems":true}},"additionalProperties":false},"script/define-props-destructuring":{"type":"object","properties":{"destructure":{"type":"string","enum":["only-when-assigned","always","never"]}},"additionalProperties":false},"html/no-empty-palpable-content":{"type":"object","properties":{"contentDirectives":{"type":"array","items":{"type":"string"}}},"additionalProperties":false},"type/strict-boolean-expressions":{"type":"object","properties":{"allowString":{"type":"boolean"},"allowNumber":{"type":"boolean"},"allowNullableObject":{"type":"boolean"},"allowNullableBoolean":{"type":"boolean"},"allowNullableString":{"type":"boolean"},"allowNullableNumber":{"type":"boolean"},"allowNullableEnum":{"type":"boolean"},"allowAny":{"type":"boolean"}},"additionalProperties":false},"script/no-restricted-globals":{"$ref":"#/definitions/NoRestrictedGlobalsOptions"},"script/no-restricted-members":{"$ref":"#/definitions/NoRestrictedMembersOptions"},"vue/component-name-in-template-casing":{"type":"object","properties":{"casing":{"type":"string","enum":["PascalCase","kebab-case"]},"registeredComponentsOnly":{"type":"boolean"},"globals":{"type":"array","items":{"type":"string"}}},"additionalProperties":false},"script/custom-event-name-casing":{"type":"object","properties":{"casing":{"type":"string","enum":["camelCase","kebab-case"]}},"additionalProperties":false},"vue/no-mutating-props":{"type":"object","properties":{"shallowOnly":{"type":"boolean"}},"additionalProperties":false},"vue/require-component-registration":{"type":"object","properties":{"globals":{"type":"array","items":{"type":"string"}}},"additionalProperties":false},"vue/sfc-element-order":{"type":"object","properties":{"order":{"type":"array","items":{"oneOf":[{"type":"string"},{"type":"array","items":{"type":"string"}}]}}},"additionalProperties":false},"vue/html-self-closing":{"type":"object","properties":{"html":{"type":"object","properties":{"void":{"type":"string","enum":["always","never","any"]},"normal":{"type":"string","enum":["always","never","any"]},"component":{"type":"string","enum":["always","never","any"]}},"additionalProperties":false},"svg":{"type":"string","enum":["always","never","any"]},"math":{"type":"string","enum":["always","never","any"]}},"additionalProperties":false},"vue/v-on-event-hyphenation":{"type":"string","enum":["always","never"]},"vue/v-bind-style":{"type":"string","enum":["shorthand","longform"]},"vue/v-on-style":{"type":"string","enum":["shorthand","longform"]},"vue/attribute-hyphenation":{"type":"string","enum":["always","never"]},"vue/valid-v-on":{"type":"object","properties":{"modifiers":{"type":"array","items":{"type":"string"}}},"additionalProperties":false},"vue/valid-v-slot":{"type":"object","properties":{"allowModifiers":{"type":"boolean"}},"additionalProperties":false},"musea/prefer-design-tokens":{"$ref":"#/definitions/MuseaPreferDesignTokensOptions"}},"additionalProperties":false},"NoRestrictedGlobalsOptions":{"type":"object","properties":{"globals":{"type":"array","items":{"$ref":"#/definitions/RestrictedGlobal"}}},"additionalProperties":false},"RestrictedGlobal":{"type":"object","properties":{"name":{"type":"string"},"message":{"type":"string"}},"additionalProperties":false,"required":["name"]},"NoRestrictedMembersOptions":{"type":"object","properties":{"members":{"type":"array","items":{"$ref":"#/definitions/RestrictedMember"}}},"additionalProperties":false},"RestrictedMember":{"type":"object","properties":{"object":{"type":"string"},"property":{"type":"string"},"message":{"type":"string"}},"additionalProperties":false,"required":["object","property"]},"MuseaPreferDesignTokensOptions":{"type":"object","properties":{"tokens":{"type":"array","items":{"$ref":"#/definitions/MuseaDesignToken"}}},"additionalProperties":false},"MuseaDesignToken":{"type":"object","properties":{"path":{"type":"string"},"value":{"type":"string"},"tier":{"type":"string"}},"additionalProperties":false,"required":["path","value"]},"FormatterConfig":{"type":"object","properties":{"preset":{"type":"string","enum":["vize","prettier","compact"]},"sortImports":{"oneOf":[{"type":"boolean","enum":[false]},{"$ref":"#/definitions/SortImportsConfig"}]},"printWidth":{"type":"integer"},"tabWidth":{"type":"integer"},"useTabs":{"type":"boolean"},"semi":{"type":"boolean"},"singleQuote":{"type":"boolean"},"jsxSingleQuote":{"type":"boolean"},"trailingComma":{"type":"string","enum":["all","none","es5"]},"bracketSpacing":{"type":"boolean"},"bracketSameLine":{"type":"boolean"},"arrowParens":{"type":"string","enum":["always","avoid"]},"endOfLine":{"type":"string","enum":["lf","crlf","cr","auto"]},"quoteProps":{"type":"string","enum":["as-needed","consistent","preserve"]},"singleAttributePerLine":{"type":"boolean"},"vueIndentScriptAndStyle":{"type":"boolean"},"sortAttributes":{"type":"boolean"},"attributeSortOrder":{"type":"string","enum":["alphabetical","as-written"]},"mergeBindAndNonBindAttrs":{"type":"boolean"},"maxAttributesPerLine":{"type":"integer"},"attributeGroups":{"type":"array","items":{"type":"array","items":{"type":"string"}}},"normalizeDirectiveShorthands":{"type":"boolean"},"sortBlocks":{"type":"boolean"},"normalizeCssQuotes":{"type":"boolean"},"normalizeInlineStyles":{"type":"boolean"},"normalizeVForParens":{"type":"boolean"},"selfCloseVoidElements":{"type":"boolean"},"reindentPreprocessorStyles":{"type":"boolean"}},"additionalProperties":false},"SortImportsConfig":{"type":"object","properties":{"partitionByNewline":{"type":"boolean"},"partitionByComment":{"type":"boolean"},"sortSideEffects":{"type":"boolean"},"order":{"type":"string","enum":["asc","desc"]},"ignoreCase":{"type":"boolean"},"newlinesBetween":{"type":"boolean"},"internalPattern":{"type":"array","items":{"type":"string"}},"groups":{"type":"array","items":{"oneOf":[{"type":"string"},{"type":"array","items":{"type":"string"}},{"type":"object","properties":{"newlinesBetween":{"type":"boolean"}},"additionalProperties":false,"required":["newlinesBetween"]}]}},"customGroups":{"type":"array","items":{"$ref":"#/definitions/ImportSortCustomGroup"}}},"additionalProperties":false},"ImportSortCustomGroup":{"type":"object","properties":{"groupName":{"type":"string"},"elementNamePattern":{"type":"array","items":{"type":"string"}},"selector":{"type":"string"},"modifiers":{"type":"array","items":{"type":"string"}}},"additionalProperties":false,"required":["groupName"]},"LanguageServerConfig":{"type":"object","properties":{"enabled":{"type":"boolean"},"lint":{"type":"boolean"},"diagnostics":{"type":"boolean"},"typecheck":{"type":"boolean"},"editor":{"type":"boolean"},"ecosystem":{"type":"boolean"},"crossFile":{"type":"boolean"},"legacyVue2":{"type":"boolean"},"completion":{"type":"boolean"},"signatureHelp":{"type":"boolean"},"hover":{"type":"boolean"},"definition":{"type":"boolean"},"references":{"type":"boolean"},"documentSymbols":{"type":"boolean"},"workspaceSymbols":{"type":"boolean"},"formatting":{"type":"boolean"},"codeActions":{"type":"boolean"},"rename":{"type":"boolean"},"codeLens":{"type":"boolean"},"semanticTokens":{"type":"boolean"},"documentLinks":{"type":"boolean"},"foldingRanges":{"type":"boolean"},"inlayHints":{"type":"boolean"},"fileRename":{"type":"boolean"},"corsa":{"type":"boolean"},"tsgo":{"type":"boolean"}},"additionalProperties":false},"GlobalTypesConfig":{"type":"object","additionalProperties":{"oneOf":[{"type":"string"},{"$ref":"#/definitions/GlobalTypeDeclaration"}]}},"GlobalTypeDeclaration":{"type":"object","properties":{"type":{"type":"string"},"defaultValue":{"type":"string"}},"additionalProperties":false,"required":["type"]},"LibConfig":{"type":"object","properties":{"dir":{"type":"string"},"uiDir":{"type":"string"},"composableDir":{"type":"string"},"lockfile":{"type":"string"},"registries":{"type":"object","additionalProperties":{"oneOf":[{"type":"string"},{"type":"object","properties":{"source":{"type":"string"},"dir":{"type":"string"}},"additionalProperties":false,"required":["source"]}]},"propertyNames":{"pattern":"^@[a-z0-9][a-z0-9._-]*$"}}},"additionalProperties":false}}}',
);

// Read the source of our Vite+ factory without starting Vite, plugins or tasks.
const sourceConfigKey = Symbol.for("@vizejs/vite-plugin/vite-plus/source");

function object(value) {
  if (value === null || typeof value !== "object") return false;
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

function merge(base, own) {
  const result = { ...base };
  for (const [key, value] of Object.entries(own)) {
    result[key] = object(value) && object(result[key]) ? merge(result[key], value) : value;
  }
  return result;
}

async function resolveSource(value, env, parents = new Set()) {
  value = await value;
  if (parents.has(value)) throw new Error("Circular Vue configuration source");
  parents.add(value);
  try {
    if ((typeof value === "function" || object(value)) && sourceConfigKey in value) {
      return await resolveSource(value[sourceConfigKey], env, parents);
    }
    const config = await (typeof value === "function" ? value(env) : value);
    if (!object(config)) throw new Error("Vite config must export an object or a config function");
    rejectLegacyConfig(config);
    return config;
  } finally {
    parents.delete(value);
  }
}

function rejectLegacyConfig(source) {
  if (Object.hasOwn(source, "extends")) {
    throw new Error(
      "Custom config extends is no longer supported; use defineConfig(vpConfig).withVue(vueConfig)",
    );
  }
  if (Object.hasOwn(source, "vize")) {
    throw new Error(
      "vize: configuration is no longer supported; move Vue settings to vue: in vite.config.*",
    );
  }
  for (const key of ["lint", "fmt", "pack"]) {
    const values = Array.isArray(source[key]) ? source[key] : [source[key]];
    if (values.some((value) => object(value) && Object.hasOwn(value, "vize"))) {
      throw new Error(
        `${key}.vize is no longer supported; use ${key}.vue or the shared vue: settings`,
      );
    }
  }
}

export const VITE_CONFIG_FILE_NAMES = [
  "vite.config.ts",
  "vite.config.js",
  "vite.config.mjs",
  "vite.config.cjs",
  "vite.config.mts",
  "vite.config.cts",
];

export function isViteConfigFile(filePath) {
  return /(?:^|[/\\])vite\.config\.(?:[cm]?[jt]s)$/.test(filePath);
}

/** Project workflows support JSX without an extra feature flag. Legacy config stays unchanged. */
export function projectConfigDefaults() {
  return { typeChecker: { jsxTypecheck: true } };
}

/** Resolve just the native settings from an ordinary Vite or Vite+ export. */
export async function resolveViteConfigExport(exported, env, configDir) {
  env ??= { mode: "development", command: "serve" };
  // Dedicated Vize workflows also use commands such as lint and check. A Vite
  // function receives only Vite's serve/build discriminator, without mutation.
  if (env.command !== "build" && env.command !== "serve") {
    env = { ...env, command: "serve" };
  }
  const source = await resolveViteConfigSource(exported, env);
  return projectViteConfigSource(source, env, configDir);
}

/** Resolve an authored Vite export without projecting away Vite-owned task settings. */
export async function resolveViteConfigSource(exported, env) {
  env ??= { mode: "development", command: "serve" };
  if (env.command !== "build" && env.command !== "serve") {
    env = { ...env, command: "serve" };
  }
  return resolveSource(exported, env);
}

/** Resolve an explicitly selected CLI config as Vite config independent of its basename. */
export async function resolveExplicitViteConfigExport(exported, env, configDir) {
  env ??= { mode: "development", command: "serve" };
  if (env.command !== "build" && env.command !== "serve") {
    env = { ...env, command: "serve" };
  }
  const source = await resolveViteConfigSource(exported, env);
  return projectExplicitViteConfigSource(source, env, configDir);
}

/** Preserve an archived native export only when its shape proves that role. */
export async function resolveExplicitSerializedConfigExport(exported, env, configDir) {
  env ??= { mode: "development", command: "serve" };
  if (env.command !== "build" && env.command !== "serve") {
    env = { ...env, command: "serve" };
  }
  const source = await resolveViteConfigSource(exported, env);
  if (
    [
      "compiler",
      "typeChecker",
      "linter",
      "formatter",
      "languageServer",
      "globalTypes",
      "files",
      "ignores",
      "entries",
    ].some((key) => Object.hasOwn(source, key))
  ) {
    return source;
  }
  return projectExplicitViteConfigSource(source, env, configDir);
}

function projectExplicitViteConfigSource(source, env, configDir) {
  if (
    ["linter", "formatter", "languageServer", "globalTypes", "files", "ignores", "entries"].some(
      (key) => Object.hasOwn(source, key),
    )
  ) {
    throw new Error(
      "vize.config.* is no longer supported; move Vue settings to vue: in vite.config.* and TypeScript settings to tsconfig.json",
    );
  }
  return projectViteConfigSource(source, env, configDir);
}

/** Project already evaluated settings; the owning adapter evaluates its Vite export once. */
export async function projectViteConfigSource(source, env, configDir) {
  if (!object(source)) throw new TypeError("Vite config source must be an object");
  env ??= { mode: "development", command: "serve" };
  // Adapters may pass a native task environment after evaluating the Vite source.
  // Its legacy native callback shares the same idempotent Vite command mapping.
  if (env.command !== "build" && env.command !== "serve") env = { ...env, command: "serve" };
  rejectLegacyConfig(source);
  const selected = normalizeVueConfigOptions(source);
  const canonical = selected === undefined ? {} : flatVueOptions(selected);
  const overrides = {};
  if (object(source.compiler)) {
    const { vueVersion, ...compiler } = source.compiler;
    const version = vueVersion ?? compiler.compatibility?.vueVersion;
    overrides.compiler = merge(
      compiler,
      version === undefined
        ? {}
        : {
            // Numeric 0.1 is the newly public spelling of Vue 0.10.
            compatibility: {
              vueVersion: String(version === 0.1 ? "0.10" : version === "legacy" ? 2 : version),
            },
          },
    );
  }
  if (typeof source.typecheck === "boolean") overrides.typeChecker = { enabled: source.typecheck };
  else if (object(source.typecheck)) overrides.typeChecker = source.typecheck;
  const lint = source.lint?.vue;
  if (typeof lint === "boolean") overrides.linter = { enabled: lint };
  else if (object(lint)) {
    const { typecheck: _typecheck, locale: _locale, helpLevel: _helpLevel, ...native } = lint;
    overrides.linter = native;
  }
  if (object(source.fmt)) {
    const { vue, sortImports } = source.fmt;
    const common = sortImports === undefined ? {} : { sortImports };
    overrides.formatter = merge(common, object(vue) ? vue : {});
  }
  const root = typeof source.root === "string" ? resolve(configDir ?? ".", source.root) : configDir;
  const config = overlayVueOptions(merge(projectConfigDefaults(), overrides), canonical);
  return root ? { ...projectPaths(config, root, false), __vizeProjectRoot: root } : config;
}

/** Validate only canonical settings; selecting them never evaluates a legacy callback. */
export function vueConfigOptions(source) {
  const selected = source.vue;
  if (selected !== undefined) {
    validateVueOption(selected, vueOptionSchema, "vue");
    assertVueActionAliases(selected, "vue");
  }
  return selected;
}

/** Resolve preferred action names to the native section names without mutating authored input. */
function normalizeVueConfigOptions(source) {
  const selected = vueConfigOptions(source);
  return selected === undefined ? undefined : normalizeVueActionAliases(selected, "vue");
}

/** Share canonical composition with adapters while their other Vite fields keep Vite merging. */
export function mergeVueConfigOptions(base, own) {
  vueConfigOptions({ vue: base });
  vueConfigOptions({ vue: own });
  if (base === undefined) return own;
  if (own === undefined) return base;
  const result = overlayVueOptions(base, own);
  vueConfigOptions({ vue: result });
  return result;
}

const vueActionAliases = [
  ["compile", "compiler"],
  ["typeCheck", "typeChecker"],
  ["fmt", "formatter"],
];

function assertVueActionAliases(value, label) {
  for (const [action, compatibility] of vueActionAliases) {
    if (value[action] !== undefined && value[compatibility] !== undefined) {
      throw new TypeError(
        `${label}.${action}: conflicts with compatibility alias ${label}.${compatibility}; remove ${label}.${compatibility} and use ${label}.${action}`,
      );
    }
  }
  if (value.lint !== undefined && value.linter !== undefined) {
    throw new TypeError(
      `${label}.lint: conflicts with compatibility alias ${label}.linter; remove ${label}.linter and use ${label}.lint, with native rules under settings.vize`,
    );
  }
  for (let index = 0; index < (value.entries?.length ?? 0); index++) {
    assertVueActionAliases(value.entries[index], `${label}.entries[${index}]`);
  }
}

function normalizeVueActionAliases(value, label) {
  assertVueActionAliases(value, label);
  const result = { ...value };
  for (const [action, compatibility] of vueActionAliases) {
    if (result[action] === undefined) continue;
    result[compatibility] = result[action];
    delete result[action];
  }
  if (result.lint !== undefined) {
    const lint = result.lint;
    const linter = projectVitePlusLintConfig(lint);
    validateVueOption(linter, { $ref: "#/definitions/LinterConfig" }, `${label}.lint`);
    result.linter = linter;
    delete result.lint;
    const projectedEntries = projectVitePlusLintOverrides(lint, `${label}.lint`);
    if (projectedEntries.length !== 0) {
      result.entries = [...projectedEntries, ...(result.entries ?? [])];
    }
  }
  if (Array.isArray(result.entries)) {
    result.entries = result.entries.map((entry, index) =>
      normalizeVueActionAliases(entry, `${label}.entries[${index}]`),
    );
  }
  return result;
}

function projectVitePlusLintConfig(lint) {
  const settings = object(lint.settings?.vize) ? lint.settings.vize : {};
  const linter = {};
  for (const key of ["preset", "typeAware"]) {
    if (settings[key] !== undefined) linter[key] = settings[key];
  }
  const rules = {};
  const ruleOptions = {};
  for (const [id, entry] of Object.entries(object(lint.rules) ? lint.rules : {})) {
    if (!id.startsWith("vize/")) continue;
    const name = id.slice("vize/".length);
    const rawSeverity = Array.isArray(entry) ? entry[0] : entry;
    const severity = rawSeverity === 1 ? "warn" : rawSeverity === 2 ? "error" : rawSeverity;
    if (severity === "off" || severity === "warn" || severity === "error") {
      rules[name] = severity;
    }
    if (Array.isArray(entry) && entry[1] !== undefined) {
      ruleOptions[name] = entry[1];
    }
  }
  if (Object.keys(rules).length !== 0) linter.rules = rules;
  if (Object.keys(ruleOptions).length !== 0) linter.ruleOptions = ruleOptions;
  return linter;
}

function projectVitePlusLintOverrides(lint, label) {
  if (!Array.isArray(lint.overrides)) return [];
  return lint.overrides.flatMap((override, index) => {
    if (!object(override)) return [];
    const linter = projectVitePlusLintConfig(override);
    if (linter.rules === undefined && linter.ruleOptions === undefined) return [];
    validateVueOption(
      linter,
      { $ref: "#/definitions/LinterConfig" },
      `${label}.overrides[${index}]`,
    );
    const files = stringArray(override.files, `${label}.overrides[${index}].files`);
    const ignores =
      override.excludeFiles === undefined
        ? undefined
        : stringArray(override.excludeFiles, `${label}.overrides[${index}].excludeFiles`);
    return [{ files, ...(ignores === undefined ? {} : { ignores }), linter }];
  });
}

function stringArray(value, label) {
  if (!Array.isArray(value) || !value.every((entry) => typeof entry === "string")) {
    throw new TypeError(`${label}: must be an array of strings to project vize/* rules`);
  }
  return [...value];
}

function flatVueOptions(selected) {
  const { version, entries, ...sections } = selected;
  return {
    ...sections,
    ...(version === undefined ? {} : { vue: { version } }),
    ...(entries === undefined ? {} : { entries: entries.map(flatVueOptions) }),
  };
}

// Native arrays replace; authored false leaves win and undefined remains absent.
function overlayVueOptions(base, own) {
  const result = { ...base };
  for (const [key, value] of Object.entries(own)) {
    if (value === undefined) continue;
    const previous = Object.hasOwn(result, key) ? result[key] : undefined;
    Object.defineProperty(result, key, {
      value: object(value) && object(previous) ? overlayVueOptions(previous, value) : value,
      enumerable: true,
      configurable: true,
      writable: true,
    });
  }
  return result;
}

function validateVueOption(value, schema, label) {
  const refuse = (reason) => {
    throw new TypeError(`${label}: ${reason}`);
  };
  if (schema.$ref) {
    return validateVueOption(
      value,
      vueOptionSchema.definitions[schema.$ref.split("/").at(-1)],
      label,
    );
  }
  if (schema.oneOf) {
    const matches = schema.oneOf.filter((candidate) => {
      try {
        validateVueOption(value, candidate, label);
        return true;
      } catch (error) {
        if (!(error instanceof TypeError)) throw error;
        return false;
      }
    });
    if (matches.length !== 1) refuse("must match one documented native option type");
    return;
  }
  if (schema.enum && !schema.enum.includes(value)) refuse("unsupported native option value");
  const valid = {
    object: object(value),
    array: Array.isArray(value),
    string: typeof value === "string",
    boolean: typeof value === "boolean",
    integer: Number.isInteger(value),
    number: typeof value === "number" && Number.isFinite(value),
    null: value === null,
  };
  if (schema.type && !valid[schema.type]) refuse(`must be ${schema.type}`);
  if (schema.minimum !== undefined && value < schema.minimum)
    refuse(`must be >= ${schema.minimum}`);
  if (schema.maximum !== undefined && value > schema.maximum)
    refuse(`must be <= ${schema.maximum}`);
  if (Array.isArray(value) && schema.items) {
    for (let index = 0; index < value.length; index++) {
      validateVueOption(value[index], schema.items, `${label}[${index}]`);
    }
  }
  if (object(value)) {
    for (const required of schema.required ?? []) {
      if (!Object.hasOwn(value, required) || value[required] === undefined)
        refuse(`missing ${required}`);
    }
    for (const [key, item] of Object.entries(value)) {
      const documented = Object.hasOwn(schema.properties ?? {}, key);
      if (!documented && schema.additionalProperties === false) refuse(`unknown option "${key}"`);
      if (schema.propertyNames?.pattern && !new RegExp(schema.propertyNames.pattern).test(key)) {
        refuse(`invalid native option name ${key}`);
      }
      const rule = documented ? schema.properties[key] : schema.additionalProperties;
      if (item !== undefined && object(rule)) validateVueOption(item, rule, `${label}.${key}`);
    }
  }
}

// Vite-owned settings use its root. Dedicated Vize files keep their established
// config-directory path contract, and CLI arguments never pass through here.
function projectPaths(config, root, entry) {
  if (!root || !object(config)) return config;
  const result = { ...config };
  if (
    typeof config.basePath === "string" ||
    config.files !== undefined ||
    (entry && config.ignores !== undefined)
  ) {
    result.basePath = resolve(root, config.basePath ?? ".");
  }
  if (Array.isArray(config.entries)) {
    result.entries = config.entries.map((value) => projectPaths(value, root, true));
  }
  if (object(config.typeChecker)) {
    result.typeChecker = { ...config.typeChecker };
    for (const key of ["tsconfig", "corsaPath", "tsgoPath", "tsrsPath", "globalsFile"]) {
      if (typeof config.typeChecker[key] === "string") {
        result.typeChecker[key] = resolve(root, config.typeChecker[key]);
      }
    }
  }
  return result;
}
