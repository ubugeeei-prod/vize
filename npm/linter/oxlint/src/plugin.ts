import { definePlugin, defineRule } from "@oxlint/plugins";
import { getRuleOptions } from "./rule-options.js";
import { vOnEventHyphenationRuleSchema } from "./v-on-event-options.js";
import { attributeHyphenationSchema } from "./attribute-options.js";
import { propNameCasingSchema } from "./prop-name-options.js";
import { componentNameCasingSchema } from "./component-name-options.js";
import { restrictedHtmlElementsSchema, restrictedVOnSchema } from "./restricted-entry-options.js";
import { blockLangSchema, maxLinesPerBlockSchema, maxLinesSchema } from "./block-limit-options.js";
import { collectActiveRule, getActiveRuleDiagnostics } from "./active-rule-collection.js";

import { getPatinaRules, withHostGlobals } from "./binding.js";
import { pluginRules } from "./plugin-rules.js";
import { createOxlintDiagnostic, shouldReportForCurrentProgram } from "./report.js";
import { getScriptMap, markDiagnosticAsReported } from "./file-state.js";
import type { PatinaRuleMeta } from "./model.js";
import { getActivePreset, getVizeSettings, isIncrementalPreset, isPatinaFile } from "./settings.js";

function createPatinaRule(ruleMeta: PatinaRuleMeta) {
  return defineRule({
    meta: {
      type: ruleMeta.defaultSeverity === "error" ? "problem" : "suggestion",
      docs: {
        description: ruleMeta.description,
      },
      schema: ruleOptionsSchema(ruleMeta.name),
    },
    createOnce(context) {
      return {
        Program(program) {
          if (!isPatinaFile(context.filename)) {
            return;
          }

          const settings = getVizeSettings(context);
          const activePreset = getActivePreset(settings);
          if (
            ruleMeta.presets.length > 0 &&
            !isIncrementalPreset(settings) &&
            !ruleMeta.presets.includes(activePreset)
          ) {
            return;
          }

          collectActiveRule(
            context,
            program,
            ruleMeta.name,
            withHostGlobals(
              ruleMeta.name,
              getRuleOptions(ruleMeta.name, contextOptions(context)),
              context.languageOptions.globals,
            ),
          );
        },
        "Program:exit"(program) {
          const collected = getActiveRuleDiagnostics(context, program, ruleMeta.name);
          if (!collected) return;
          const { state } = collected;
          const helpLevel = getVizeSettings(context).helpLevel ?? "full";
          const scriptMap = getScriptMap(state);
          const diagnostics = collected.diagnostics.filter((diagnostic) =>
            shouldReportForCurrentProgram(diagnostic, state, scriptMap),
          );
          if (diagnostics.length === 0) {
            return;
          }

          for (const diagnostic of diagnostics) {
            if (!markDiagnosticAsReported(state, diagnostic)) {
              continue;
            }

            context.report(
              createOxlintDiagnostic(diagnostic, state, scriptMap, helpLevel, program),
            );
          }
        },
      };
    },
  });
}

