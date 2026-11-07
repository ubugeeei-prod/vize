import fs from "node:fs";

import type { Context } from "@oxlint/plugins";
import { lintPatina } from "./binding.js";
import {
  diagnosticsForRule,
  getFileState,
  indexDiagnosticsByRule,
  type FileState,
} from "./file-state.js";
import type { PatinaDiagnostic, PatinaRuleOptions, PatinaSettings } from "./model.js";
import { createRuleSelection, type RuleSelection } from "./rule-selection.js";
import { getVizeSettings, isTypeAwareRuleName } from "./settings.js";

interface ProgramCollection {
  state: FileState;
  physicalIdentity: PhysicalIdentity;
  runtimeSettings: PatinaSettings;
  optionsByRule: Map<string, PatinaRuleOptions | undefined>;
  selection?: RuleSelection;
  remaining?: Set<string>;
}

interface PhysicalIdentity {
  dev: bigint;
  ino: bigint;
  size: bigint;
  mtimeNs: bigint;
  ctimeNs: bigint;
}

// Both pinned hosts visit every active rule's Program before Program:exit.
// Weak keys keep this traversal state independent of the bounded file cache.
const programs = new WeakMap<object, ProgramCollection>();
const results = new WeakMap<
  FileState,
  { key: string; diagnostics: Map<string, PatinaDiagnostic[]> }
>();

export function collectActiveRule(
  context: Context,
  program: object,
  ruleName: string,
  options?: PatinaRuleOptions,
): void {
  let collection = programs.get(program);
  if (!collection || collection.selection) {
    const state = getFileState(context);
    collection = {
      state,
      physicalIdentity: physicalIdentity(context.physicalFilename),
      runtimeSettings: getVizeSettings(context),
      optionsByRule: new Map(),
    };
    programs.set(program, collection);
  }
  // An in-place options edit after registration must not mutate this pass.
  collection.optionsByRule.set(
    ruleName,
    options === undefined ? undefined : structuredClone(options),
  );
}

export function getActiveRuleDiagnostics(
  context: Context,
  program: object,
  ruleName: string,
): { state: FileState; diagnostics: readonly PatinaDiagnostic[] } | undefined {
  const collection = programs.get(program);
  if (!collection?.optionsByRule.has(ruleName)) return undefined;
  // Other plugins can change a physical file/settings between exit callbacks.
  // Preserve source authority through every report, including a cached pass.
  if (!hasCurrentProgramInputs(context, collection)) {
    programs.delete(program);
    throw new Error("Vize source or configuration changed before rule execution");
  }
  if (!collection.selection) {
    collection.selection = createRuleSelection(collection.optionsByRule);
    collection.remaining = new Set(collection.selection.names);
  }
  const { state, selection } = collection;
  let cached = results.get(state);
  if (cached?.key !== selection.cacheKey) {
    const settings = getVizeSettings(context);
    const diagnostics = indexDiagnosticsByRule(
      lintPatina(
        state.source,
        state.filename,
        {
          ...settings,
          typeAware: settings.typeAware === true || selection.names.some(isTypeAwareRuleName),
        },
        selection.names,
        selection.options,
      ).diagnostics,
    );
    if (cached) {
      state.reportedDiagnostics.clear();
      state.reportedTypeAwareRuntimeDiagnostic = false;
    }
    cached = { key: selection.cacheKey, diagnostics };
    results.set(state, cached);
  }
  const diagnostics = diagnosticsForRule(state, cached.diagnostics, ruleName);
  collection.remaining!.delete(ruleName);
  if (collection.remaining!.size === 0) programs.delete(program);
  return { state, diagnostics };
}

function physicalIdentity(filename: string): PhysicalIdentity {
  const stat = fs.statSync(filename, { bigint: true });
  return {
    dev: stat.dev,
    ino: stat.ino,
    size: stat.size,
    mtimeNs: stat.mtimeNs,
    ctimeNs: stat.ctimeNs,
  };
}

function hasCurrentProgramInputs(context: Context, collection: ProgramCollection): boolean {
  const currentIdentity = physicalIdentity(context.physicalFilename);
  const expectedIdentity = collection.physicalIdentity;
  if (
    currentIdentity.dev !== expectedIdentity.dev ||
    currentIdentity.ino !== expectedIdentity.ino ||
    currentIdentity.size !== expectedIdentity.size ||
    currentIdentity.mtimeNs !== expectedIdentity.mtimeNs ||
    currentIdentity.ctimeNs !== expectedIdentity.ctimeNs ||
    context.sourceCode.text !== collection.state.extractedScript
  ) {
    return false;
  }

  const current = getVizeSettings(context);
  const expected = collection.runtimeSettings;
  return (
    current.locale === expected.locale &&
    current.helpLevel === expected.helpLevel &&
    current.preset === expected.preset &&
    current.typeAware === expected.typeAware &&
    current.corsaPath === expected.corsaPath
  );
}
