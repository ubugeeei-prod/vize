import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  FEATURE_SETTING_KEYS,
  LINT_ONLY_CONFIGURATION_UPDATES,
  createDocumentSelector,
  describeCapabilities,
  getInitializationOptions,
  hasAnyEnabledCapability,
  hasAnyExplicitCapabilityValue,
  shouldStartFromConfiguration,
  type ConfigurationInspection,
  type VizeConfigurationLike,
} from "../../editors/vscode/src/extension-core.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

const FEATURE_TO_OPTION = {
  "lint.enable": "lint",
  "diagnostics.enable": "lint",
  "typecheck.enable": "typecheck",
  "editor.enable": "editor",
  "ecosystem.enable": "ecosystem",
  "optionsApi.enable": "optionsApi",
  "legacyVue2.enable": "legacyVue2",
  "completion.enable": "completion",
  "signatureHelp.enable": "signatureHelp",
  "hover.enable": "hover",
  "definition.enable": "definition",
  "references.enable": "references",
  "documentSymbols.enable": "documentSymbols",
  "workspaceSymbols.enable": "workspaceSymbols",
  "codeActions.enable": "codeActions",
  "rename.enable": "rename",
  "codeLens.enable": "codeLens",
  "formatting.enable": "formatting",
  "semanticTokens.enable": "semanticTokens",
  "documentLinks.enable": "documentLinks",
  "foldingRanges.enable": "foldingRanges",
  "inlayHints.enable": "inlayHints",
  "fileRename.enable": "fileRename",
  "autoInsert.enable": "autoInsert",
} as const satisfies Record<(typeof FEATURE_SETTING_KEYS)[number], string>;

const FEATURE_MANIFEST_DEFAULTS = {
  "lint.enable": true,
  "diagnostics.enable": false,
  "typecheck.enable": true,
  "editor.enable": true,
  "ecosystem.enable": true,
  "optionsApi.enable": false,
  "legacyVue2.enable": false,
  "completion.enable": true,
  "signatureHelp.enable": true,
  "hover.enable": true,
  "definition.enable": true,
  "references.enable": true,
  "documentSymbols.enable": true,
  "workspaceSymbols.enable": true,
  "codeActions.enable": true,
  "rename.enable": true,
  "codeLens.enable": true,
  "formatting.enable": false,
  "semanticTokens.enable": true,
  "documentLinks.enable": true,
  "foldingRanges.enable": true,
  "inlayHints.enable": true,
  "fileRename.enable": true,
  "autoInsert.enable": false,
} as const satisfies Record<(typeof FEATURE_SETTING_KEYS)[number], boolean>;

type Scope = "global" | "workspace" | "workspaceFolder";
type ConfigValue = {
  scope: Scope;
  value: unknown;
};

class FakeConfig implements VizeConfigurationLike {
  readonly values: Record<string, ConfigValue>;

  constructor(values: Record<string, unknown>, scopes: Record<string, Scope> = {}) {
    this.values = Object.fromEntries(
      Object.entries(values).map(([key, value]) => [
        key,
        {
          scope: scopes[key] ?? "workspace",
          value,
        },
      ]),
    );
  }

  get<T>(key: string, defaultValue: T): T {
    return Object.hasOwn(this.values, key) ? (this.values[key].value as T) : defaultValue;
  }

  inspect<T>(key: string): ConfigurationInspection<T> | undefined {
    const entry = this.values[key];
    if (!entry) {
      return undefined;
    }
    if (entry.scope === "global") {
      return { globalValue: entry.value as T };
    }
    if (entry.scope === "workspaceFolder") {
      return { workspaceFolderValue: entry.value as T };
    }
    return { workspaceValue: entry.value as T };
  }
}

function readJson<T>(relativePath: string): T {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), "utf-8")) as T;
}

for (const key of FEATURE_SETTING_KEYS) {
  const option = FEATURE_TO_OPTION[key];

  test(`vscode explicit false for ${key} maps to ${option}:false`, () => {
    assert.deepEqual(getInitializationOptions(new FakeConfig({ enable: true, [key]: false })), {
      [option]: false,
    });
  });

  test(`vscode explicit true for ${key} maps to ${option}:true`, () => {
    assert.deepEqual(getInitializationOptions(new FakeConfig({ enable: true, [key]: true })), {
      [option]: true,
    });
  });
}

for (const key of FEATURE_SETTING_KEYS) {
  const option = FEATURE_TO_OPTION[key];

  for (const scope of ["global", "workspace", "workspaceFolder"] as const) {
    for (const value of [false, true] as const) {
      test(`vscode ${scope} ${key}:${value} is explicit and maps to ${option}`, () => {
        const config = new FakeConfig({ enable: true, [key]: value }, { [key]: scope });

        assert.deepEqual(getInitializationOptions(config), { [option]: value });
        assert.equal(hasAnyExplicitCapabilityValue(config), true);
      });
    }
  }
}

for (const key of FEATURE_SETTING_KEYS) {
  test(`vscode manifest default for ${key} matches extension-core fallback`, () => {
    const manifest = readJson<{
      contributes?: {
        configuration?: { properties?: Record<string, { default?: unknown; type?: string }> };
      };
    }>("editors/vscode/package.json");
    const property = manifest.contributes?.configuration?.properties?.[`vize.${key}`];

    assert.equal(property?.type, "boolean", `vize.${key}`);
    assert.equal(property?.default, FEATURE_MANIFEST_DEFAULTS[key], `vize.${key}`);
  });
}

