import type { CreateVizeVueRulesOptions, LoaderEntry, VizeStyleLanguage } from "../types/index.js";

const DEFAULT_LANGUAGES: VizeStyleLanguage[] = ["scss", "sass", "less", "stylus", "styl"];

const PREPROCESSOR_LOADER_MAP: Record<string, string> = {
  scss: "sass-loader",
  sass: "sass-loader",
  less: "less-loader",
  styl: "stylus-loader",
  stylus: "stylus-loader",
};

export function createVizeVueRules(
  options: CreateVizeVueRulesOptions = {},
): Record<string, unknown>[] {
  const {
    isProduction = false,
    nativeCss = true,
    styleLanguages = DEFAULT_LANGUAGES,
    vizeLoader = "@vizejs/rspack-plugin/loader",
    vizeStyleLoader = "@vizejs/rspack-plugin/style-loader",
    styleInjectLoader = "style-loader",
    styleExtractLoader,
    cssLoader = "css-loader",
    loaderOptions,
    styleLoaderOptions,
    typescript = false,
  } = options;

  const normalizedLanguages = Array.from(new Set(styleLanguages.filter((lang) => lang !== "css")));

  const styleLoaderEntry = {
    loader: vizeStyleLoader,
    ...(styleLoaderOptions ? { options: styleLoaderOptions } : {}),
  };

  const oneOf: Record<string, unknown>[] = [];

  for (const lang of normalizedLanguages) {
    const preprocessorLoader = PREPROCESSOR_LOADER_MAP[lang];
    if (!preprocessorLoader) {
      continue;
    }

    oneOf.push(
      createStyleRule({
        lang,
        module: true,
        nativeCss,
        styleLoaderEntry,
        preprocessorLoader,
        isProduction,
        styleInjectLoader,
        styleExtractLoader,
        cssLoader,
      }),
    );
  }

  oneOf.push(
    createStyleRule({
      module: true,
      nativeCss,
      styleLoaderEntry,
      isProduction,
      styleInjectLoader,
      styleExtractLoader,
      cssLoader,
    }),
  );

  for (const lang of normalizedLanguages) {
    const preprocessorLoader = PREPROCESSOR_LOADER_MAP[lang];
    if (!preprocessorLoader) {
      continue;
    }

    oneOf.push(
      createStyleRule({
        lang,
        module: false,
        nativeCss,
        styleLoaderEntry,
        preprocessorLoader,
        isProduction,
        styleInjectLoader,
        styleExtractLoader,
        cssLoader,
      }),
    );
  }

  oneOf.push(
    createStyleRule({
      module: false,
      nativeCss,
      styleLoaderEntry,
      isProduction,
      styleInjectLoader,
      styleExtractLoader,
      cssLoader,
    }),
  );

  oneOf.push({
    use: [
      {
        loader: vizeLoader,
        options: loaderOptions,
      },
    ],
  });

  return [
    {
      test: /\.vue$/,
      oneOf,
    },
    // TypeScript post-processing: strip type annotations from compiled .vue output.
    // @vizejs/native preserves TS syntax (like @vue/compiler-sfc), so a downstream
    // transpiler is needed until native adds an optional TS-stripping pass.
    ...(typescript
      ? [
          {
            test: /\.vue$/,
            // Exclude ALL sub-requests (style, custom blocks, etc.) — only
            // the main SFC compilation output contains TypeScript that needs
            // to be transpiled.  Custom block content may be JSON, YAML, or
            // other non-JS formats that would break the TS parser.
            resourceQuery: { not: [/type=/] },
            enforce: "post" as const,
            ...(typescript === true
              ? {
                  loader: "builtin:swc-loader",
                  options: {
                    jsc: {
                      parser: {
                        syntax: "typescript",
                      },
                    },
                  },
                }
              : typeof typescript === "string"
                ? { loader: typescript }
                : typescript),
            type: "javascript/auto",
          },
        ]
      : []),
  ];
}

function createStyleRule(options: {
  lang?: string;
  module: boolean;
  nativeCss: boolean;
  styleLoaderEntry: Record<string, unknown>;
  preprocessorLoader?: string;
  isProduction: boolean;
  styleInjectLoader: LoaderEntry;
  styleExtractLoader?: LoaderEntry;
  cssLoader: LoaderEntry;
}): Record<string, unknown> {
  const {
    lang,
    module,
    nativeCss,
    styleLoaderEntry,
    preprocessorLoader,
    isProduction,
    styleInjectLoader,
    styleExtractLoader,
    cssLoader,
  } = options;

  // Build order-independent regex using lookaheads.
  // Each part is wrapped in (?=.*part) so query parameter order doesn't matter.
  // This ensures e.g. "type=style&lang=scss&module=true" matches regardless of
  // whether module comes before or after lang.
  const queryParts = ["type=style"];
  if (module) {
    queryParts.push("module");
  }
  if (lang) {
    queryParts.push(`lang=${lang}`);
  }

  const resourceQuery = new RegExp(queryParts.map((p) => `(?=.*${p})`).join(""));

  if (nativeCss) {
    return {
      resourceQuery,
      type: module ? "css/module" : "css/auto",
      use: preprocessorLoader ? [preprocessorLoader, styleLoaderEntry] : [styleLoaderEntry],
    };
  }

  const cssLoaderEntry =
    typeof cssLoader === "string"
      ? {
          loader: cssLoader,
          options: {
            modules: {
              auto: (_resourcePath: string, resourceQueryArg: unknown) =>
                typeof resourceQueryArg === "string" && resourceQueryArg.includes("module="),
            },
          },
        }
      : cssLoader;

  return {
    resourceQuery,
    type: "javascript/auto",
    use: [
      isProduction && styleExtractLoader ? styleExtractLoader : styleInjectLoader,
      cssLoaderEntry,
      ...(preprocessorLoader ? [preprocessorLoader] : []),
      styleLoaderEntry,
    ],
  };
}
