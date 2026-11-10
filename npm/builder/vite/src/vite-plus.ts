import type { PluginOption } from "vite-plus";
import type {
  ConfigWithVizeTasks,
  VitePlusConfig,
  VizePlusConfigFactory,
  VizePlusOptions,
  VueConfig,
  VueConfigObject,
  VueOptions,
} from "./vite-plus/types.ts";
import { sourceConfigKey, taskConfigKey } from "./vite-plus/types.ts";
import { configureTools } from "./vite-plus/conflicts.ts";
import { createTasks } from "./vite-plus/tasks.ts";
import { normalizeConfig, resolveConfig } from "./vite-plus/config.ts";
import { configurePack } from "./vite-plus/pack.ts";
import { mergeVueConfigOptions } from "../../../cli/src/config/vite-runtime.mjs";

export type {
  ConfigWithVizeTasks,
  VitePlusConfig,
  VizePlusConfigFactory,
  VizePlusOptions,
  VizeTask,
  VueConfig,
  VueOptions,
  VueOptionsEntry,
  VueConfigObject,
  VizeCompilerOptions,
  VizeLintOptions,
  VizePackOptions,
} from "./vite-plus/types.ts";

export type StaticVizePlusOptions = Omit<VizePlusOptions, "taskCache" | "tasks">;

/** Add the native Vue toolchain to an ordinary Vite+ export. */
export function defineConfig(
  config: VueConfig = {},
  integration: VizePlusOptions = {},
): VizePlusConfigFactory {
  return Object.assign(
    async (env: Parameters<VizePlusConfigFactory>[0]) => {
      const source = await resolveConfig(config, env);
      return configureResolvedConfig(source, integration);
    },
    {
      [sourceConfigKey]: config,
      withVue(vue: VueOptions): VizePlusConfigFactory {
        return defineConfig(async (env) => {
          const source = await resolveConfig(config, env);
          return { ...source, vue: mergeVueConfigOptions(source.vue, vue) };
        }, integration);
      },
    },
  );
}

/**
 * Configure a literal Vite+ object without wrapping its outer `defineConfig` call.
 *
 * Keep `run.tasks` literal in the caller so Vite+ can extract it without evaluating
 * the config module. Generated tasks are deliberately disabled on this path; use
 * the stable `vize-vp-task` package binary in those literal task commands.
 */
export async function vitePlusConfig(
  config: VueConfigObject = {},
  integration: StaticVizePlusOptions = {},
): Promise<ConfigWithVizeTasks> {
  return configureResolvedConfig(config, { ...integration, tasks: false });
}

async function configureResolvedConfig(
  source: VueConfigObject,
  integration: VizePlusOptions,
): Promise<ConfigWithVizeTasks> {
  const { vp, metadata, compiler, plugin } = normalizeConfig(source, integration);
  const { options } = metadata;
  const resolvingMetadata = process.env["VP_RESOLVING_CONFIG_METADATA"] === "1";
  const legacy =
    !resolvingMetadata &&
    compiler !== false &&
    (await import("./plugin/vue-version.ts")).isLegacyVueCompatibilityMode(compiler);
  const plugins =
    resolvingMetadata || compiler === false || legacy
      ? [...(vp.plugins ?? [])]
      : await withoutVueCompiler(vp.plugins ?? []);
  if (compiler !== false && !resolvingMetadata) {
    const { vize } = await import("./plugin/index.ts");
    // The optional Vite+ peer may expose a different Vite type instance.
    // User pre-plugins must be able to resolve imports before the compiler.
    plugins.push(
      vize({
        ...plugin,
        configMode: false,
        config: metadata.config,
      }) as unknown as PluginOption,
    );
  }
  return {
    ...configureTools(vp, options, !resolvingMetadata),
    plugins,
    pack: await configurePack(source.pack, compiler, metadata),
    run:
      options.tasks === false
        ? vp.run
        : {
            ...vp.run,
            tasks: {
              ...createTasks(vp.run?.tasks, options.tasks, options.taskCache),
              ...vp.run?.tasks,
            },
          },
    [sourceConfigKey]: source,
    [taskConfigKey]: metadata,
  };
}

async function withoutVueCompiler(plugins: PluginOption[]): Promise<PluginOption[]> {
  const result: PluginOption[] = [];
  for (const option of plugins) {
    const plugin = await option;
    if (Array.isArray(plugin)) result.push(...(await withoutVueCompiler(plugin)));
    else if (plugin && plugin.name !== "vite:vue") result.push(plugin);
  }
  return result;
}

/** Add Vue settings to an existing Vite+ export without another Vite+ injection. */
export function withVue(
  vue: VueOptions,
  vpConfig: VitePlusConfig = {},
  integration: VizePlusOptions = {},
): VizePlusConfigFactory {
  return defineConfig(vpConfig, integration).withVue(vue);
}

/** Compatibility alias for the original config-first helper. */
export { defineConfig as withVize };
