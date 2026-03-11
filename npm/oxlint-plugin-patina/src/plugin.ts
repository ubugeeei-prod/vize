import { definePlugin, defineRule, type Diagnostic } from "@oxlint/plugins";

import { getPatinaRules } from "./binding.js";
import {
  getFileState,
  getDiagnosticsForRule,
  getScriptMap,
  type FileState,
} from "./file-state.js";
import { formatPatinaMessage } from "./format.js";
import type { PatinaDiagnostic, PatinaRuleMeta } from "./model.js";
import { createSourceSnippet, mapToScriptLoc } from "./script-map.js";
import { isVueLikeFile } from "./settings.js";

function createOxlintDiagnostic(
  diagnostic: PatinaDiagnostic,
  state: FileState,
): Diagnostic {
  const scriptMap = getScriptMap(state);
  const loc = mapToScriptLoc(diagnostic, scriptMap);
  const message = formatPatinaMessage(
    diagnostic,
    loc !== null,
    loc ? null : createSourceSnippet(state.sourceLines, diagnostic),
  );

  return {
    loc: loc ?? {
      start: { line: 1, column: 1 },
      end: { line: 1, column: 1 },
    },
    message,
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

          const state = getFileState(context);
          const diagnostics = getDiagnosticsForRule(context, state, ruleMeta.name);
          if (diagnostics.length === 0) {
            return;
          }

          for (const diagnostic of diagnostics) {
            context.report(createOxlintDiagnostic(diagnostic, state));
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
    name: "patina",
  },
  rules: patinaRules,
});
