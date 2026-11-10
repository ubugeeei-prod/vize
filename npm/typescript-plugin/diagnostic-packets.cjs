"use strict";
const fs = require("node:fs");
const path = require("node:path");

const diagnosticProfiles = new Map();
let diagnosticProfileExitHook = false;

/**
 * Benchmark-only phase evidence. Production pays no clock or allocation cost
 * unless the harness supplies an output path.
 */
function createDiagnosticRecorder() {
  const output = process.env.VIZE_TYPESCRIPT_PLUGIN_DIAGNOSTIC_PROFILE;
  if (!output) return undefined;
  let profile = diagnosticProfiles.get(output);
  if (!profile) {
    profile = { contract: "vize-typescript-plugin-diagnostics-v1", pid: process.pid, events: [] };
    diagnosticProfiles.set(output, profile);
  }
  const flush = () => {
    try {
      fs.mkdirSync(path.dirname(output), { recursive: true });
      fs.writeFileSync(output, `${JSON.stringify(profile, null, 2)}\n`);
    } catch {
      // Evidence capture must never change editor behavior.
    }
  };
  if (!diagnosticProfileExitHook) {
    diagnosticProfileExitHook = true;
    process.once("exit", () => {
      for (const [file, value] of diagnosticProfiles) {
        try {
          fs.mkdirSync(path.dirname(file), { recursive: true });
          fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
        } catch {
          // Evidence capture must never change editor behavior.
        }
      }
    });
  }
  const record = (event) => profile.events.push(event);
  record.flush = flush;
  return record;
}

function diagnosticPhaseTimer(record, method, file) {
  if (!record) {
    return {
      measure(_phase, call) {
        return call();
      },
      finish() {},
    };
  }
  const started = performance.now();
  const phasesMs = {};
  return {
    measure(phase, call) {
      const phaseStarted = performance.now();
      try {
        return call();
      } finally {
        phasesMs[phase] = (phasesMs[phase] || 0) + performance.now() - phaseStarted;
      }
    },
    finish(details) {
      record({
        atMs: started,
        method,
        file,
        totalMs: performance.now() - started,
        phasesMs,
        ...details,
      });
    },
  };
}

/**
 * Authored diagnostics for tsserver's three diagnostic requests. The syntactic
 * and semantic packets of one SFC share one complete check, directive pass and
 * mapping per project version; any project-version or projection change starts
 * a new pass. Suggestions stay independent of that packet.
 */
