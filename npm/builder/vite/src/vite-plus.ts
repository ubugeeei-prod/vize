import type { PluginOption } from "vite-plus";
import type {
  VitePlusConfig,
  VizePlusConfigFactory,
  VizePlusOptions,
  VueConfig,
  VueOptions,
} from "./vite-plus/types.ts";
import { sourceConfigKey, taskConfigKey } from "./vite-plus/types.ts";
import { configureTools } from "./vite-plus/conflicts.ts";
import { createTasks } from "./vite-plus/tasks.ts";
import { normalizeConfig, resolveConfig } from "./vite-plus/config.ts";
import { configurePack } from "./vite-plus/pack.ts";
import { isLegacyVueCompatibilityMode } from "./plugin/vue-version.ts";

export type {
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

/** Extend the installed Vite+ configuration with the native Vue toolchain. */
export function defineConfig(
  config: VueConfig = {},
  integration: VizePlusOptions = {},
): VizePlusConfigFactory {
  return Object.assign(
    async (env: Parameters<VizePlusConfigFactory>[0]) => {
      const source = await resolveConfig(config, env);
      const { vp, metadata, compiler } = normalizeConfig(source, integration);
      const { options } = metadata;
      const plugins =
        compiler === false || isLegacyVueCompatibilityMode(compiler)
          ? [...(vp.plugins ?? [])]
          : await withoutVueCompiler(vp.plugins ?? []);
      if (compiler !== false) {
        const { vize } = await import("./plugin/index.ts");
        // The optional Vite+ peer may expose a different Vite type instance.
        // User pre-plugins must be able to resolve imports before the compiler.
        plugins.push(
          vize({
            ...compiler,
            configMode: false,
            config: metadata.config,
          }) as unknown as PluginOption,
        );
      }
      return {
        ...configureTools(vp, options),
        plugins,
        pack: await configurePack(source.pack, compiler, metadata),
        run:
          options.tasks === false
            ? vp.run
            : {
                ...vp.run,
                tasks: { ...createTasks(vp.run?.tasks, options.tasks), ...vp.run?.tasks },
              },
        [taskConfigKey]: metadata,
      };
    },
    { [sourceConfigKey]: config },
  );
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
  return defineConfig({ extends: vpConfig, vue }, integration);
}

/** Compatibility alias for the original config-first helper. */
export { defineConfig as withVize };
