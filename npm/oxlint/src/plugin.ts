import { definePlugin, defineRule, type Diagnostic } from "@oxlint/plugins";
import { getRuleOptions } from "./rule-options.js";
import { collectActiveRule, getActiveRuleDiagnostics } from "./active-rule-collection.js";

import { getPatinaRules } from "./binding.js";
import {
  getScriptMap,
  getSfcBlocks,
  markDiagnosticAsReported,
  type FileState,
} from "./file-state.js";
import { formatPatinaMessage } from "./format.js";
import type {
  HelpLevel,
  PatinaDiagnostic,
  PatinaRuleMeta,
  SfcBlock,
  SingleScriptMap,
} from "./model.js";
import { formatBlockLabel, getDiagnosticBlock } from "./sfc-blocks.js";
import { mapToScriptLoc } from "./script-map.js";
import { isLocationBridgeProgram } from "./workaround.js";
import { isScriptLikeFile } from "./file-kinds.js";
import { getActivePreset, getVizeSettings, isIncrementalPreset, isPatinaFile } from "./settings.js";

function createOxlintDiagnostic(
  diagnostic: PatinaDiagnostic,
  state: FileState,
  scriptMap: SingleScriptMap | null,
  helpLevel: HelpLevel,
  program: { range: [number, number] },
): Diagnostic {
  const loc = state.usesOriginalLocations
    ? createOriginalSfcLoc(diagnostic)
    : mapToScriptLoc(diagnostic, scriptMap);
  const block = loc === null ? getDiagnosticBlock(diagnostic, getSfcBlocks(state)) : null;
  const message = formatPatinaMessage(diagnostic, {
    hasMappedLocation: loc !== null,
    blockLabel: formatBlockLabel(block),
    helpLevel,
  });
  // An empty `<script>` has no line for a column to land on. Oxlint rejects
  // that line/column pair with RangeError, so anchor the report on the program
  // node. Template coordinates stay in the message; the script program cannot
  // address a node that lives outside it.
  if (loc == null && state.extractedScript.length === 0) {
    return { node: program, message };
  }

  const fallbackColumn = state.extractedScript.length === 0 ? 0 : 1;
  return {
    loc: loc ?? {
      start: { line: 1, column: fallbackColumn },
      end: { line: 1, column: fallbackColumn },
    },
    message,
  };
}

function shouldReportForCurrentProgram(
  diagnostic: PatinaDiagnostic,
  state: FileState,
  scriptMap: SingleScriptMap | null,
): boolean {
  if (state.usesOriginalLocations) {
    return isScriptLikeFile(state.filename) || isLocationBridgeProgram(state.extractedScript);
  }
  if (scriptMap == null) {
    return true;
  }

  const block = getDiagnosticBlock(diagnostic, getSfcBlocks(state));
  return !isScriptBlock(block) || block === scriptMap.block;
}

function isScriptBlock(block: SfcBlock | null): boolean {
  return block?.kind === "script" || block?.kind === "script-setup";
}

function createOriginalSfcLoc(diagnostic: PatinaDiagnostic): Diagnostic["loc"] {
  return {
    start: {
      line: diagnostic.location.start.line,
      column: Math.max(0, diagnostic.location.start.column - 1),
    },
    end: {
      line: diagnostic.location.end.line,
      column: Math.max(0, diagnostic.location.end.column - 1),
    },
  };
}

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
            getRuleOptions(ruleMeta.name, contextOptions(context)),
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

function ruleOptionsSchema(ruleName: string): unknown[] {
  switch (ruleName) {
    case "vue/component-name-in-template-casing":
      return [
        {
          oneOf: [
            { enum: ["PascalCase", "kebab-case"] },
            objectSchema({
              casing: { enum: ["PascalCase", "kebab-case"] },
              registeredComponentsOnly: { type: "boolean" },
              globals: stringArraySchema(),
            }),
          ],
        },
      ];
    case "script/custom-event-name-casing":
      return [
        {
          oneOf: [
            { enum: ["camelCase", "kebab-case"] },
            objectSchema({ casing: { enum: ["camelCase", "kebab-case"] } }),
          ],
        },
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
    case "script/define-macros-order":
      return [defineMacrosOrderSchema()];
    case "vue/sfc-element-order":
      return [sfcElementOrderSchema()];
    case "vue/html-self-closing":
      return [htmlSelfClosingSchema()];
    case "vue/v-on-event-hyphenation":
    case "vue/attribute-hyphenation":
      return [{ enum: ["always", "never"] }];
    case "vue/v-on-style":
      return [{ enum: ["shorthand", "longform"] }];
    case "vue/require-component-registration":
      return [objectSchema({ globals: stringArraySchema() })];
    case "vue/valid-v-on":
      return [objectSchema({ modifiers: stringArraySchema() })];
    case "vue/valid-v-slot":
      return [objectSchema({ allowModifiers: { type: "boolean" } })];
    case "musea/prefer-design-tokens":
      return [designTokensSchema()];
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
  rules: patinaRules,
});