test("vscode lint.enable takes precedence over deprecated diagnostics.enable", () => {
  assert.deepEqual(
    getInitializationOptions(
      new FakeConfig({
        enable: true,
        "diagnostics.enable": true,
        "lint.enable": false,
      }),
    ),
    { lint: false },
  );
  assert.deepEqual(
    getInitializationOptions(
      new FakeConfig({
        enable: true,
        "diagnostics.enable": false,
        "lint.enable": true,
      }),
    ),
    { lint: true },
  );
});

test("vscode deprecated diagnostics.enable still configures lint when lint is absent", () => {
  assert.deepEqual(
    getInitializationOptions(new FakeConfig({ enable: true, "diagnostics.enable": true })),
    { lint: true },
  );
  assert.deepEqual(
    getInitializationOptions(new FakeConfig({ enable: true, "diagnostics.enable": false })),
    { lint: false },
  );
});

test("vscode feature keys are complete boolean manifest properties", () => {
  const manifest = readJson<{
    contributes?: { configuration?: { properties?: Record<string, { type?: string }> } };
  }>("editors/vscode/package.json");
  const properties = manifest.contributes?.configuration?.properties ?? {};

  assert.deepEqual(Object.keys(FEATURE_TO_OPTION).sort(), [...FEATURE_SETTING_KEYS].sort());
  for (const key of FEATURE_SETTING_KEYS) {
    assert.equal(properties[`vize.${key}`]?.type, "boolean", `vize.${key}`);
  }
});

test("vscode lint-only profile updates every feature switch exactly once", () => {
  const keys = LINT_ONLY_CONFIGURATION_UPDATES.map(([key]) => key);

  assert.equal(new Set(keys).size, keys.length);
  assert.deepEqual(keys.sort(), ["enable", ...FEATURE_SETTING_KEYS].sort());
  assert.deepEqual(Object.fromEntries(LINT_ONLY_CONFIGURATION_UPDATES), {
    enable: true,
    "lint.enable": true,
    "diagnostics.enable": false,
    "typecheck.enable": false,
    "editor.enable": false,
    "ecosystem.enable": false,
    "optionsApi.enable": false,
    "legacyVue2.enable": false,
    "completion.enable": false,
    "signatureHelp.enable": false,
    "hover.enable": false,
    "definition.enable": false,
    "references.enable": false,
    "documentSymbols.enable": false,
    "workspaceSymbols.enable": false,
    "codeActions.enable": false,
    "rename.enable": false,
    "codeLens.enable": false,
    "formatting.enable": false,
    "semanticTokens.enable": false,
    "documentLinks.enable": false,
    "foldingRanges.enable": false,
    "inlayHints.enable": false,
    "fileRename.enable": false,
    "autoInsert.enable": false,
  });
});

test("vscode explicit feature switches suppress synthesized recommended defaults", () => {
  for (const key of FEATURE_SETTING_KEYS) {
    const options = getInitializationOptions(new FakeConfig({ enable: true, [key]: false }));
    assert.deepEqual(Object.keys(options), [FEATURE_TO_OPTION[key]]);
  }
});

test("vscode start and enabled-capability checks handle false-only configs", () => {
  const config = new FakeConfig({
    enable: true,
    "completion.enable": false,
    "editor.enable": false,
    "lint.enable": false,
  });

  assert.equal(shouldStartFromConfiguration(config), true);
  assert.equal(hasAnyEnabledCapability(config), false);
  assert.equal(hasAnyExplicitCapabilityValue(config), true);
});

test("vscode file rename can be enabled without the editor bundle", () => {
  const config = new FakeConfig({
    enable: true,
    "editor.enable": false,
    "fileRename.enable": true,
  });

  assert.deepEqual(getInitializationOptions(config), {
    editor: false,
    fileRename: true,
  });
  assert.equal(hasAnyEnabledCapability(config), true);
  assert.equal(hasAnyExplicitCapabilityValue(config), true);
});

test("vscode automatic insertion is an explicit opt-in independent of the editor bundle", () => {
  const config = new FakeConfig({
    enable: true,
    "editor.enable": false,
    "autoInsert.enable": true,
  });

  assert.deepEqual(getInitializationOptions(config), {
    editor: false,
    autoInsert: true,
  });
  assert.equal(hasAnyEnabledCapability(config), true);
});

test("vscode explicit capability detection ignores unrelated settings", () => {
  const config = new FakeConfig({
    enable: true,
    serverPath: "/tmp/vize",
    "trace.server": "verbose",
  });

  assert.equal(hasAnyExplicitCapabilityValue(config), false);
  assert.deepEqual(getInitializationOptions(config), {
    editor: true,
    ecosystem: true,
    lint: true,
    typecheck: true,
  });
});

test("vscode document selector does not duplicate language and scheme pairs", () => {
  const selector = createDocumentSelector();
  const pairs = selector.map(({ language, scheme }) => `${language}:${scheme}`);

  assert.equal(new Set(pairs).size, pairs.length);
  assert.deepEqual(pairs.sort(), [
    "art-vue:file",
    "art-vue:untitled",
    "html:file",
    "html:untitled",
    "vue:file",
    "vue:untitled",
  ]);
});

test("vscode capability description only includes true options in stable order", () => {
  assert.equal(
    describeCapabilities({
      codeActions: false,
      completion: true,
      editor: true,
      unknownFutureFeature: true,
    }),
    "completion, editor bundle, unknownFutureFeature",
  );
});

test("vscode capability description names file rename explicitly", () => {
  assert.equal(
    describeCapabilities({
      documentLinks: true,
      fileRename: true,
      rename: false,
    }),
    "document links, file rename",
  );
});
