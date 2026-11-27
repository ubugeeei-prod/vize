import type {
  NuxtComponentBuildMode,
  NuxtComponentDescriptor,
  NuxtComponentImport,
} from "./components.ts";
import { createNuxtComponentResolver } from "./components.ts";

type ComponentResolver = ReturnType<typeof createNuxtComponentResolver>;
type ResolveComponent = (name: string) => NuxtComponentImport | null;

interface NuxtProjectOptions {
  buildDir: string;
  dev?: boolean;
  modules: unknown[];
  rootDir: string;
}

interface ModuleGraphLike {
  getModuleById(id: string): unknown;
}

interface HotUpdateContext {
  environment?: { moduleGraph: ModuleGraphLike };
}

interface HotUpdateOptions {
  type?: "create" | "update" | "delete";
  modules: unknown[];
}

/** The component names one compiled module resolved, and what each resolved to. */
interface ModuleResolutions {
  mode: NuxtComponentBuildMode;
  keys: Map<string, string>;
}

// Nuxt fills in the extension of a component path after `components:extend`
// returns, so one scan can name `client-only` and the next `client-only.js`.
const COMPONENT_FILE_EXT_RE = /\.(?:[cm]?[jt]s|vue)$/;

function resolutionKey(resolved: NuxtComponentImport | null): string {
  if (!resolved) return "";
  const file = resolved.filePath.replace(COMPONENT_FILE_EXT_RE, "");
  return `${file}#${resolved.exportName}#${resolved.lazy ? "lazy" : ""}#${resolved.mode ?? ""}`;
}

/**
 * The bridge resolves Nuxt components inside Vize's compiled modules, where
 * Nuxt's own loader never runs, so Nuxt cannot know which of those modules
 * named a component. When a component file appears or disappears, Nuxt
 * rescans its components before later hot-update hooks run; this then reloads
 * every compiled module whose component names now resolve differently. A page
 * saved together with a new component then imports it, and a page whose
 * component was renamed or deleted loses the import and warns instead of
 * failing to load, as with Nuxt's loader and @vitejs/plugin-vue.
 *
 * Resolutions are only recorded in dev, so builds pay nothing for this.
 */
export function createNuxtComponentBridge(nuxtOptions: NuxtProjectOptions) {
  const resolver: ComponentResolver = createNuxtComponentResolver({
    buildDir: nuxtOptions.buildDir,
    moduleNames: nuxtOptions.modules.filter(
      (moduleName): moduleName is string => typeof moduleName === "string",
    ),
    rootDir: nuxtOptions.rootDir,
  });
  const dev = nuxtOptions.dev === true;
  // Per environment, because the client and the server resolve the same id
  // in different modes, and each has its own module graph.
  const recorded = new WeakMap<object, Map<string, ModuleResolutions>>();

  return {
    register(components: NuxtComponentDescriptor[]): void {
      resolver.register(components);
    },

    /** The resolver for one transform of `id`, which records what it resolves in dev. */
    resolverFor(
      environment: object | undefined,
      id: string,
      mode: NuxtComponentBuildMode,
    ): ResolveComponent {
      if (!dev || !environment) return (name) => resolver.resolve(name, mode);
      const modules = recorded.get(environment) ?? new Map<string, ModuleResolutions>();
      recorded.set(environment, modules);
      // A new transform replaces what the previous output of `id` resolved,
      // and a module that resolves nothing is not kept.
      modules.delete(id);
      let keys: Map<string, string> | undefined;
      return (name) => {
        const resolved = resolver.resolve(name, mode);
        if (!keys) modules.set(id, { mode, keys: (keys = new Map()) });
        keys.set(name, resolutionKey(resolved));
        return resolved;
      };
    },

    hotUpdate: {
      handler(this: HotUpdateContext, { type, modules }: HotUpdateOptions): unknown[] | void {
        if (type === "update" || !this.environment) return;
        const records = recorded.get(this.environment);
        if (!records) return;
        const affected = new Set(modules);
        for (const [id, { mode, keys }] of records) {
          let changed = false;
          for (const [name, key] of keys) {
            if (resolutionKey(resolver.resolve(name, mode)) !== key) {
              changed = true;
              break;
            }
          }
          if (!changed) continue;
          // Its next transform records again; a module no page loads again
          // must not be reported on every later rescan.
          records.delete(id);
          const module = this.environment.moduleGraph.getModuleById(id);
          if (module) affected.add(module);
        }
        if (affected.size !== modules.length) return [...affected];
      },
    },
  };
}
