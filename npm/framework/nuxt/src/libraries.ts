import { createRequire } from "node:module";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

/** Options for auto-registering `@vizejs/ui` components (and its `use*` helpers). */
export interface VizeNuxtUiOptions {
  /** Component name prefix, for example `"Vz"` for `<VzButton>`. @default "" */
  prefix?: string;
  /** Only register these families (package subpaths without `./`). @default every family */
  include?: string[];
  /** Never register these families. @default [] */
  exclude?: string[];
  /** Also auto-import the `use*` helpers exported by the included families. @default true */
  composables?: boolean;
  /** Resolve to the package or to `vize lib pull` copies. @default "package" */
  source?: "local" | "package";
  /** Lockfile path relative to the Nuxt root. @default "vize-lib.lock.json" */
  lockfile?: string;
  /** In local mode, fall back to the package for families that were not pulled. @default true */
  fallback?: boolean;
}

/** Options for auto-importing `@vizejs/composable` exports. */
export interface VizeNuxtComposablesOptions {
  /** Only import from these entries (for example `"use-toggle"`). @default every entry */
  include?: string[];
  /** Never import from these entries. @default [] */
  exclude?: string[];
  /** Only import these export names. @default every runtime export */
  names?: string[];
  /** Resolve to the package or to `vize lib pull` copies. @default "package" */
  source?: "local" | "package";
  /** Lockfile path relative to the Nuxt root. @default "vize-lib.lock.json" */
  lockfile?: string;
  /** In local mode, fall back to the package for entries that were not pulled. @default true */
  fallback?: boolean;
}

/** A resolved import (`import { name as as } from from`). */
interface ResolvedImport {
  readonly name: string;
  readonly as?: string;
  readonly from: string;
}

/** The resolver surface this module needs from `@vizejs/ui/resolver`. */
export interface UiResolverModule {
  listVizeUiComponents(options: Record<string, unknown>): readonly ResolvedImport[];
  listVizeUiComposables(options: Record<string, unknown>): readonly ResolvedImport[];
}

/** The resolver surface this module needs from `@vizejs/composable/resolver`. */
export interface ComposableResolverModule {
  listVizeComposableImports(options: Record<string, unknown>): readonly ResolvedImport[];
}

/** Loaders for the resolver modules (injectable for tests). */
export interface VizeLibraryLoaders {
  ui: () => Promise<UiResolverModule>;
  composables: () => Promise<ComposableResolverModule>;
}

/** Components and imports to register with Nuxt. */
export interface VizeLibraryPlan {
  /** `addComponent` inputs: registered tag name, export name, and module. */
  components: { name: string; export: string; filePath: string }[];
  /** `addImports` inputs. */
  imports: { name: string; from: string }[];
}

function projectLoader<Module>(rootDir: string, specifier: string, option: string) {
  return async (): Promise<Module> => {
    const projectRequire = createRequire(join(rootDir, "package.json"));
    let resolved: string;
    try {
      resolved = projectRequire.resolve(specifier);
    } catch (error) {
      throw new Error(
        `[@vizejs/nuxt] \`vize.${option}\` needs ${specifier.replace(/\/resolver$/, "")} installed in the project`,
        { cause: error },
      );
    }
    return (await import(pathToFileURL(resolved).href)) as Module;
  };
}

/** Resolve resolver modules from the Nuxt project, so the project's installed versions drive registration. */
export function createProjectLibraryLoaders(rootDir: string): VizeLibraryLoaders {
  return {
    ui: projectLoader<UiResolverModule>(rootDir, "@vizejs/ui/resolver", "ui"),
    composables: projectLoader<ComposableResolverModule>(
      rootDir,
      "@vizejs/composable/resolver",
      "composables",
    ),
  };
}

function withoutUndefined(options: object): Record<string, unknown> {
  return Object.fromEntries(Object.entries(options).filter(([, value]) => value !== undefined));
}

/**
 * Plan component and import registrations from the library catalogs.
 * Everything resolves to direct family/entry subpaths, so tree shaking is
 * identical to hand-written imports; Nuxt generates the typed `.d.ts` files.
 */
export async function planVizeLibraryIntegration(
  options: {
    ui?: boolean | VizeNuxtUiOptions;
    composables?: boolean | VizeNuxtComposablesOptions;
  },
  rootDir: string,
  loaders: VizeLibraryLoaders = createProjectLibraryLoaders(rootDir),
): Promise<VizeLibraryPlan> {
  const plan: VizeLibraryPlan = { components: [], imports: [] };
  if (options.ui !== undefined && options.ui !== false) {
    const ui = options.ui === true ? {} : options.ui;
    const resolverOptions = withoutUndefined({
      prefix: ui.prefix,
      include: ui.include,
      exclude: ui.exclude,
      source: ui.source,
      lockfile: ui.lockfile,
      fallback: ui.fallback,
      root: rootDir,
    });
    const module = await loaders.ui();
    for (const item of module.listVizeUiComponents(resolverOptions)) {
      plan.components.push({ name: item.as ?? item.name, export: item.name, filePath: item.from });
    }
    if (ui.composables !== false) {
      for (const item of module.listVizeUiComposables(resolverOptions)) {
        plan.imports.push({ name: item.name, from: item.from });
      }
    }
  }
  if (options.composables !== undefined && options.composables !== false) {
    const composables = options.composables === true ? {} : options.composables;
    const module = await loaders.composables();
    const taken = new Set(plan.imports.map((item) => item.name));
    for (const item of module.listVizeComposableImports(
      withoutUndefined({ ...composables, root: rootDir }),
    )) {
      // A name exported by both libraries keeps the first (ui) registration.
      if (!taken.has(item.name)) plan.imports.push({ name: item.name, from: item.from });
    }
  }
  return plan;
}

/** The `@nuxt/kit` functions registration uses. */
export interface VizeLibraryKit {
  addComponent(options: { name: string; export: string; filePath: string }): unknown;
  addImports(imports: { name: string; from: string }[]): unknown;
}

/** Register the planned components and imports with Nuxt. */
export async function setupVizeLibraries(
  options: {
    ui?: boolean | VizeNuxtUiOptions;
    composables?: boolean | VizeNuxtComposablesOptions;
  },
  rootDir: string,
  loadKit: () => Promise<VizeLibraryKit>,
  loaders?: VizeLibraryLoaders,
): Promise<VizeLibraryPlan> {
  const enabled =
    (options.ui !== undefined && options.ui !== false) ||
    (options.composables !== undefined && options.composables !== false);
  if (!enabled) return { components: [], imports: [] };
  const plan = await planVizeLibraryIntegration(options, rootDir, loaders);
  const kit = await loadKit();
  for (const component of plan.components) kit.addComponent(component);
  if (plan.imports.length > 0) kit.addImports(plan.imports);
  return plan;
}
