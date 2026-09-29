import { definePlugin, defineRule, type Diagnostic } from "@oxlint/plugins";

import { getPatinaRules } from "./binding.js";
import {
  getFileState,
  getDiagnosticsForRule,
  getScriptMap,
  getSfcBlocks,
  markDiagnosticAsReported,
  type FileState,
} from "./file-state.js";
import { formatPatinaMessage } from "./format.js";
import type {
  ComponentNameInTemplateCasingOption,
  HelpLevel,
  HtmlSelfClosingOption,
  HyphenationStyle,
  NoMutatingPropsOption,
  PatinaDiagnostic,
  PatinaRuleOptions,
  PatinaRuleMeta,
  CustomEventNameCasingOption,
  SfcElementOrderOption,
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

          const helpLevel = settings.helpLevel ?? "full";
          const state = getFileState(context);
          const scriptMap = getScriptMap(state);
          const ruleOptions = getRuleOptions(ruleMeta.name, contextOptions(context));
          const diagnostics = getDiagnosticsForRule(
            context,
            state,
            ruleMeta.name,
            ruleOptions,
          ).filter((diagnostic) => shouldReportForCurrentProgram(diagnostic, state, scriptMap));
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
      return [{ enum: ["PascalCase", "kebab-case"] }];
    case "script/custom-event-name-casing":
      return [{ enum: ["camelCase", "kebab-case"] }];
    case "vue/no-mutating-props":
      return [noMutatingPropsSchema()];
    case "vue/sfc-element-order":
      return [sfcElementOrderSchema()];
    case "vue/html-self-closing":
      return [htmlSelfClosingSchema()];
    case "vue/v-on-event-hyphenation":
    case "vue/attribute-hyphenation":
      return [{ enum: ["always", "never"] }];
    default:
      return [];
  }
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

function getRuleOptions(
  ruleName: string,
  options: readonly unknown[],
): PatinaRuleOptions | undefined {
  const firstOption = options[0];
  switch (ruleName) {
    case "vue/component-name-in-template-casing":
      if (isComponentNameInTemplateCasingOption(firstOption)) {
        return { componentNameInTemplateCasing: firstOption };
      }
      break;
    case "script/custom-event-name-casing":
      if (isCustomEventNameCasingOption(firstOption)) {
        return { customEventNameCasing: firstOption };
      }
      break;
    case "vue/no-mutating-props":
      if (isNoMutatingPropsOption(firstOption)) {
        return { noMutatingProps: firstOption };
      }
      break;
    case "vue/sfc-element-order":
      if (isSfcElementOrderOption(firstOption)) {
        return { sfcElementOrder: firstOption };
      }
      break;
    case "vue/html-self-closing":
      if (isHtmlSelfClosingOption(firstOption)) {
        return { htmlSelfClosing: firstOption };
      }
      break;
    case "vue/v-on-event-hyphenation":
      if (isHyphenationStyle(firstOption)) {
        return { vOnEventHyphenation: firstOption };
      }
      break;
    case "vue/attribute-hyphenation":
      if (isHyphenationStyle(firstOption)) {
        return { attributeHyphenation: firstOption };
      }
      break;
  }
  return undefined;
}

function isComponentNameInTemplateCasingOption(
  value: unknown,
): value is ComponentNameInTemplateCasingOption {
  return value === "PascalCase" || value === "kebab-case";
}

function isCustomEventNameCasingOption(value: unknown): value is CustomEventNameCasingOption {
  return value === "camelCase" || value === "kebab-case";
}

function isNoMutatingPropsOption(value: unknown): value is NoMutatingPropsOption {
  if (!isRecord(value)) {
    return false;
  }
  return hasOnlyKeys(value, ["shallowOnly"]) && optionalBooleanField(value.shallowOnly);
}

function isSfcElementOrderOption(value: unknown): value is SfcElementOrderOption {
  if (!isRecord(value)) {
    return false;
  }
  return (
    hasOnlyKeys(value, ["order"]) &&
    (value.order === undefined ||
      (Array.isArray(value.order) && value.order.every(isSfcElementOrderGroup)))
  );
}

function isSfcElementOrderGroup(value: unknown): boolean {
  return typeof value === "string" || (Array.isArray(value) && value.every(isString));
}

function isHtmlSelfClosingOption(value: unknown): value is HtmlSelfClosingOption {
  if (!isRecord(value)) {
    return false;
  }
  return (
    hasOnlyKeys(value, ["html", "svg", "math"]) &&
    optionField(value.svg) &&
    optionField(value.math) &&
    (value.html === undefined ||
      (isRecord(value.html) &&
        hasOnlyKeys(value.html, ["void", "normal", "component"]) &&
        optionField(value.html.void) &&
        optionField(value.html.normal) &&
        optionField(value.html.component)))
  );
}

function isHyphenationStyle(value: unknown): value is HyphenationStyle {
  return value === "always" || value === "never";
}

function optionField(value: unknown): boolean {
  return value === undefined || value === "always" || value === "never" || value === "any";
}

function optionalBooleanField(value: unknown): boolean {
  return value === undefined || typeof value === "boolean";
}

function isString(value: unknown): value is string {
  return typeof value === "string";
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function hasOnlyKeys(value: Record<string, unknown>, keys: readonly string[]): boolean {
  return Object.keys(value).every((key) => keys.includes(key));
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
