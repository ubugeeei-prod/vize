/**
 * The `#build/*` files the nuxt-ui corpus resolves, as `nuxt prepare` writes
 * them for an app that installs Nuxt UI with its default options:
 *
 * - `.nuxt/ui/**`: every component theme (src/templates.ts, production branch
 *   of `writeThemeTemplate`) evaluated with the module's default options and
 *   serialized as JSON, each variant value narrowed to its `as const` key
 *   list so `tv()` variant props keep their literal types. Prose and content
 *   themes are written too, as an app with @nuxt/content gets them.
 * - `.nuxt/types/ui.d.ts`: the module's `CustomAppConfig` augmentation
 *   (`types/ui.d.ts` in src/templates.ts).
 * - `.nuxt/types/app.config.d.ts`: Nuxt's `AppConfig` augmentation over the
 *   module's default app config, which every `useAppConfig()` reads.
 *
 * The themes and defaults come from the copied, pinned src, so the output
 * moves only with the corpus revision.
 *
 *   node typecheck-corpus-nuxt-ui-theme.ts <project>
 *
 * Runs in its own process: the resolve hook that lets Node load the
 * extensionless theme imports exists only here.
 */

import { mkdirSync, writeFileSync } from "node:fs";
import { registerHooks } from "node:module";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

type Theme = Record<string, unknown> & { variants?: Record<string, Record<string, unknown>> };
type ThemeModule = Record<string, Theme | ((options: unknown) => Theme)>;
type Options = {
  theme: { colors?: string[]; defaultVariants?: unknown; unstyled?: boolean; prefix?: string };
};

/** scule's `kebabCase` for the camelCase export names the theme indexes use. */
export const kebabCase = (name: string): string =>
  name.replace(/([a-z0-9])([A-Z])/gu, "$1-$2").toLowerCase();

/** One theme template's text, as `writeThemeTemplate` returns it outside `--uiDev`. */
export function themeTemplate(result: Theme): string {
  const variants = Object.entries(result.variants ?? {})
    .filter(([, values]) => Object.keys(values).some((key) => key !== "true" && key !== "false"))
    .map(([key]) => key);
  let json = JSON.stringify(result, null, 2);
  for (const variant of variants) {
    json = json.replace(
      new RegExp(`("${variant}": "[^"]+")`, "g"),
      `$1 as typeof ${variant}[number]`,
    );
    json = json.replace(
      new RegExp(`("${variant}": \\[\\s*)((?:"[^"]+",?\\s*)+)(\\])`, "g"),
      (_, before: string, match: string, after: string) =>
        `${before}${match.replace(/("[^"]+")/g, `$1 as typeof ${variant}[number]`)}${after}`,
    );
  }
  const declarations = variants
    .filter((variant) => json.includes(`as typeof ${variant}`))
    .map(
      (variant) =>
        `const ${variant} = ${JSON.stringify(Object.keys(result.variants![variant]!), null, 2)} as const`,
    );
  return [...declarations, `export default ${json}`].join("\n\n");
}

/**
 * The module's `types/ui.d.ts` template. `@nuxt/ui` resolves to the package's
 * own `runtime/types` exports; here the corpus's copy of them.
 */
export function uiTypesTemplate(colors: string[], icons: string[]): string {
  const iconUnion = icons.length ? icons.map((icon) => JSON.stringify(icon)).join(" | ") : "string";
  return `import * as ui from '#build/ui'
import type { TVConfig, DeepRequired } from '../../src/runtime/types'
import type { defaultConfig } from 'tailwind-variants'
import colors from 'tailwindcss/colors'

type IconsConfig = Record<${iconUnion} | (string & {}), string>

type NeutralColor = 'slate' | 'gray' | 'zinc' | 'neutral' | 'stone' | 'taupe' | 'mauve' | 'mist' | 'olive'
type Color = Exclude<keyof typeof colors, 'inherit' | 'current' | 'transparent' | 'black' | 'white' | NeutralColor> | (string & {})

type AppConfigUI = {
  colors?: {
    ${colors.map((color) => `'${color}'?: Color`).join("\n\t\t")}
    neutral?: NeutralColor | (string & {})
  }
  icons?: Partial<IconsConfig>
  prefix?: string
  tv?: typeof defaultConfig
} & TVConfig<typeof ui>

type AppConfigRuntimeUI = DeepRequired<Pick<AppConfigUI, 'colors' | 'icons' | 'tv'>> & typeof ui

declare module '@nuxt/schema' {
  interface AppConfigInput {
    /**
     * Nuxt UI theme configuration
     * @see https://ui.nuxt.com/docs/getting-started/theme/components
     */
    ui?: AppConfigUI
  }
  interface CustomAppConfig {
    ui: AppConfigRuntimeUI
  }
}

export {}
`;
}