function createMappedDiagnostics(ts, service, support, relatedInformation, unique) {
  const recordDiagnostics = createDiagnosticRecorder();
  const diagnosticPackets = new Map();
  const get = (file) => support?.get(file);
  const methods = Object.create(null);
  for (const method of [
    "getSyntacticDiagnostics",
    "getSemanticDiagnostics",
    "getSuggestionDiagnostics",
  ]) {
    methods[method] = (file) => {
      const timer = diagnosticPhaseTimer(recordDiagnostics, method, file);
      timer.measure("ensure", () => support?.ensure?.(file));
      const document = timer.measure("projection", () => get(file));
      if (!document) {
        const result = timer
          .measure("typescript", () => service[method](file))
          .map((value) => ({
            ...value,
            ...(value.relatedInformation
              ? { relatedInformation: relatedInformation(value.relatedInformation) }
              : {}),
          }));
        timer.finish({ projected: false, resultCount: result.length });
        return result;
      }
      const sourceFile = timer.measure(
        "authoredSource",
        () =>
          (document.authoredFile ??= ts.createSourceFile(
            file,
            document.source,
            ts.ScriptTarget.Latest,
          )),
      );
      const diagnostic = (value) => ({
        ...value,
        file: sourceFile,
        category: ts.DiagnosticCategory.Error,
      });
      if (document.error) {
        const result =
          method === "getSemanticDiagnostics"
            ? [
                diagnostic({
                  start: 0,
                  length: 0,
                  code: 100003,
                  source: "vize",
                  messageText: document.error,
                }),
              ]
            : [];
        timer.finish({
          projected: true,
          projectionError: true,
          sourceLength: document.source.length,
          resultCount: result.length,
        });
        return result;
      }
      // Suggestions bypass directives, so they never pay for another semantic check.
      if (method === "getSuggestionDiagnostics") {
        const current = timer.measure("typescriptSuggestion", () => service[method](file));
        const result = timer.measure("mapping", () => mapDiagnostics(current, []));
        timer.finish({
          projected: true,
          sourceLength: document.source.length,
          generatedLength: document.projection.text.length,
          resultCount: result.length,
        });
        return result;
      }

      const projectVersion = timer.measure("version", () => support?.projectVersion?.());
      const cached = projectVersion === undefined ? undefined : diagnosticPackets.get(file);
      if (
        cached &&
        cached.projectVersion === projectVersion &&
        cached.document === document &&
        cached.packets[method]
      ) {
        const result = cached.packets[method];
        timer.finish({
          projected: true,
          cacheHit: true,
          projectVersion,
          sourceLength: document.source.length,
          generatedLength: document.projection.text.length,
          resultCount: result.length,
        });
        return result;
      }

      // EXPECT applies to the complete syntax+semantic set, never a per-packet subset.
      const syntax = timer.measure("typescriptSyntactic", () =>
        service.getSyntacticDiagnostics(file),
      );
      const semantic = timer.measure("typescriptSemantic", () =>
        service.getSemanticDiagnostics(file),
      );
      const applied = timer.measure("directives", () =>
        support.engine.applyDiagnosticDirectives(document.projection, [...syntax, ...semantic]),
      );
      const packets = timer.measure("mapping", () => {
        const nativeSyntax = new Set(document.projection.diagnostics);
        const sourceDiagnostics = applied.sourceDiagnostics ?? [];
        const sourceSyntax = sourceDiagnostics.filter((value) => nativeSyntax.has(value));
        const sourceSemantic = sourceDiagnostics.filter((value) => !nativeSyntax.has(value));
        const syntaxChannel = new Set(syntax);
        const semanticChannel = new Set(semantic);
        return {
          getSyntacticDiagnostics: mapDiagnostics(
            applied.diagnostics.filter((value) => syntaxChannel.has(value)),
            sourceSyntax,
          ),
          getSemanticDiagnostics: mapDiagnostics(
            applied.diagnostics.filter((value) => semanticChannel.has(value)),
            sourceSemantic,
          ),
        };
      });
      const completedVersion = timer.measure("completedVersion", () => support?.projectVersion?.());
      if (completedVersion !== undefined)
        diagnosticPackets.set(file, { projectVersion: completedVersion, document, packets });
      const result = packets[method];
      timer.finish({
        projected: true,
        cacheHit: false,
        projectVersion: completedVersion,
        sourceLength: document.source.length,
        generatedLength: document.projection.text.length,
        resultCount: result.length,
      });
      return result;

      function mapDiagnostics(current, authored) {
        const mapped = current.flatMap((value) => {
          const range = support.engine.mapLanguageDiagnosticRange(
            document.projection,
            value.start ?? 0,
            value.length ?? 0,
          );
          const textSpan =
            range.fidelity === "none" ? undefined : { start: range.start, length: range.length };
          if (!textSpan && [6133, 6192, 6196].includes(value.code)) return [];
          const result = { ...value, file: sourceFile, ...(textSpan || { start: 0, length: 0 }) };
          if (!textSpan)
            result.messageText = `In generated Vize code: ${ts.flattenDiagnosticMessageText(value.messageText, "\n")}`;
          if (value.relatedInformation)
            result.relatedInformation = relatedInformation(value.relatedInformation);
          return [result];
        });
        return unique([...mapped, ...authored.map(diagnostic)]);
      }
    };
  }
  return {
    methods,
    dispose() {
      recordDiagnostics?.flush();
      diagnosticPackets.clear();
    },
  };
}

module.exports = { createMappedDiagnostics };
