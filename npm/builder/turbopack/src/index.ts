import path from "node:path";
import type { NextConfig } from "next";
import type { VizeTurbopackOptions } from "./types.ts";

export type { VizeTurbopackOptions } from "./types.ts";

const ownedStyle = /[?&]vize=turbopack(?:&|$)/;
const moduleStyle = /[?&]module(?:=|&|$)/;
const inlineStyle = /[?&]inline(?:=|&|$)/;

/** Loader rules for native Vue compilation and Turbopack's own CSS pipeline. */
export function vizeTurbopack(
  options: VizeTurbopackOptions = {},
): NonNullable<NextConfig["turbopack"]> {
  const { styleLoaders = {}, root = process.cwd(), ...compiler } = options;
  const languages = ["css", ...Object.keys(styleLoaders).filter((lang) => lang !== "css")];
  if (languages.some((lang) => !/^[a-z][a-z0-9-]*$/.test(lang))) {
    throw new Error(
      "[vize] Style loader language names must contain lowercase letters, digits or hyphens.",
    );
  }
  const rules: NonNullable<NonNullable<NextConfig["turbopack"]>["rules"]> = {};
  rules["*.vue"] = [false, true].flatMap((ssr) =>
    [false, true].map((isProduction) => ({
      condition: {
        all: [
          { not: { query: ownedStyle } },
          ssr ? { not: "browser" as const } : ("browser" as const),
          isProduction ? ("production" as const) : ("development" as const),
        ],
      },
      loaders: [
        {
          loader: "@vizejs/turbopack-plugin/loader",
          options: {
            ...compiler,
            root: path.resolve(root),
            ssr,
            isProduction,
            styleLanguages: languages,
          },
        },
      ],
      as: "*.js",
      type: "ecmascript" as const,
    })),
  );
  // Disjoint conditions are essential: Turbopack runs every matching rule.
  rules["*"] = languages.flatMap((lang) =>
    [false, true].flatMap((module) =>
      [false, true].map((inline) => ({
        condition: {
          all: [
            { query: ownedStyle },
            { query: new RegExp(`[?&]lang=${lang}(?:&|$)`) },
            module ? { query: moduleStyle } : { not: { query: moduleStyle } },
            inline
              ? { all: [{ query: inlineStyle }, { path: "*.vue" }] }
              : { not: { query: inlineStyle } },
          ],
        },
        loaders: [
          {
            loader: "@vizejs/turbopack-plugin/scope-loader",
            options: { root: path.resolve(root), sourceMap: options.sourceMap ?? true },
          },
          ...(styleLoaders[lang] ?? []),
          ...(inline ? ["@vizejs/turbopack-plugin/style-loader"] : []),
        ],
        // A processed CSS asset must not re-enter its SFC/preprocessor loaders.
        ...(inline ? { as: module ? "*.module.css" : "*.css" } : {}),
        type: module ? ("css-module" as const) : ("css" as const),
      })),
    ),
  );
  return {
    root: path.resolve(root),
    rules,
    resolveExtensions: [".vue", ".tsx", ".ts", ".jsx", ".js", ".mjs", ".node", ".json"],
  };
}

/** Configure Vue's compile-time flags while preserving the host configuration. */
export function withVize(config: NextConfig = {}, options: VizeTurbopackOptions = {}): NextConfig {
  const vize = vizeTurbopack({ ...options, root: options.root ?? config.turbopack?.root });
  const rules = { ...config.turbopack?.rules };
  for (const [pattern, additions] of Object.entries(vize.rules ?? {})) {
    const current = rules[pattern];
    rules[pattern] = [
      ...(current === undefined ? [] : Array.isArray(current) ? current : [current]),
      ...(Array.isArray(additions) ? additions : [additions]),
    ];
  }
  return {
    ...config,
    compiler: {
      ...config.compiler,
      define: {
        __VUE_OPTIONS_API__: true,
        __VUE_PROD_DEVTOOLS__: false,
        __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: false,
        ...config.compiler?.define,
      },
    },
    turbopack: {
      ...vize,
      ...config.turbopack,
      root: vize.root,
      rules,
      resolveExtensions: [
        ...new Set([
          ".vue",
          ...(config.turbopack?.resolveExtensions ?? vize.resolveExtensions ?? []),
        ]),
      ],
    },
  };
}

export default vizeTurbopack;
