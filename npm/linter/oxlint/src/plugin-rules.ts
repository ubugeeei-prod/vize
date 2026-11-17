// Plugin lint rules from `vue.plugins` (#13074), hosted inside Oxlint.
//
// `settings.vize.pluginRules: true` runs the plugins of the nearest
// `vite.config.*` above each Vue file, as `vize lint` would. One native
// session per config owns one sidecar for the whole Oxlint process; each Vue file crosses into it once, and both rules
// read that one answer. Severity stays the host's: `plugin-errors` reports the
// plugin diagnostics Vize resolved as errors and `plugin-warnings` the rest.
// A session or rule failure is reported on the file instead of skipped.
import fs from "node:fs";
import path from "node:path";

import { defineRule } from "@oxlint/plugins";
import type { Context } from "@oxlint/plugins";

import { getFileState, getScriptMap, markDiagnosticAsReported } from "./file-state.js";
import type { FileState } from "./file-state.js";
import type { PatinaDiagnostic, PatinaLintResult } from "./model.js";
import { loadBinding } from "./native.js";
import { createOxlintDiagnostic, shouldReportForCurrentProgram } from "./report.js";
import { getVizeSettings } from "./settings.js";

interface PluginLintSession {
  lint(source: string, filename: string): PatinaLintResult;
}

type Answer = readonly PatinaDiagnostic[] | Error;

const VITE_CONFIG_FILE_NAMES = [
  "vite.config.ts",
  "vite.config.js",
  "vite.config.mjs",
  "vite.config.cjs",
  "vite.config.mts",
  "vite.config.cts",
];

const configByDirectory = new Map<string, string | null>();
const sessions = new Map<string, PluginLintSession | Error>();
const answers = new WeakMap<FileState, Answer>();
const reportedFailures = new WeakSet<FileState>();

function pluginConfigFile(context: Context): string | undefined {
  const settings = context.settings as { vize?: { pluginRules?: unknown } };
  if (settings.vize?.pluginRules !== true) return undefined;
  return nearestViteConfig(path.dirname(path.resolve(context.cwd, context.physicalFilename)));
}

function nearestViteConfig(directory: string): string | undefined {
  const known = configByDirectory.get(directory);
  if (known !== undefined) return known ?? undefined;
  const found = VITE_CONFIG_FILE_NAMES.map((name) => path.join(directory, name)).find((file) =>
    fs.existsSync(file),
  );
  const parent = path.dirname(directory);
  const resolved = found ?? (parent === directory ? undefined : nearestViteConfig(parent));
  configByDirectory.set(directory, resolved ?? null);
  return resolved;
}

function session(configFile: string): PluginLintSession | Error {
  const existing = sessions.get(configFile);
  if (existing !== undefined) return existing;
  let created: PluginLintSession | Error;
  try {
    const { PluginLintSession: Session } = loadBinding() as unknown as {
      PluginLintSession: new (configFile: string) => PluginLintSession;
    };
    created = new Session(configFile);
  } catch (error) {
    created = error instanceof Error ? error : new Error(String(error));
  }
  sessions.set(configFile, created);
  return created;
}

function answer(state: FileState, configFile: string): Answer {
  const existing = answers.get(state);
  if (existing !== undefined) return existing;
  const host = session(configFile);
  let found: Answer;
  if (host instanceof Error) {
    found = host;
  } else {
    try {
      found = host.lint(state.source, state.filename).diagnostics;
    } catch (error) {
      found = error instanceof Error ? error : new Error(String(error));
    }
  }
  answers.set(state, found);
  return found;
}

function createPluginRule(severity: PatinaDiagnostic["severity"]) {
  return defineRule({
    meta: {
      type: severity === "error" ? "problem" : "suggestion",
      docs: {
        description:
          severity === "error"
            ? "Report `vue.plugins` lint rules configured as errors"
            : "Report `vue.plugins` lint rules configured as warnings",
      },
    },
    createOnce(context) {
      return {
        "Program:exit"(program) {
          const configFile = pluginConfigFile(context);
          if (configFile === undefined || !context.filename.endsWith(".vue")) return;
          const state = getFileState(context);
          const found = answer(state, configFile);
          if (found instanceof Error) {
            // One visible failure per file, on the error rule.
            if (severity === "error" && !reportedFailures.has(state)) {
              reportedFailures.add(state);
              context.report({ node: program, message: found.message });
            }
            return;
          }
          const scriptMap = getScriptMap(state);
          const helpLevel = getVizeSettings(context).helpLevel ?? "full";
          for (const diagnostic of found) {
            if (diagnostic.severity !== severity) continue;
            if (!shouldReportForCurrentProgram(diagnostic, state, scriptMap)) continue;
            if (!markDiagnosticAsReported(state, diagnostic)) continue;
            // One Oxlint rule carries every plugin rule, so the message names it.
            const named = { ...diagnostic, message: `[${diagnostic.rule}] ${diagnostic.message}` };
            context.report(createOxlintDiagnostic(named, state, scriptMap, helpLevel, program));
          }
        },
      };
    },
  });
}

export const pluginRules = {
  "plugin-errors": createPluginRule("error"),
  "plugin-warnings": createPluginRule("warning"),
};
