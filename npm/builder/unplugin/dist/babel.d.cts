import { r as VizeUnpluginOptions } from "./types-D3or3aSV.cjs";

//#region src/babel.d.ts
type BabelParserOptions = Record<string, unknown> & {
  filename?: string;
  sourceFilename?: string;
  sourceFileName?: string;
  plugins?: unknown[];
  sourceType?: string;
};
type BabelParse = (code: string, options?: BabelParserOptions) => unknown;
declare function vizeBabelPlugin(
  _api: unknown,
  rawOptions?: VizeUnpluginOptions,
): {
  name: string;
  manipulateOptions: (_options: unknown, parserOptions: BabelParserOptions) => void;
  parserOverride: (source: string, parserOptions: BabelParserOptions, parse: BabelParse) => unknown;
};
export = vizeBabelPlugin;
