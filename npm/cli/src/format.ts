import { createRequire } from "node:module";

import type { FormatFileOptionsNapi, FormatOptionsNapi, FormatResultNapi } from "@vizejs/native";

import { assertMatchingNativeVersion } from "./native-version-guard.js";

/**
 * Formatter options, named as in the `fmt` / `vue.formatter` config. Each one
 * given overrides the configured value, as a `vize fmt` flag does.
 */
export type FormatterOverrides = FormatOptionsNapi;

/** Options for {@link format}: the file name, where its config comes from, and overrides. */
export interface FormatOptions extends FormatterOverrides {
  /**
   * Name of the file the source is written to. Its extension selects the
   * formatter (`.vue`, `.ts`, `.tsx`, `.js`, `.tsrx`, `.json`, ...),
   * and its directory selects the configuration the way `vize fmt` run from
   * that directory would. Relative names resolve from `process.cwd()`. The
   * file itself is never read or written.
   */
  filename: string;
  /**
   * A Vite config file or project directory to read instead of discovering
   * one from `filename`, or `false` to format with defaults only.
   * @default discovered from the directory of `filename`
   */
  config?: string | false;
}

/** What {@link format} returns. */
export interface FormatResult {
  /** The formatted source: the bytes `vize fmt --write` would write. */
  code: string;
  /** Whether `code` differs from the input. */
  changed: boolean;
}

type NativeFormatter = {
  formatFile(source: string, options: FormatFileOptionsNapi): Promise<FormatResultNapi>;
  formatFileSync(source: string, options: FormatFileOptionsNapi): FormatResultNapi;
};

const require = createRequire(import.meta.url);
let native: NativeFormatter | undefined;

// Loaded on first use so importing config helpers never loads the addon.
function formatter(): NativeFormatter {
  if (native === undefined) {
    assertMatchingNativeVersion();
    native = require("@vizejs/native") as NativeFormatter;
  }
  return native;
}

function request(source: unknown, options: FormatOptions | undefined): FormatFileOptionsNapi {
  if (typeof source !== "string") {
    throw new TypeError("format(source, options): source must be a string");
  }
  if (typeof options?.filename !== "string" || options.filename === "") {
    throw new TypeError("format(source, options): options.filename must be a non-empty string");
  }
  const { filename, config, ...overrides } = options;
  if (config !== undefined && config !== false && typeof config !== "string") {
    throw new TypeError("format(source, options): options.config must be a path or false");
  }
  return {
    filename,
    config: typeof config === "string" ? config : undefined,
    noConfig: config === false,
    overrides,
  };
}

/**
 * Format `source` as `vize fmt --write` would format a file named
 * `options.filename`, with the same configuration. Runs off the main thread.
 * Rejects with the formatter's message when the source does not parse, the
 * file type is not formatted, or the configuration is invalid.
 */
export function format(source: string, options: FormatOptions): Promise<FormatResult> {
  try {
    return formatter().formatFile(source, request(source, options));
  } catch (error) {
    return Promise.reject(error);
  }
}

/** Synchronous {@link format}, for callers that cannot await. */
export function formatSync(source: string, options: FormatOptions): FormatResult {
  return formatter().formatFileSync(source, request(source, options));
}
