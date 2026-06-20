import { compileVueModule } from "./compiler.ts";
import { createFilter } from "./filter.ts";
import { generateOutput } from "./style.ts";
import { normalizeOptions } from "./unplugin.ts";
import type { CachedCompiledModule, VizeUnpluginOptions } from "./types.ts";

type BabelParserOptions = Record<string, unknown> & {
  filename?: string;
  sourceFilename?: string;
  sourceFileName?: string;
  plugins?: unknown[];
  sourceType?: string;
};

type BabelParse = (code: string, options?: BabelParserOptions) => unknown;

export default function vizeBabelPlugin(
  _api: unknown,
  rawOptions: VizeUnpluginOptions = {},
): {
  name: string;
  manipulateOptions: (_options: unknown, parserOptions: BabelParserOptions) => void;
  parserOverride: (source: string, parserOptions: BabelParserOptions, parse: BabelParse) => unknown;
} {
  const options = normalizeOptions(rawOptions);
  const filter = createFilter(options.include, options.exclude);
  const cache = new Map<string, CachedCompiledModule>();

  return {
    name: "babel-plugin-vize",
    manipulateOptions(_options, parserOptions) {
      ensureParserPlugin(parserOptions, "typescript");
      ensureParserPlugin(parserOptions, "jsx");
    },
    parserOverride(source, parserOptions, parse) {
      const filename = getFilename(parserOptions);
      if (!filename || !filename.endsWith(".vue") || !filter(filename)) {
        return undefined;
      }

      const { compiled, warnings } = compileVueModule(filename, source, options, cache);
      for (const warning of warnings) {
        process.emitWarning(`[vize] ${warning}`, { type: "VizeWarning" });
      }

      const generated = generateOutput(compiled, {
        isProduction: options.isProduction,
        isDev: false,
        filePath: filename,
      });

      return parse(generated, {
        ...parserOptions,
        filename,
        sourceType: "module",
      });
    },
  };
}

function getFilename(parserOptions: BabelParserOptions): string {
  return String(
    parserOptions.filename ?? parserOptions.sourceFilename ?? parserOptions.sourceFileName ?? "",
  );
}

function ensureParserPlugin(parserOptions: BabelParserOptions, pluginName: string): void {
  const plugins = parserOptions.plugins ?? [];
  if (!plugins.some((plugin) => parserPluginName(plugin) === pluginName)) {
    plugins.push(pluginName);
  }
  parserOptions.plugins = plugins;
}

function parserPluginName(plugin: unknown): string | null {
  if (typeof plugin === "string") {
    return plugin;
  }
  if (Array.isArray(plugin) && typeof plugin[0] === "string") {
    return plugin[0];
  }
  return null;
}
