import { definePlugin, defineRule, eslintCompatPlugin, type Context } from "@oxlint/plugins";
import { getPatinaRules } from "./binding.js";
import { isPatinaFile } from "./file-kinds.js";
import { formatPatinaMessage } from "./format.js";
import { loadBinding } from "./native.js";
import oxlintPlugin from "./plugin.js";
import { getRuleOptions } from "./rule-options.js";
import { createRuleSelection } from "./rule-selection.js";
import { getActivePreset, getVizeSettings, isTypeAwareRuleName } from "./settings.js";
import { assertSourceText, mapSourceOffsets, mapTextEdits } from "./eslint-source.js";
import type { PatinaDiagnostic, PatinaRuleOptions, PatinaSettings } from "./model.js";

interface Collected {
  source: string;
  sourceCode: Context["sourceCode"];
  filename: string;
  settings: PatinaSettings;
  options: Map<string, PatinaRuleOptions | undefined>;
  diagnostics?: Map<string, PatinaDiagnostic[]>;
  offsets?: Map<number, number>;
}

// ESLint calls every rule's Program before any Program:exit. The source code
// object is the authority, including an editor's unsaved text; never read disk.
const programs = new WeakMap<object, Collected>();
const metadata = getPatinaRules();
const byName = new Map(metadata.map((rule) => [rule.name, rule]));

function collect(context: Context, program: object, name: string): void {
  const source = context.sourceCode.text;
  let collected = programs.get(program);
  if (!collected || collected.diagnostics) {
    collected = {
      source,
      sourceCode: context.sourceCode,
      filename: context.physicalFilename,
      settings: structuredClone(getVizeSettings(context)),
      options: new Map(),
    };
    programs.set(program, collected);
  }
  if (source !== collected.source || context.physicalFilename !== collected.filename) {
    throw new Error("Vize source changed during ESLint rule collection");
  }
  collected.options.set(name, structuredClone(getRuleOptions(name, context.options)));
}

function run(collected: Collected): void {
  assertSourceText(collected.source);
  const selection = createRuleSelection(collected.options);
  const settings = collected.settings;
  const result = loadBinding().lintPatinaSfc(collected.source, {
    filename: collected.filename,
    locale: settings.locale,
    helpLevel: settings.helpLevel,
    preset: getActivePreset(settings),
    enabledRules: selection.names,
    typeAware: settings.typeAware === true || selection.names.some(isTypeAwareRuleName),
    corsaPath: settings.corsaPath,
    ...selection.options,
    includeFixes: true,
  });
  if (result.filename !== collected.filename) {
    throw new Error("Vize returned diagnostics for another ESLint source");
  }
  const diagnostics = new Map<string, PatinaDiagnostic[]>();
  const offsets: number[] = [];
  for (const diagnostic of result.diagnostics) {
    if (!selection.names.includes(diagnostic.rule)) {
      throw new Error(`Vize returned an unselected ESLint rule: ${diagnostic.rule}`);
    }
    if (byName.get(diagnostic.rule)?.fixable && !Object.hasOwn(diagnostic, "fix")) {
      throw new Error("The Vize native binding does not expose ESLint text edits; update it");
    }
    const current = diagnostics.get(diagnostic.rule) ?? [];
    current.push(diagnostic);
    diagnostics.set(diagnostic.rule, current);
    offsets.push(diagnostic.location.start.offset, diagnostic.location.end.offset);
    for (const edit of diagnostic.fix?.edits ?? []) offsets.push(edit.start, edit.end);
  }
  collected.offsets = mapSourceOffsets(collected.source, offsets);
  collected.diagnostics = diagnostics;
}

const rules = Object.fromEntries(
  metadata.map((rule) => [
    rule.name,
    defineRule({
      meta: {
        ...oxlintPlugin.rules[rule.name]!.meta,
        ...(rule.fixable ? { fixable: "code" as const } : {}),
      },
      createOnce(context) {
        let participated: object | undefined;
        return {
          before() {
            participated = undefined;
          },
          after() {
            if (participated) programs.delete(participated);
            participated = undefined;
          },
          Program(program) {
            if (!isPatinaFile(context.physicalFilename)) return;
            const preset = getActivePreset(getVizeSettings(context));
            if (
              preset !== "incremental" &&
              rule.presets.length > 0 &&
              !rule.presets.includes(preset)
            )
              return;
            collect(context, program, rule.name);
            participated = program;
          },
          "Program:exit"(program) {
            if (participated !== program) return;
            const collected = programs.get(program);
            if (!collected?.options.has(rule.name)) return;
            if (
              collected.source !== context.sourceCode.text ||
              collected.sourceCode !== context.sourceCode ||
              collected.filename !== context.physicalFilename ||
              JSON.stringify(collected.settings) !== JSON.stringify(getVizeSettings(context))
            ) {
              throw new Error("Vize source changed before ESLint rule execution");
            }
            if (!collected.diagnostics) run(collected);
            const sourceCode = context.sourceCode;
            for (const diagnostic of collected.diagnostics!.get(rule.name) ?? []) {
              const start = collected.offsets!.get(diagnostic.location.start.offset)!;
              const end = collected.offsets!.get(diagnostic.location.end.offset)!;
              if (start > end) throw new Error("Vize returned a reversed diagnostic span");
              const loc = {
                start: sourceCode.getLocFromIndex(start),
                end: sourceCode.getLocFromIndex(end),
              };
              if (
                loc.start.line !== diagnostic.location.start.line ||
                loc.start.column + 1 !== diagnostic.location.start.column ||
                loc.end.line !== diagnostic.location.end.line ||
                loc.end.column + 1 !== diagnostic.location.end.column
              )
                throw new Error("Vize returned inconsistent ESLint location metadata");
              const edits = diagnostic.fix
                ? mapTextEdits(diagnostic.fix.edits, collected.offsets!)
                : undefined;
              context.report({
                loc,
                message: formatPatinaMessage(diagnostic, {
                  hasMappedLocation: true,
                  blockLabel: null,
                  helpLevel: collected.settings.helpLevel ?? "full",
                }),
                ...(edits
                  ? {
                      fix(fixer) {
                        return edits.map((edit) => fixer.replaceTextRange(edit.range, edit.text));
                      },
                    }
                  : {}),
              });
            }
          },
        };
      },
    }),
  ]),
);

/** ESLint rules over the complete source supplied by the configured parser. */
export default eslintCompatPlugin(
  definePlugin({
    meta: { name: "oxlint-plugin-vize" },
    rules,
  }),
);
export { configs, createVizeRuleConfig } from "./configs.js";
