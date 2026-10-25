import { createLanguage } from "@volar/language-core";
import { createProxyLanguageService, decorateLanguageServiceHost } from "@volar/typescript";
import { applyDiagnosticDirectives } from "@vizejs/native/language-projection.js";
import { createVizeLanguagePlugin, VizeVirtualCode } from "./language-plugin.js";

/** Compose the actual Volar context and TypeScript service, including mapper diagnostics. */
export function createVizeLanguageService(ts, host, options = {}) {
  const plugin = createVizeLanguagePlugin(ts, options);
  const registry = new Map();
  const originalSnapshot = host.getScriptSnapshot.bind(host);
  const language = createLanguage([plugin], registry, (id) => {
    const snapshot = originalSnapshot(id);
    if (snapshot) language.scripts.set(id, snapshot);
    else language.scripts.delete(id);
  });
  const adapterHost = Object.fromEntries(
    Object.entries(host).map(([key, value]) => [
      key,
      typeof value === "function" ? value.bind(host) : value,
    ]),
  );
  adapterHost.getScriptSnapshot = originalSnapshot;
  adapterHost.getScriptKind ??= () => ts.ScriptKind.Unknown;
  adapterHost.resolveModuleNameLiterals ??= (
    literals,
    containingFile,
    redirectedReference,
    compilerOptions,
  ) =>
    literals.map((literal) =>
      ts.resolveModuleName(
        literal.text,
        containingFile,
        compilerOptions,
        adapterHost,
        undefined,
        redirectedReference,
      ),
    );
  decorateLanguageServiceHost(ts, language, adapterHost);
  const rawService = ts.createLanguageService(adapterHost);
  const diagnosticCache = new WeakMap();

  function rootFor(fileName) {
    const root = language.scripts.get(fileName)?.generated?.root;
    return root instanceof VizeVirtualCode ? root : undefined;
  }
  function diagnosticsFor(fileName) {
    const program = rawService.getProgram();
    const root = rootFor(fileName);
    if (!program || !root) return;
    let cached = diagnosticCache.get(program);
    if (!cached) diagnosticCache.set(program, (cached = new Map()));
    if (!cached.has(fileName)) {
      const syntactic = rawService.getSyntacticDiagnostics(fileName);
      const semantic = rawService.getSemanticDiagnostics(fileName);
      const filtered = applyDiagnosticDirectives(root.projection, [...syntactic, ...semantic]);
      const kept = new Set(filtered.diagnostics);
      cached.set(fileName, {
        syntactic: syntactic.filter((d) => kept.has(d)),
        semantic: semantic.filter((d) => kept.has(d)),
        sourceDiagnostics: filtered.sourceDiagnostics,
      });
    }
    return cached.get(fileName);
  }
  const filteringService = new Proxy(rawService, {
    get(target, key) {
      if (key === "getSyntacticDiagnostics" || key === "getSemanticDiagnostics") {
        return (fileName) =>
          diagnosticsFor(fileName)?.[
            key === "getSyntacticDiagnostics" ? "syntactic" : "semantic"
          ] ?? target[key](fileName);
      }
      const value = target[key];
      return typeof value === "function" ? value.bind(target) : value;
    },
  });
  const { proxy, initialize } = createProxyLanguageService(filteringService);
  initialize(language);

  function sourceFileFor(fileName) {
    const source = language.scripts.get(fileName);
    if (!source?.generated) return;
    return ts.createSourceFile(
      fileName,
      source.snapshot.getText(0, source.snapshot.getLength()),
      ts.ScriptTarget.Latest,
      false,
      ts.ScriptKind.TSX,
    );
  }
  function authoredDiagnostic(diagnostic) {
    return {
      ...diagnostic,
      file: diagnostic.file
        ? (sourceFileFor(diagnostic.file.fileName) ?? diagnostic.file)
        : undefined,
      relatedInformation: diagnostic.relatedInformation?.map(authoredDiagnostic),
    };
  }
  const languageService = new Proxy(proxy, {
    get(target, key) {
      if (key === "getSyntacticDiagnostics" || key === "getSemanticDiagnostics") {
        return (fileName) => {
          const diagnostics = target[key](fileName).map(authoredDiagnostic);
          if (key === "getSemanticDiagnostics") {
            const file = sourceFileFor(fileName);
            if (file)
              diagnostics.push(
                ...(diagnosticsFor(fileName)?.sourceDiagnostics ?? []).map((d) => ({
                  ...d,
                  file,
                  category: ts.DiagnosticCategory.Error,
                  source: "vize",
                })),
              );
          }
          return diagnostics;
        };
      }
      if (key === "dispose")
        return () => {
          for (const id of [...registry.keys()]) language.scripts.delete(id);
          rawService.dispose();
        };
      const value = target[key];
      return typeof value === "function" ? value.bind(target) : value;
    },
  });
  return { language, languageService, plugin };
}
