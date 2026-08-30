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
  HelpLevel,
  PatinaDiagnostic,
  PatinaRuleMeta,
  SfcBlock,
  SingleScriptMap,
} from "./model.js";
import { formatBlockLabel, getDiagnosticBlock } from "./sfc-blocks.js";
import { mapToScriptLoc } from "./script-map.js";
import { getActivePreset, getVizeSettings, isIncrementalPreset, isPatinaFile } from "./settings.js";

function createOxlintDiagnostic(
  diagnostic: PatinaDiagnostic,
  state: FileState,
  scriptMap: SingleScriptMap | null,
  helpLevel: HelpLevel,
): Diagnostic {
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

function shouldReportForCurrentProgram(
  diagnostic: PatinaDiagnostic,
  state: FileState,
  scriptMap: SingleScriptMap | null,
): boolean {
  if (state.usesOriginalLocations || scriptMap == null) {
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
    },
    createOnce(context) {
      return {
        Program() {
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
          const diagnostics = getDiagnosticsForRule(context, state, ruleMeta.name).filter(
            (diagnostic) => shouldReportForCurrentProgram(diagnostic, state, scriptMap),
          );
          if (diagnostics.length === 0) {
            return;
          }

          for (const diagnostic of diagnostics) {
            if (!markDiagnosticAsReported(state, diagnostic)) {
              continue;
            }

            context.report(createOxlintDiagnostic(diagnostic, state, scriptMap, helpLevel));
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
