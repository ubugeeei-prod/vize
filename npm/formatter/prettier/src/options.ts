import type { FormatOptionsNapi } from "@vizejs/native";
import type { ParserOptions } from "prettier";

export interface VizePluginOptions {
  vizeVueVersion?: "2" | "3";
}

export type VizeOptions = ParserOptions & VizePluginOptions;

export function nativeOptions(options: VizeOptions, length: number): FormatOptionsNapi {
  const unsupported: [string, unknown, unknown][] = [
    ["htmlWhitespaceSensitivity", options.htmlWhitespaceSensitivity, "css"],
    ["vueIndentScriptAndStyle", options.vueIndentScriptAndStyle, false],
    ["embeddedLanguageFormatting", options.embeddedLanguageFormatting, "auto"],
    ["proseWrap", options.proseWrap, "preserve"],
    ["objectWrap", options.objectWrap, "preserve"],
    ["experimentalTernaries", options.experimentalTernaries, false],
    ["experimentalOperatorPosition", options.experimentalOperatorPosition, "end"],
  ];
  for (const [name, value, supported] of unsupported) {
    if (value !== undefined && value !== supported) {
      throw new Error(
        `prettier-plugin-vize does not support ${name}=${JSON.stringify(value)}. Supported value: ${JSON.stringify(supported)}.`,
      );
    }
  }
  if ((options.rangeStart ?? 0) !== 0 || (options.rangeEnd ?? Infinity) < length) {
    throw new Error(
      "prettier-plugin-vize supports whole-file formatting only; rangeStart/rangeEnd cannot select a partial file.",
    );
  }
  if (!Number.isInteger(options.printWidth) || options.printWidth < 1 || options.printWidth > 320) {
    throw new Error(
      "prettier-plugin-vize printWidth must be an integer from 1 to 320 (the shared script formatter's supported range).",
    );
  }
  if (!Number.isInteger(options.tabWidth) || options.tabWidth < 0 || options.tabWidth > 24) {
    throw new Error(
      "prettier-plugin-vize tabWidth must be an integer from 0 to 24 (the shared formatter's supported range).",
    );
  }
  return {
    preset: "prettier",
    vueVersion: options.vizeVueVersion ?? "3",
    printWidth: options.printWidth,
    tabWidth: options.tabWidth,
    useTabs: options.useTabs,
    semi: options.semi,
    singleQuote: options.singleQuote,
    jsxSingleQuote: options.jsxSingleQuote,
    quoteProps: options.quoteProps,
    trailingComma: options.trailingComma,
    arrowParens: options.arrowParens,
    bracketSpacing: options.bracketSpacing,
    bracketSameLine: options.bracketSameLine,
    singleAttributePerLine: options.singleAttributePerLine,
    // Prettier normalizes input and owns final EOL emission through the Doc.
    endOfLine: "lf",
  };
}
