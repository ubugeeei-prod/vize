"use strict";
const { isSfc } = require("./virtual-modules.cjs");

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
  for (const key of Object.keys(service))
    proxy[key] = typeof service[key] === "function" ? service[key].bind(service) : service[key];
  const get = (file) => support?.get(file);
  proxy.dispose = () => {
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
      const document = get(file);
      if (!document)
        return service[method](file).map((value) => ({
          ...value,
          ...(value.relatedInformation
            ? { relatedInformation: relatedInformation(value.relatedInformation) }
            : {}),
        }));
      const sourceFile = ts.createSourceFile(file, document.source, ts.ScriptTarget.Latest);
      const diagnostic = (value) => ({
        ...value,
        file: sourceFile,
        category: ts.DiagnosticCategory.Error,
      });
      if (document.error)
        return method === "getSemanticDiagnostics"
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
      let applied, current;
      // Suggestions bypass directives, so they never pay for another semantic check.
      if (method === "getSuggestionDiagnostics") current = service[method](file);
      else {
        // EXPECT applies to the complete syntax+semantic set, never a per-packet subset.
        const syntax = service.getSyntacticDiagnostics(file);
        const semantic = service.getSemanticDiagnostics(file);
        applied = support.engine.applyDiagnosticDirectives(document.projection, [
          ...syntax,
          ...semantic,
        ]);
        const channel = new Set(method === "getSyntacticDiagnostics" ? syntax : semantic);
        current = applied.diagnostics.filter((value) => channel.has(value));
      }
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
      const nativeSyntax = new Set(document.projection.diagnostics);
      const authored = (applied?.sourceDiagnostics ?? []).filter((value) =>
        method === "getSyntacticDiagnostics" ? nativeSyntax.has(value) : !nativeSyntax.has(value),
      );
      return unique([...mapped, ...authored.map(diagnostic)]);
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
      proxy[method] = (file, ...args) =>
        isSfc(file) ? { spans: [], endOfLineState: 0 } : service[method](file, ...args);
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
  const document = support?.get(file);
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
