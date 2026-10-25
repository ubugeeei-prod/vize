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
  function span(file, value, feature, edit = false) {
    if (!value) return value;
    const document = get(file);
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
  function positions(file, position, feature) {
    const document = get(file);
    if (!document) return [position];
    return [
      ...new Set(
        (document.projection?.mappings || [])
          .filter(
            (row) =>
              row[5] & feature &&
              (feature !== Feature.completion || (row[4] === 0 && row[1] === row[3])) &&
              position >= row[2] &&
              position <= row[2] + row[3],
          )
          .map((row) => (row[4] === 0 ? row[0] + position - row[2] : row[0])),
      ),
    ];
  }
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
      return combine
        ? unique(results.flatMap((result) => result || []))
        : results.find((result) => result !== undefined);
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
      // EXPECT applies to the complete syntax+semantic set, never a per-packet subset.
      const all = [
        ...service.getSyntacticDiagnostics(file),
        ...service.getSemanticDiagnostics(file),
      ];
      const applied = support.engine.applyDiagnosticDirectives(document.projection, all);
      const current =
        method === "getSuggestionDiagnostics" ? service[method](file) : applied.diagnostics;
      const mapped = current.flatMap((value) => {
        const textSpan = span(file, { start: value.start ?? 0, length: value.length ?? 0 });
        if (!textSpan && [6133, 6192, 6196].includes(value.code)) return [];
        const result = { ...value, file: sourceFile, ...(textSpan || { start: 0, length: 0 }) };
        if (!textSpan)
          result.messageText = `In generated Vize code: ${ts.flattenDiagnosticMessageText(value.messageText, "\n")}`;
        if (value.relatedInformation)
          result.relatedInformation = relatedInformation(value.relatedInformation);
        return [result];
      });
      // Return compiler syntax+semantic diagnostics once, in the semantic lane.
      return method === "getSyntacticDiagnostics"
        ? []
        : unique([
            ...mapped,
            ...(method === "getSemanticDiagnostics"
              ? applied.sourceDiagnostics.map(diagnostic)
              : []),
          ]);
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
module.exports = { createMappedLanguageService };
