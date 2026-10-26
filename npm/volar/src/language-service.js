import { createLanguage } from "@volar/language-core";
import { createProxyLanguageService, decorateLanguageServiceHost } from "@volar/typescript";
import {
  applyDiagnosticDirectives,
  mapLanguageDiagnosticRange,
} from "@vizejs/native/language-projection.js";
import { createVizeLanguagePlugin, VizeVirtualCode } from "./language-plugin.js";

/** Compose the actual Volar context and TypeScript service, including mapper diagnostics. */
export function createVizeLanguageService(ts, host, options = {}) {
  const plugin = createVizeLanguagePlugin(ts, options);
  const registry = new Map();
  const originalSnapshot = host.getScriptSnapshot.bind(host);
  const language = createLanguage([plugin], registry, (id) => {
    const snapshot = originalSnapshot(id);
    const standardLanguage = /\.[cm]?tsx?$/i.test(id)
      ? /\.tsx$/i.test(id)
        ? "typescriptreact"
        : "typescript"
      : /\.[cm]?jsx?$/i.test(id)
        ? /\.jsx$/i.test(id)
          ? "javascriptreact"
          : "javascript"
        : /\.json$/i.test(id)
          ? "json"
          : undefined;
    if (snapshot) language.scripts.set(id, snapshot, plugin.getLanguageId(id) ?? standardLanguage);
    else language.scripts.delete(id);
  });
  // Hosts may be class instances with non-enumerable prototype methods and
  // private fields. Keep those methods bound to their actual owner while
  // letting Volar decorate a private facade instead of mutating that owner.
  const boundMethods = new WeakMap();
  const adapterHost = new Proxy(Object.create(host), {
    get(target, key, receiver) {
      if (Object.hasOwn(target, key)) return Reflect.get(target, key, receiver);
      const value = host[key];
      if (typeof value !== "function") return value;
      if (!boundMethods.has(value)) boundMethods.set(value, value.bind(host));
      return boundMethods.get(value);
    },
    set(target, key, value) {
      Object.defineProperty(target, key, {
        value,
        writable: true,
        enumerable: true,
        configurable: true,
      });
      return true;
    },
  });
  adapterHost.getScriptSnapshot = originalSnapshot;
  adapterHost.getCompilationSettings = () => {
    const settings = host.getCompilationSettings();
    if (settings.allowNonTsExtensions === false)
      throw new Error("Volar requires allowNonTsExtensions for Vue service scripts");
    return { ...settings, allowNonTsExtensions: true };
  };
  adapterHost.getScriptKind ??= () => ts.ScriptKind.Unknown;
  if (!adapterHost.resolveModuleNameLiterals && !adapterHost.resolveModuleNames)
    adapterHost.resolveModuleNameLiterals = (
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
  const proxied = createProxyLanguageService(rawService);
  proxied.initialize(language);
  const proxy = proxied.proxy;

  const sourceFiles = new WeakMap();
  function sourceFileFor(fileName) {
    const source = language.scripts.get(fileName);
    if (!source?.generated) return;
    let files = sourceFiles.get(source.snapshot);
    if (!files) sourceFiles.set(source.snapshot, (files = new Map()));
    let file = files.get(fileName);
    if (!file) {
      file = ts.createSourceFile(
        fileName,
        source.snapshot.getText(0, source.snapshot.getLength()),
        ts.ScriptTarget.Latest,
        false,
        ts.ScriptKind.TSX,
      );
      files.set(fileName, file);
    }
    return file;
  }
  function authoredDiagnostic(diagnostic) {
    const root = diagnostic.file && rootFor(diagnostic.file.fileName);
    const mapped =
      root && diagnostic.start !== undefined && diagnostic.length !== undefined
        ? mapLanguageDiagnosticRange(root.projection, diagnostic.start, diagnostic.length)
        : undefined;
    const unmapped = mapped?.fidelity === "none";
    const messageText = !unmapped
      ? diagnostic.messageText
      : typeof diagnostic.messageText === "string"
        ? `Vize generated TypeScript: ${diagnostic.messageText}`
        : {
            ...diagnostic.messageText,
            messageText: `Vize generated TypeScript: ${diagnostic.messageText.messageText}`,
          };
    return {
      ...diagnostic,
      ...(mapped && { start: mapped.start, length: mapped.length }),
      messageText,
      ...(unmapped && { source: "vize-generated" }),
      file: diagnostic.file
        ? (sourceFileFor(diagnostic.file.fileName) ?? diagnostic.file)
        : undefined,
      relatedInformation: diagnostic.relatedInformation?.map(authoredDiagnostic),
    };
  }
  const languageService = new Proxy(proxy, {
    get(target, key) {
      if (key === "toLineColumnOffset")
        return (fileName, position) => {
          const authored = sourceFileFor(fileName);
          if (authored) return authored.getLineAndCharacterOfPosition(position);
          if (rawService.toLineColumnOffset)
            return rawService.toLineColumnOffset(fileName, position);
          throw new Error(`TypeScript host cannot resolve line and column for ${fileName}`);
        };
      if (key === "getSyntacticDiagnostics" || key === "getSemanticDiagnostics") {
        return (fileName) => {
          const collection = diagnosticsFor(fileName);
          const raw =
            collection?.[key === "getSyntacticDiagnostics" ? "syntactic" : "semantic"] ??
            rawService[key](fileName);
          const diagnostics = raw.map(authoredDiagnostic);
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
          for (const id of registry.keys()) language.scripts.delete(id);
          rawService.dispose();
        };
      const value = target[key];
      return typeof value === "function" ? value.bind(target) : value;
    },
  });
  return { language, languageService, plugin };
}
