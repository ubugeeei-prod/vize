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
  hasExplicitConfigurationValue,
  shouldStartFromConfiguration,
  type ConfigurationInspection,
  type VizeConfigurationLike,
} from "../../editors/vscode/src/extension-core.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

type Scope = "global" | "workspace" | "workspaceFolder";
type ConfigValue = {
  scope: Scope;
  value: unknown;
};

const MATRIX_FEATURES = [
  ["lint.enable", "lint"],
  ["typecheck.enable", "typecheck"],
  ["editor.enable", "editor"],
  ["ecosystem.enable", "ecosystem"],
  ["optionsApi.enable", "optionsApi"],
  ["legacyVue2.enable", "legacyVue2"],
  ["completion.enable", "completion"],
  ["hover.enable", "hover"],
  ["definition.enable", "definition"],
  ["references.enable", "references"],
  ["formatting.enable", "formatting"],
] as const;
const MATRIX_CASE_COUNT = 1 << MATRIX_FEATURES.length;

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
    if (!Object.hasOwn(this.values, key)) {
      return defaultValue;
    }
    return this.values[key].value as T;
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

function readRepoFile(relativePath: string): string {
  return fs.readFileSync(path.join(root, relativePath), "utf-8");
}

function readJson<T>(relativePath: string): T {
  return JSON.parse(readRepoFile(relativePath)) as T;
}

for (let mask = 0; mask < MATRIX_CASE_COUNT; mask++) {
  test(`vscode lsp initialization feature matrix ${mask.toString(16).padStart(3, "0")}`, () => {
    const values: Record<string, unknown> = { enable: true };
    const expected: Record<string, boolean> = {};

    for (let index = 0; index < MATRIX_FEATURES.length; index++) {
      const [settingKey, optionKey] = MATRIX_FEATURES[index];
      const enabled = (mask & (1 << index)) !== 0;
      values[settingKey] = enabled;
      expected[optionKey] = enabled;
    }

    const config = new FakeConfig(values);
    assert.deepEqual(getInitializationOptions(config), expected);
    assert.equal(
      hasAnyEnabledCapability(config),
      Object.values(expected).some((enabled) => enabled),
    );
    assert.equal(shouldStartFromConfiguration(config), true);
  });
}

test("lsp editor matrix generates more than 2000 executable cases", () => {
  assert.equal(MATRIX_CASE_COUNT, 2048);
});

test("lint-only profile explicitly disables every non-lint capability", () => {
  const updates = Object.fromEntries(LINT_ONLY_CONFIGURATION_UPDATES);
  const missingKeys = FEATURE_SETTING_KEYS.filter((key) => !Object.hasOwn(updates, key));

  assert.deepEqual(missingKeys, []);
  assert.deepEqual(getInitializationOptions(new FakeConfig(updates)), {
    codeActions: false,
    codeLens: false,
    completion: false,
    definition: false,
    documentLinks: false,
    documentSymbols: false,
    ecosystem: false,
    editor: false,
    fileRename: false,
    autoInsert: false,
    foldingRanges: false,
    formatting: false,
    hover: false,
    inlayHints: false,
    legacyVue2: false,
    lint: true,
    optionsApi: false,
    references: false,
    rename: false,
    semanticTokens: false,
    signatureHelp: false,
    typecheck: false,
    workspaceSymbols: false,
  });
});

test("feature setting keys are backed by the VS Code manifest", () => {
  const manifest = readJson<{
    contributes?: {
      configuration?: {
        properties?: Record<string, unknown>;
      };
    };
  }>("editors/vscode/package.json");
  const properties = manifest.contributes?.configuration?.properties ?? {};

  assert.ok(Object.hasOwn(properties, "vize.enable"));
  for (const key of FEATURE_SETTING_KEYS) {
    assert.ok(Object.hasOwn(properties, `vize.${key}`), `missing manifest setting vize.${key}`);
  }
});

