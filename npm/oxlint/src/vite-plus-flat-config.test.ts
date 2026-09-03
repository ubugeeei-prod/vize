import assert from "node:assert/strict";
import test from "node:test";

import {
  loadVitePlusConfigHelpersFromDist,
  typecheckVitePlusConfigConsumer,
} from "./test-support/vite-plus-workspace.ts";

const { createVizeLintFlatConfig, defineVizeLintConfig, flatConfigs } =
  await loadVitePlusConfigHelpersFromDist();

void test("packed declarations type-check as a strict Vite+ lint consumer", () => {
  typecheckVitePlusConfigConsumer(`import { createVizeLintConfig, defineVizeLintConfig, flatConfigs } from "oxlint-plugin-vize";
import type { VitePlusLintPlugin, VizeLintFlatConfig } from "oxlint-plugin-vize";
import { defineConfig } from "vite-plus";
import type { OxlintConfig } from "vite-plus/lint";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends
  (<Value>() => Value extends Right ? 1 : 2) ? true : false;
type Assert<Condition extends true> = Condition;
type VitePlusPlugin = NonNullable<OxlintConfig["plugins"]>[number];
type _PluginNamesStayInSync = Assert<Equal<VitePlusLintPlugin, VitePlusPlugin>>;
const _flatConfig: VizeLintFlatConfig = flatConfigs.recommended;

export default defineConfig({
  lint: defineVizeLintConfig(
    ...flatConfigs.recommended,
    ...flatConfigs.ecosystem,
    {
      plugins: ["typescript"],
      rules: {
        "no-console": "off",
        "typescript/consistent-type-imports": [
          "error",
          { disallowTypeAnnotations: false, fixStyle: "inline-type-imports" },
        ],
      },
      settings: {
        vize: {
          helpLevel: "none",
        },
      },
    },
    {
      ...createVizeLintConfig({
        preset: "incremental",
      }),
    },
    {
      overrides: [
        {
          files: ["**/*.ts"],
          rules: {
            "typescript/consistent-type-imports": [
              "warn",
              { disallowTypeAnnotations: true, fixStyle: "separate-type-imports" },
            ],
          },
        },
      ],
    },
  ),
});

createVizeLintConfig({
  // @ts-expect-error Vite+ keeps built-in plugin names as a closed union.
  plugins: ["not-a-vite-plus-plugin"],
  rules: {
    // @ts-expect-error Unknown severities must not be widened to string or any.
    "no-console": "verbose",
  },
});
`);
});

void test("defineVizeLintConfig merges Flat Config fragments into Vite+'s object shape", () => {
  const config = defineVizeLintConfig(
    ...flatConfigs.recommended,
    {
      extends: [...flatConfigs.ecosystem],
      ignorePatterns: ["dist/**"],
      rules: {
        "no-console": "warn",
      },
      settings: {
        vize: {
          helpLevel: "none",
        },
      },
    },
    createVizeLintFlatConfig({
      plugins: ["typescript"],
      preset: "incremental",
      rules: {
        "typescript/consistent-type-imports": "error",
      },
    }),
    {
      ignorePatterns: ["coverage/**", "dist/**"],
    },
  );

  assert.deepEqual(config.jsPlugins, ["oxlint-plugin-vize"]);
  assert.deepEqual(config.plugins, ["vue", "typescript"]);
  assert.deepEqual(config.ignorePatterns, ["dist/**", "coverage/**"]);
  assert.equal(config.settings.vize.preset, "incremental");
  assert.equal(config.settings.vize.helpLevel, "none");
  assert.equal(config.rules["vize/script/valid-define-props"], "error");
  assert.equal(config.rules["vize/ecosystem/router-link-require-to"], "error");
  assert.equal(config.rules["vize/script/no-options-api"], undefined);
  assert.equal(config.rules["no-console"], "warn");
  assert.equal(config.rules["typescript/consistent-type-imports"], "error");
});
