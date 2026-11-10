"use strict";
const fs = require("node:fs");
const path = require("node:path");
const { isSfc } = require("./virtual-modules.cjs");

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

function createMappedLanguageService(ts, service, support) {
  const shared = support?.engine?.LanguageFeature || {};
  const Feature = {
    hover: shared.Hover,
    completion: shared.Completion,
    definition: shared.Definition,
    typeDefinition: shared.TypeDefinition,
    implementation: shared.Implementation,
    references: shared.References,
    rename: shared.Rename,
    codeActions: shared.CodeActions,
  };
  const proxy = Object.create(null);
  const recordDiagnostics = createDiagnosticRecorder();
  const diagnosticPackets = new Map();
  for (const key of Object.keys(service))
    proxy[key] = typeof service[key] === "function" ? service[key].bind(service) : service[key];
  const get = (file) => support?.get(file);
  proxy.dispose = () => {
    recordDiagnostics?.flush();
    diagnosticPackets.clear();
    support?.dispose?.();
    service.dispose?.();
  };
  // tsserver uses this method after receiving mapped numeric spans. Its
  // default implementation reads the generated Program's line map again.
  if (service.toLineColumnOffset)
    proxy.toLineColumnOffset = (file, position) => {
      const document = get(file);
      if (!document) return service.toLineColumnOffset(file, position);
      document.authoredFile ??= ts.createSourceFile(file, document.source, ts.ScriptTarget.Latest);
      return ts.getLineAndCharacterOfPosition(document.authoredFile, position);
    };
  const span = (file, value, feature, edit = false) =>
    authoredSpan(support, file, value, feature, edit);
  const positions = (file, position, feature) =>
    generatedPositions(support, file, position, feature);
  function location(value, feature, edit = false) {
    const textSpan = span(value.fileName, value.textSpan, feature, edit);
    if (!textSpan) return undefined;
    const result = { ...value, textSpan };
    for (const key of ["contextSpan", "originalTextSpan", "originalContextSpan"]) {
      if (result[key])
        result[key] = span(result.originalFileName || result.fileName, result[key], feature, edit);
    }
    return result;
  }
  function relatedInformation(values) {
    return values?.map((related) => {
      const relatedDocument = related.file && get(related.file.fileName);
      if (!relatedDocument) return related;
      const mapped = span(related.file.fileName, {
        start: related.start ?? 0,
        length: related.length ?? 0,
      });
      return {
        ...related,
        file: ts.createSourceFile(
          related.file.fileName,
          relatedDocument.source,
          ts.ScriptTarget.Latest,
        ),
        ...(mapped || { start: 0, length: 0 }),
      };
    });
  }
  function query(method, feature, map, combine = false) {
    if (typeof service[method] !== "function") return;
    proxy[method] = (file, position, ...args) => {
      support?.ensure?.(file);
      const results = positions(file, position, feature).map((offset) =>
        map(service[method](file, offset, ...args), file),
      );
      if (combine) return unique(results.flatMap((result) => result || []));
      return (
        results.find((result) => result !== undefined) ??
        (method === "getRenameInfo"
          ? {
              canRename: false,
              localizedErrorMessage: "This source range has no safe rename projection",
            }
          : undefined)
      );
    };
  }
  const locations =
    (feature, edit = false) =>
    (values) =>
      values?.flatMap((value) => location(value, feature, edit) || []);
  query("getQuickInfoAtPosition", Feature.hover, (value, file) => {
    if (!value) return undefined;
    const textSpan = span(file, value.textSpan, Feature.hover);
    return textSpan && { ...value, textSpan };
  });
  for (const [method, feature] of [
    ["getDefinitionAtPosition", Feature.definition],
    ["getTypeDefinitionAtPosition", Feature.typeDefinition],
    ["getImplementationAtPosition", Feature.implementation],
    ["getReferencesAtPosition", Feature.references],
    ["findRenameLocations", Feature.rename],
  ]) {
    query(method, feature, locations(feature, feature === Feature.rename), true);
  }
  if (service.getFileReferences)
    proxy.getFileReferences = (file) =>
      locations(Feature.references)(service.getFileReferences(file));
  query("getDefinitionAndBoundSpan", Feature.definition, (value, file) => {
    if (!value) return undefined;
    const textSpan = span(file, value.textSpan, Feature.definition);
    return (
      textSpan && {
        ...value,
        textSpan,
        definitions: locations(Feature.definition)(value.definitions),
      }
    );
  });
  query(
    "findReferences",
    Feature.references,
    (values) =>
      values?.flatMap((value) => {
        const definition = location(value.definition, Feature.references);
        return definition
          ? [{ definition, references: locations(Feature.references)(value.references) }]
          : [];
      }),
    true,
  );
  query("getRenameInfo", Feature.rename, (value, file) => {
    if (!value?.canRename) return value;
    const triggerSpan = span(file, value.triggerSpan, Feature.rename, true);
    return triggerSpan
      ? { ...value, triggerSpan }
      : { canRename: false, localizedErrorMessage: "This generated range is not safe to edit" };
  });
  query("getCompletionsAtPosition", Feature.completion, (value, file) => {
    if (!value) return undefined;
    const entries = value.entries
      .filter((entry) => !/^__vize/i.test(entry.name))
      .flatMap((entry) => {
        const replacementSpan = span(file, entry.replacementSpan, Feature.completion, true);
        return entry.replacementSpan && !replacementSpan
          ? []
          : [{ ...entry, ...(replacementSpan ? { replacementSpan } : {}) }];
      });
    const optionalReplacementSpan = span(
      file,
      value.optionalReplacementSpan,
      Feature.completion,
      true,
    );
    if (value.optionalReplacementSpan && !optionalReplacementSpan) return undefined;
    return { ...value, entries, optionalReplacementSpan };
  });
  function changes(values, feature) {
    const result = [];
    for (const value of values || []) {
      const textChanges = value.textChanges.map((change) => ({
        ...change,
        span: span(value.fileName, change.span, feature, true),
      }));
      // Applying half an action can corrupt a document: reject the whole action.
      if (textChanges.some((change) => !change.span)) return undefined;
      result.push({ ...value, textChanges });
    }
    return result;
  }
  query("getCompletionEntryDetails", Feature.completion, (value) => {
    if (!value) return undefined;
    const codeActions = [];
    for (const action of value.codeActions || []) {
      const mapped = changes(action.changes, Feature.completion);
      if (mapped) codeActions.push({ ...action, changes: mapped });
    }
    return { ...value, ...(value.codeActions ? { codeActions } : {}) };
  });
  for (const method of [
    "getSyntacticDiagnostics",
    "getSemanticDiagnostics",
    "getSuggestionDiagnostics",
  ]) {
    proxy[method] = (file) => {
      const timer = diagnosticPhaseTimer(recordDiagnostics, method, file);
      timer.measure("ensure", () => support?.ensure?.(file));
      const document = timer.measure("projection", () => get(file));
      if (!document) {
        const result = timer.measure("typescript", () => service[method](file)).map((value) => ({
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
      const completedVersion = timer.measure("completedVersion", () =>
        support?.projectVersion?.(),
      );
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
  // Methods without an authored-range adapter must never expose generated
  // offsets or edits. Clients can use Vize's LSP for these additional features.
  for (const method of [
    "getFormattingEditsForDocument",
    "getFormattingEditsForRange",
    "getFormattingEditsAfterKeystroke",
    "getNavigationBarItems",
    "getOutliningSpans",
    "getTodoComments",
    "getBraceMatchingAtPosition",
    "getApplicableRefactors",
    "getDocumentHighlights",
    "getCodeFixesAtPosition",
  ]) {
    if (typeof service[method] !== "function") continue;
    proxy[method] = (file, ...args) => {
      if (isSfc(file)) return [];
      const result = service[method](file, ...args);
      return method === "getCodeFixesAtPosition"
        ? result.flatMap((action) => {
            const mapped = changes(action.changes, Feature.codeActions);
            return mapped ? [{ ...action, changes: mapped }] : [];
          })
        : result;
    };
  }
  for (const method of [
    "getSignatureHelpItems",
    "getSmartSelectionRange",
    "prepareCallHierarchy",
    "provideInlayHints",
    "getEditsForRefactor",
    "getJsxClosingTagAtPosition",
    "getDocCommentTemplateAtPosition",
    "getSpanOfEnclosingComment",
    "getBreakpointStatementAtPosition",
    "getNameOrDottedNameSpan",
    "getLinkedEditingRangeAtPosition",
  ]) {
    if (typeof service[method] !== "function") continue;
    proxy[method] = (file, ...args) => {
      if (isSfc(file)) return method === "provideInlayHints" ? [] : undefined;
      const result = service[method](file, ...args);
      if (method !== "getEditsForRefactor" || !result) return result;
      const edits = changes(result.edits, Feature.codeActions);
      return edits ? { ...result, edits } : undefined;
    };
  }
  // Preparation is deliberately unsupported for SFCs. Follow-up requests can
  // arrive directly, including cross-file results queried from ordinary TS.
  for (const method of ["provideCallHierarchyIncomingCalls", "provideCallHierarchyOutgoingCalls"]) {
    if (!service[method]) continue;
    proxy[method] = (file, ...args) => {
      if (isSfc(file)) return [];
      return service[method](file, ...args).filter((call) => !isSfc((call.from || call.to)?.file));
    };
  }
  if (service.organizeImports)
    proxy.organizeImports = (scope, ...args) =>
      isSfc(scope.fileName)
        ? []
        : changes(service.organizeImports(scope, ...args), Feature.codeActions) || [];
  if (service.getCombinedCodeFix)
    proxy.getCombinedCodeFix = (scope, ...args) => {
      if (isSfc(scope.fileName)) return { changes: [] };
      const result = service.getCombinedCodeFix(scope, ...args);
      const mapped = changes(result.changes, Feature.codeActions);
      return mapped ? { ...result, changes: mapped } : { changes: [] };
    };
  if (service.getEditsForFileRename)
    proxy.getEditsForFileRename = (...args) =>
      changes(service.getEditsForFileRename(...args), Feature.codeActions) || [];
  if (service.getNavigateToItems)
    proxy.getNavigateToItems = (...args) =>
      locations(shared.DocumentSymbols)(service.getNavigateToItems(...args));
  if (service.getNavigationTree)
    proxy.getNavigationTree = (file) =>
      isSfc(file)
        ? { text: "", kind: ts.ScriptElementKind.moduleElement, kindModifiers: "", spans: [] }
        : service.getNavigationTree(file);
  for (const method of [
    "getEncodedSyntacticClassifications",
    "getEncodedSemanticClassifications",
  ]) {
    if (service[method])
      proxy[method] = (file, range, ...args) => {
        if (!isSfc(file)) return service[method](file, range, ...args);
        support?.ensure?.(file);
        const document = get(file);
        const feature = shared.SemanticTokens;
        const end = range.start + range.length;
        let generatedStart = Infinity,
          generatedEnd = 0;
        for (const row of document?.projection?.mappings || []) {
          if (!(row[5] & feature)) continue;
          const start = Math.max(range.start, row[2]);
          const stop = Math.min(end, row[2] + row[3]);
          if (start >= stop) continue;
          generatedStart = Math.min(
            generatedStart,
            row[4] === 0 ? row[0] + start - row[2] : row[0],
          );
          generatedEnd = Math.max(
            generatedEnd,
            row[4] === 0 ? row[0] + stop - row[2] : row[0] + row[1],
          );
        }
        if (generatedStart === Infinity)
          return { spans: [], endOfLineState: ts.EndOfLineState.None };
        const result = service[method](
          file,
          { start: generatedStart, length: generatedEnd - generatedStart },
          ...args,
        );
        const classified = [];
        for (let index = 0; index < result.spans.length; index += 3) {
          const mapped = authoredProjectionSpan(
            document,
            {
              start: result.spans[index],
              length: result.spans[index + 1],
            },
            feature,
          );
          if (!mapped) continue;
          const start = Math.max(range.start, mapped.start);
          const stop = Math.min(end, mapped.start + mapped.length);
          if (start < stop) classified.push([start, stop - start, result.spans[index + 2]]);
        }
        classified.sort((left, right) => left[0] - right[0]);
        return { ...result, spans: unique(classified).flat() };
      };
  }
  return proxy;
}
function unique(values) {
  const seen = new Set();
  return values.filter((value) => {
    const key = JSON.stringify(value, (name, item) => (name === "file" ? item?.fileName : item));
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}
function authoredSpan(support, file, value, feature, edit = false) {
  if (!value) return value;
  return authoredProjectionSpan(support?.get(file), value, feature, edit);
}
function authoredProjectionSpan(document, value, feature, edit = false) {
  if (!document) return value;
  if (!document.projection) return undefined;
  const rows = document.projection.mappings.filter(
    (row) =>
      (!feature || row[5] & feature) &&
      value.start >= row[0] &&
      value.start + value.length <= row[0] + row[1] &&
      (!edit ||
        (row[4] === 0 &&
          row[1] === row[3] &&
          document.projection.text.slice(row[0], row[0] + row[1]) ===
            document.source.slice(row[2], row[2] + row[3]))),
  );
  rows.sort((a, b) => a[1] - b[1]);
  const row = rows[0];
  if (!row) return undefined;
  return row[4] === 0
    ? { start: row[2] + value.start - row[0], length: value.length }
    : { start: row[2], length: row[3] };
}
function generatedPositions(support, file, position, feature) {
  const document = support?.get(file);
  if (!document) return [position];
  return [
    ...new Set(
      (document.projection?.mappings || [])
        .filter(
          (row) =>
            row[5] & feature &&
            (feature !== support?.engine?.LanguageFeature?.Completion ||
              (row[4] === 0 && row[1] === row[3])) &&
            position >= row[2] &&
            position <= row[2] + row[3],
        )
        .map((row) => (row[4] === 0 ? row[0] + position - row[2] : row[0])),
    ),
  ];
}
module.exports = { createMappedLanguageService, authoredSpan, generatedPositions };