function ruleOptionsSchema(ruleName: string): unknown[] | Record<string, unknown> {
  switch (ruleName) {
    case "vue/prop-name-casing":
      return [propNameCasingSchema()];
    case "vue/component-name-in-template-casing":
      return componentNameCasingSchema();
    case "script/custom-event-name-casing":
      return [
        {
          oneOf: [
            { enum: ["lowercase", "camelCase", "kebab-case"] },
            objectSchema({ casing: { enum: ["lowercase", "camelCase", "kebab-case"] } }),
          ],
        },
      ];
    case "script/consistent-type-imports":
      return [
        objectSchema({
          prefer: { enum: ["type-imports", "no-type-imports"] },
          fixStyle: { enum: ["separate-type-imports", "inline-type-imports"] },
          disallowTypeAnnotations: { type: "boolean" },
          emitDecoratorMetadata: { type: "boolean" },
        }),
      ];
    case "script/define-props-destructuring":
      return [objectSchema({ destructure: { enum: ["only-when-assigned", "always", "never"] } })];
    case "html/no-empty-palpable-content":
      return [objectSchema({ contentDirectives: stringArraySchema() })];
    case "type/strict-boolean-expressions":
      return [strictBooleanExpressionsSchema()];
    case "script/no-restricted-globals":
      return [restrictedGlobalsSchema()];
    case "script/no-restricted-members":
      return [restrictedMembersSchema()];
    case "vue/no-mutating-props":
      return [noMutatingPropsSchema()];
    case "vue/warn-custom-directive":
      return [objectSchema({ ignore: stringArraySchema() })];
    case "vue/no-bare-strings-in-template":
      return [objectSchema({ allowlist: stringArraySchema() })];
    case "a11y/anchor-has-content":
      return [objectSchema({ accessibleChildren: stringArraySchema() })];
    case "a11y/label-has-for":
      return [objectSchema({ controlComponents: stringArraySchema() })];
    case "vue/enforce-style-attribute":
      return [
        objectSchema({
          allow: {
            type: "array",
            items: { enum: ["plain", "scoped", "module"] },
            uniqueItems: true,
          },
        }),
      ];
    case "vue/no-restricted-syntax":
      return {
        type: "array",
        items: {
          oneOf: [
            { type: "string" },
            {
              type: "object",
              properties: { selector: { type: "string" }, message: { type: "string" } },
              required: ["selector"],
              additionalProperties: false,
            },
          ],
        },
        uniqueItems: true,
      };
    case "vue/quote-props":
      return [{ enum: ["as-needed"] }];
    case "vue/sort-keys":
      return [
        { enum: ["asc", "desc"] },
        objectSchema({
          caseSensitive: { type: "boolean" },
          natural: { type: "boolean" },
        }),
      ];
    case "script/define-macros-order":
      return [defineMacrosOrderSchema()];
    case "vue/sfc-element-order":
      return [sfcElementOrderSchema()];
    case "vue/html-self-closing":
      return [htmlSelfClosingSchema()];
    case "vue/html-quotes":
      return [{ enum: ["double", "single"] }, objectSchema({ avoidEscape: { type: "boolean" } })];
    case "vue/mustache-interpolation-spacing":
      return [{ enum: ["always", "never"] }];
    case "vue/v-on-event-hyphenation":
      return vOnEventHyphenationRuleSchema();
    case "vue/attribute-hyphenation":
      return attributeHyphenationSchema();
    case "vue/v-on-style":
      return [{ enum: ["shorthand", "longform"] }];
    case "vue/v-slot-style":
      return [
        {
          oneOf: [
            { enum: ["shorthand", "longform"] },
            objectSchema({
              atComponent: { enum: ["shorthand", "longform", "v-slot"] },
              default: { enum: ["shorthand", "longform", "v-slot"] },
              named: { enum: ["shorthand", "longform"] },
            }),
          ],
        },
      ];
    case "vue/v-bind-style":
      return [
        { enum: ["shorthand", "longform"] },
        objectSchema({ sameNameShorthand: { enum: ["always", "never", "ignore"] } }),
      ];
    case "vue/no-restricted-html-elements":
      return restrictedHtmlElementsSchema();
    case "vue/no-restricted-v-on":
      return restrictedVOnSchema();
    case "vue/require-component-registration":
    case "vue/no-undefined-refs":
      return [objectSchema({ globals: stringArraySchema() })];
    case "vue/valid-v-on":
      return [objectSchema({ modifiers: stringArraySchema() })];
    case "vue/valid-v-slot":
      return [objectSchema({ allowModifiers: { type: "boolean" } })];
    case "musea/prefer-design-tokens":
      return [designTokensSchema()];
    case "vue/block-lang":
      return [blockLangSchema()];
    case "max-lines":
      return [maxLinesSchema()];
    case "vue/max-lines-per-block":
      return [maxLinesPerBlockSchema()];
    default:
      return [];
  }
}

function defineMacrosOrderSchema(): unknown {
  return objectSchema({
    order: {
      type: "array",
      items: { type: "string", minLength: 1 },
      uniqueItems: true,
    },
  });
}

function objectSchema(properties: Record<string, unknown>): Record<string, unknown> {
  return { type: "object", properties, additionalProperties: false };
}

function stringArraySchema(): unknown {
  return { type: "array", items: { type: "string" } };
}

function strictBooleanExpressionsSchema(): unknown {
  const boolean = { type: "boolean" };
  return objectSchema({
    allowString: boolean,
    allowNumber: boolean,
    allowNullableObject: boolean,
    allowNullableBoolean: boolean,
    allowNullableString: boolean,
    allowNullableNumber: boolean,
    allowNullableEnum: boolean,
    allowAny: boolean,
  });
}

function restrictedGlobalsSchema(): unknown {
  return objectSchema({
    globals: {
      type: "array",
      items: {
        ...objectSchema({ name: { type: "string" }, message: { type: "string" } }),
        required: ["name"],
      },
    },
  });
}

function restrictedMembersSchema(): unknown {
  return objectSchema({
    members: {
      type: "array",
      items: {
        ...objectSchema({
          object: { type: "string" },
          property: { type: "string" },
          message: { type: "string" },
        }),
        required: ["object", "property"],
      },
    },
  });
}

function designTokensSchema(): unknown {
  return objectSchema({
    tokens: {
      type: "array",
      items: {
        ...objectSchema({
          path: { type: "string" },
          value: { type: "string" },
          tier: { type: "string" },
        }),
        required: ["path", "value"],
      },
    },
  });
}

function noMutatingPropsSchema(): unknown {
  return {
    type: "object",
    properties: {
      shallowOnly: { type: "boolean" },
    },
    additionalProperties: false,
  };
}

function sfcElementOrderSchema(): unknown {
  const selector = { type: "string" };
  return {
    type: "object",
    properties: {
      order: {
        type: "array",
        items: {
          oneOf: [
            selector,
            {
              type: "array",
              items: selector,
            },
          ],
        },
      },
    },
    additionalProperties: false,
  };
}

function htmlSelfClosingSchema(): unknown {
  const style = { enum: ["always", "never", "any"] };
  return {
    type: "object",
    properties: {
      html: {
        type: "object",
        properties: {
          void: style,
          normal: style,
          component: style,
        },
        additionalProperties: false,
      },
      svg: style,
      math: style,
    },
    additionalProperties: false,
  };
}

function contextOptions(context: unknown): readonly unknown[] {
  const options = (context as { options?: unknown }).options;
  return Array.isArray(options) ? options : [];
}

const patinaRules = Object.fromEntries(
  getPatinaRules().map((ruleMeta) => [ruleMeta.name, createPatinaRule(ruleMeta)]),
);

export default definePlugin({
  meta: {
    name: "vize",
  },
  rules: { ...patinaRules, ...pluginRules },
});