test("every direct VS Code feature switch forwards explicit LSP option values", () => {
  for (const key of FEATURE_SETTING_KEYS) {
    if (key === "diagnostics.enable") {
      continue;
    }

    const option = key.slice(0, -".enable".length);

    assert.deepEqual(
      getInitializationOptions(new FakeConfig({ enable: true, [key]: true })),
      { [option]: true },
      `${key} should enable ${option}`,
    );
    assert.deepEqual(
      getInitializationOptions(new FakeConfig({ enable: true, [key]: false })),
      { [option]: false },
      `${key} should disable ${option}`,
    );
  }
});

test("document selector covers supported language and URI scheme product", () => {
  assert.deepEqual(createDocumentSelector(), [
    { scheme: "file", language: "vue" },
    { scheme: "file", language: "art-vue" },
    { scheme: "file", language: "html" },
    { scheme: "untitled", language: "vue" },
    { scheme: "untitled", language: "art-vue" },
    { scheme: "untitled", language: "html" },
  ]);
});

test("default recommended profile is only synthesized for explicit enable without workspace lsp config", () => {
  const logs: string[] = [];

  assert.deepEqual(
    getInitializationOptions(new FakeConfig({ enable: true }), {
      log: (message) => logs.push(message),
    }),
    {
      editor: true,
      ecosystem: true,
      lint: true,
      typecheck: true,
    },
  );
  assert.equal(logs.length, 1);
  assert.deepEqual(
    getInitializationOptions(new FakeConfig({ enable: true }), {
      hasWorkspaceLspConfig: true,
      log: (message) => logs.push(message),
    }),
    {},
  );
});

test("diagnostics alias keeps lint precedence predictable", () => {
  assert.deepEqual(
    getInitializationOptions(new FakeConfig({ "diagnostics.enable": true, enable: true })),
    { lint: true },
  );
  assert.deepEqual(
    getInitializationOptions(new FakeConfig({ "diagnostics.enable": false, enable: true })),
    { lint: false },
  );
  assert.deepEqual(
    getInitializationOptions(
      new FakeConfig({
        "diagnostics.enable": false,
        enable: true,
        "lint.enable": true,
      }),
    ),
    { lint: true },
  );
});

test("explicit configuration detection respects global workspace and folder scopes", () => {
  assert.equal(hasExplicitConfigurationValue(new FakeConfig({ enable: true }), "enable"), true);
  assert.equal(
    hasExplicitConfigurationValue(new FakeConfig({ enable: true }, { enable: "global" }), "enable"),
    true,
  );
  assert.equal(
    hasExplicitConfigurationValue(
      new FakeConfig({ enable: true }, { enable: "workspaceFolder" }),
      "enable",
    ),
    true,
  );
  assert.equal(hasExplicitConfigurationValue(new FakeConfig({}), "enable"), false);
});

test("start condition mirrors VS Code workspace lsp fallback rules", () => {
  assert.equal(shouldStartFromConfiguration(new FakeConfig({ enable: true })), true);
  assert.equal(shouldStartFromConfiguration(new FakeConfig({ enable: false }), true), false);
  assert.equal(shouldStartFromConfiguration(new FakeConfig({}), true), true);
  assert.equal(shouldStartFromConfiguration(new FakeConfig({}), false), false);
});

test("capability descriptions expose readable editor labels", () => {
  assert.equal(describeCapabilities({}), "none");
  assert.equal(
    describeCapabilities({
      codeActions: true,
      editor: true,
      formatting: false,
      lint: true,
      optionsApi: true,
      semanticTokens: true,
    }),
    "code actions, editor bundle, lint, Vue 3 Options API, semantic tokens",
  );
});

test("editor integrations launch vize lsp by default", () => {
  assert.match(readRepoFile("editors/vscode/src/extension.ts"), /args:\s*\["lsp"\]/);
  assert.match(readRepoFile("editors/vscode/src/extension.ts"), /args:\s*\["lsp",\s*"--debug"\]/);
  assert.match(readRepoFile("editors/zed/src/lib.rs"), /vec!\["lsp"\.to_string\(\)\]/);
  assert.match(readRepoFile("editors/emacs/vize.el"), /'\("vize" "lsp"\)/);
  assert.match(readRepoFile("editors/helix/languages.toml"), /args = \["lsp"\]/);
});