/** Nuxt's `types/app.config.d.ts` for an app with no app.config file of its own. */
export function appConfigTypesTemplate(appConfig: unknown): string {
  return `import type { CustomAppConfig } from '@nuxt/schema'
import type { Defu } from 'defu'

declare const inlineConfig = ${JSON.stringify(appConfig, null, 2)}
type ResolvedAppConfig = Defu<typeof inlineConfig, []>
type IsAny<T> = 0 extends 1 & T ? true : false

type MergedAppConfig<Resolved extends Record<string, unknown>, Custom extends Record<string, unknown>> = {
  [K in keyof (Resolved & Custom)]: K extends keyof Custom
    ? unknown extends Custom[K]
      ? Resolved[K]
      : IsAny<Custom[K]> extends true
        ? Resolved[K]
        : Custom[K] extends Record<string, any>
            ? Resolved[K] extends Record<string, any>
              ? MergedAppConfig<Resolved[K], Custom[K]>
              : Exclude<Custom[K], undefined>
            : Exclude<Custom[K], undefined>
    : Resolved[K]
}

declare module '@nuxt/schema' {
  interface AppConfig extends MergedAppConfig<ResolvedAppConfig, CustomAppConfig> { }
}
`;
}

/** Let Node load the extensionless relative imports the themes use, as the bundler resolves them. */
function resolveExtensionless(): void {
  registerHooks({
    resolve(specifier, context, nextResolve) {
      if (specifier.startsWith(".") && !/\.[cm]?[jt]s$/u.test(specifier)) {
        for (const suffix of [".ts", "/index.ts"]) {
          try {
            return nextResolve(`${specifier}${suffix}`, context);
          } catch {
            // Try the next spelling.
          }
        }
      }
      return nextResolve(specifier, context);
    },
  });
}

async function main(): Promise<void> {
  resolveExtensionless();
  const project = resolve(process.argv[2] ?? "");
  const load = async <T>(file: string) =>
    (await import(pathToFileURL(join(project, "src", file)).href)) as T;
  const { defaultOptions, getDefaultConfig, resolveColors } = await load<{
    defaultOptions: Options;
    getDefaultConfig: (theme: Options["theme"]) => { icons: Record<string, string> };
    resolveColors: (colors?: string[]) => string[];
  }>("utils/defaults.ts");
  const { applyDefaultVariants, applyUnstyled, applyPrefixToObject } =
    await load<Record<string, (result: Theme, option: unknown) => Theme>>("utils/theme.ts");
  // The module's `defu(options, defaults)` with no user options, then its setup's color resolution.
  const options = structuredClone(defaultOptions);
  options.theme.colors = resolveColors(options.theme.colors);
  const out = join(project, ".nuxt");
  const write = (file: string, text: string) => {
    mkdirSync(dirname(join(out, file)), { recursive: true });
    writeFileSync(join(out, file), text);
  };
  const names: Record<string, string[]> = { ui: [], "ui/prose": [], "ui/content": [] };
  for (const [dir, entry] of [
    ["ui", "theme/index.ts"],
    ["ui/prose", "theme/prose/index.ts"],
    ["ui/content", "theme/content/index.ts"],
  ] as const) {
    for (const [name, template] of Object.entries(await load<ThemeModule>(entry))) {
      let result = typeof template === "function" ? template(options) : template;
      result = applyDefaultVariants!(result, options.theme.defaultVariants);
      result = applyUnstyled!(result, options.theme.unstyled);
      result = applyPrefixToObject!(result, options.theme.prefix);
      write(`${dir}/${kebabCase(name)}.ts`, themeTemplate(result));
      names[dir]!.push(name);
    }
  }
  // The index re-exports, as the module's `ui/index.ts` and `ui/prose/index.ts` templates write them.
  const reexport = (directory: string) => (name: string) =>
    `export { default as ${name} } from ${JSON.stringify(`./${directory}${kebabCase(name)}`)}`;
  write("ui/prose/index.ts", names["ui/prose"]!.map(reexport("")).join("\n"));
  write(
    "ui/index.ts",
    [
      ...names.ui!.map(reexport("")),
      ...names["ui/content"]!.map(reexport("content/")),
      "export * as prose from './prose'",
    ].join("\n"),
  );
  // No NuxtImg component is registered, so the template falls back to a plain <img>.
  write("ui-image-component.ts", 'export default "img"');
  const ui = getDefaultConfig(options.theme);
  write("types/ui.d.ts", uiTypesTemplate(options.theme.colors, Object.keys(ui.icons)));
  write("types/app.config.d.ts", appConfigTypesTemplate({ nuxt: {}, ui }));
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
