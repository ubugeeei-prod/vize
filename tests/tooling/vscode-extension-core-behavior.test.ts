import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  FEATURE_SETTING_KEYS,
  createDocumentSelector,
  describeCapabilities,
  getInitializationOptions,
  hasAnyEnabledCapability,
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
} as const satisfies Record<(typeof FEATURE_SETTING_KEYS)[number], string>;

class FakeConfig implements VizeConfigurationLike {
  readonly values: Record<string, unknown>;

  constructor(values: Record<string, unknown>) {
    this.values = values;
  }

  get<T>(key: string, defaultValue: T): T {
    return Object.hasOwn(this.values, key) ? (this.values[key] as T) : defaultValue;
  }

  inspect<T>(key: string): ConfigurationInspection<T> | undefined {
    if (!Object.hasOwn(this.values, key)) {
      return undefined;
    }
    return { workspaceValue: this.values[key] as T };
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
