import { enableSyntax, hasJsx, rejectForeignRuntime } from "./ast.ts";
import { getCompiler } from "./native.ts";
import { normalizeOptions } from "./options.ts";
import { captureReservations, restoreReservations } from "./scopes.ts";
import { compileSfcForBabel, installSfcFileState, isVueSfc, type CompiledSfc } from "./sfc.ts";
import { composeInputMap } from "./source-map.ts";
import type {
  BabelApi,
  BabelFile,
  BabelParse,
  FileAst,
  ParserOptions,
  VizeBabelMetadata,
  VizeBabelOptions,
} from "./types.ts";

export type { VizeBabelMetadata, VizeBabelOptions, VizeBabelSfcMetadata } from "./types.ts";

const compiled = Symbol.for("@vizejs/babel-plugin/compiled");

/** Babel owns the returned hooks; consumers configure the strongly typed options. */
export default function vizeBabelPlugin(
  apiValue: unknown,
  rawOptions: VizeBabelOptions = {},
): { name: string } {
  const api = apiValue as BabelApi;
  api.assertVersion("^7.29.7 || ^8.0.9");
  const options = normalizeOptions(rawOptions);
  const sfcOutputs = new WeakMap<object, CompiledSfc>();
  const assertComplete = (ast: FileAst) => {
    if (hasJsx(ast, api.types.VISITOR_KEYS)) {
      throw new Error(
        "JSX was introduced after the native phase; run JSX-producing transforms in an earlier Babel pass",
      );
    }
  };
  const plugin = {
    name: "vize-vue-jsx",
    manipulateOptions(host: { filename?: string }, parser: Parameters<typeof enableSyntax>[1]) {
      enableSyntax(host.filename ?? "", parser);
    },
    parserOverride(source: string, parserOptions: ParserOptions, parse: BabelParse) {
      const filename = String(
        parserOptions.filename ??
          parserOptions.sourceFilename ??
          parserOptions.sourceFileName ??
          "",
      );
      if (!filename || !isVueSfc(filename)) return undefined;
      const compiledSfc = compileSfcForBabel(source, filename, parserOptions, parse, options);
      sfcOutputs.set(compiledSfc.ast, compiledSfc);
      return compiledSfc.ast;
    },
    pre(file: BabelFile) {
      const compiledSfc = sfcOutputs.get(file.ast);
      if (compiledSfc) {
        file.set(compiled, true);
        installSfcFileState(file, compiledSfc);
        return;
      }
      if (!hasJsx(file.ast, api.types.VISITOR_KEYS)) return;
      if (file.get(compiled))
        throw new Error("JSX was introduced after this file's native compilation pass");
      rejectForeignRuntime(file.ast, file.opts.plugins, file.scope);
      const reservations = captureReservations(file);
      const maps = Boolean(file.opts.sourceMaps);
      const hostGenerator = file.opts.generatorOpts;
      const printed = api.transformFromAstSync(file.ast, file.code, {
        filename: file.opts.filename,
        sourceFileName: file.opts.sourceFileName,
        sourceRoot: file.opts.sourceRoot,
        babelrc: false,
        configFile: false,
        plugins: [],
        presets: [],
        ast: false,
        sourceMaps: maps,
        inputSourceMap: maps ? (file.inputMap?.toObject() ?? false) : false,
        comments: file.opts.comments,
        generatorOpts: {
          ...(typeof hostGenerator === "object" && hostGenerator !== null ? hostGenerator : {}),
          sourceMaps: maps,
        },
      });
      if (!printed || typeof printed.code !== "string")
        throw new Error("Babel could not serialize the current JSX AST");
      const parserPlugins = file.opts.parserOpts?.plugins ?? [];
      const typescript =
        /\.(?:tsx|ts|mts|cts)$/iu.test(file.opts.filename ?? "") ||
        parserPlugins.some((p) => (Array.isArray(p) ? p[0] : p) === "typescript");
      const output = getCompiler().compileJsx(printed.code, {
        filename: file.opts.filename,
        lang: typescript ? "tsx" : "jsx",
        jsxMode: options.jsxMode,
        jsxCompat: options.jsxCompat,
        ssr: options.ssr,
        sourceMap: maps,
      });
      file.set(compiled, true);
      if (output.errors.length > 0) throw new Error(output.errors.join("\n"));
      if (output.scopedStyles.length > 0 && options.scopedStyles === "error") {
        throw new Error(
          'Scoped JSX CSS requires a bundler consumer; set scopedStyles: "metadata" and consume file.metadata.vize.jsx.scopedStyles',
        );
      }
      const generated = api.parseSync(output.code, {
        filename: file.opts.filename,
        babelrc: false,
        configFile: false,
        plugins: [],
        presets: [],
        sourceType: "module",
        parserOpts: { ...file.opts.parserOpts, sourceType: "module" },
      });
      if (!generated || hasJsx(generated, api.types.VISITOR_KEYS)) {
        throw new Error("The native JSX compiler did not return a complete JSX-free module");
      }
      const inputMap = maps ? composeInputMap(output.map, printed.map) : null;
      file.path.replaceWith(generated.program);
      for (const key of ["comments", "tokens", "errors", "start", "end", "loc"]) {
        if (key in generated) file.ast[key] = generated[key];
        else delete file.ast[key];
      }
      file.scope = file.path.scope;
      file.scope.crawl();
      restoreReservations(file, reservations);
      file.code = output.code;
      file.inputMap = inputMap;
      const old = file.metadata.vize;
      const metadata: VizeBabelMetadata = {
        defaultMode: options.jsxMode,
        ssr: options.ssr,
        nativePasses: 1,
        warnings: output.warnings,
        scopedStyles: output.scopedStyles,
      };
      file.metadata.vize = {
        ...(typeof old === "object" && old !== null ? old : {}),
        jsx: metadata,
      };
      for (const warning of output.warnings)
        process.emitWarning(`[vize] ${warning}`, { type: "VizeJsxWarning" });
    },
    post(file: BabelFile) {
      assertComplete(file.ast);
    },
    generatorOverride(ast: FileAst) {
      // Later post hooks run after our post. Inspect their final AST, then
      // return undefined so Babel or another real override owns generation.
      assertComplete(ast);
      return undefined;
    },
  };
  return plugin;
}
