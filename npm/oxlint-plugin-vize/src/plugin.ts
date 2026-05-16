import { definePlugin, defineRule, type Diagnostic } from "@oxlint/plugins";

import { getPatinaRules } from "./binding.js";
import {
  getFileState,
  getDiagnosticsForRule,
  getScriptMap,
  getSfcBlocks,
  markRuleAsReported,
  type FileState,
} from "./file-state.js";
import { formatPatinaMessage } from "./format.js";
import type { HelpLevel, PatinaDiagnostic, PatinaRuleMeta } from "./model.js";
import { formatBlockLabel, getDiagnosticBlock } from "./sfc-blocks.js";
import { mapToScriptLoc } from "./script-map.js";
import {
  getActivePreset,
  getVizeSettings,
  isIncrementalPreset,
  isVueLikeFile,
} from "./settings.js";

function createOxlintDiagnostic(
  diagnostic: PatinaDiagnostic,
  state: FileState,
  helpLevel: HelpLevel,
): Diagnostic {
  const scriptMap = getScriptMap(state);
  const loc = state.usesOriginalLocations
    ? createOriginalSfcLoc(diagnostic)
    : mapToScriptLoc(diagnostic, scriptMap);
  const block = loc === null ? getDiagnosticBlock(diagnostic, getSfcBlocks(state)) : null;

  return {
    loc: loc ?? {
      start: { line: 1, column: 1 },
      end: { line: 1, column: 1 },
    },
    message: formatPatinaMessage(diagnostic, {
      hasMappedLocation: loc !== null,
      blockLabel: formatBlockLabel(block),
      helpLevel,
    }),
  };
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
    },
    createOnce(context) {
      return {
        Program() {
          if (!isVueLikeFile(context.filename)) {
            return;
          }

          const settings = getVizeSettings(context);
          const activePreset = getActivePreset(settings);
          if (!isIncrementalPreset(settings) && !ruleMeta.presets.includes(activePreset)) {
            return;
          }

          const helpLevel = settings.helpLevel ?? "full";
          const state = getFileState(context);
          const diagnostics = getDiagnosticsForRule(context, state, ruleMeta.name);
          if (diagnostics.length === 0) {
            return;
          }
          if (!markRuleAsReported(state, ruleMeta.name)) {
            return;
          }

          for (const diagnostic of diagnostics) {
            context.report(createOxlintDiagnostic(diagnostic, state, helpLevel));
          }
        },
      };
    },
  });
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
