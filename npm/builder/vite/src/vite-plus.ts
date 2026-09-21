import type { UserConfigExport } from "./types.ts";
import type { PluginOption } from "vite-plus";
import type { VitePlusConfig, VizePlusConfigFactory, VizePlusOptions } from "./vite-plus/types.ts";
import { taskConfigKey } from "./vite-plus/types.ts";
import { configureTools } from "./vite-plus/conflicts.ts";
import { createTasks } from "./vite-plus/tasks.ts";

export type {
  VitePlusConfig,
  VizePlusConfigFactory,
  VizePlusOptions,
  VizeTask,
} from "./vite-plus/types.ts";

/** One shared Vize config, with optional configuration owned and typed by Vite+. */
export function withVue(
  config?: UserConfigExport,
  options: VizePlusOptions = {},
): VizePlusConfigFactory {
  function create(vpConfig: VitePlusConfig = {}): VizePlusConfigFactory {
    const factory: VizePlusConfigFactory = Object.assign(
      async (env: Parameters<VizePlusConfigFactory>[0]) => {
        const resolved = await (typeof vpConfig === "function" ? vpConfig(env) : vpConfig);
        const plugins =
          options.plugin === false
            ? [...(resolved.plugins ?? [])]
            : await withoutVueCompiler(resolved.plugins ?? []);
        if (options.plugin !== false) {
          const { vize } = await import("./plugin/index.ts");
          plugins.unshift(vize({ ...options.plugin, config }));
        }
        return {
          ...configureTools(resolved, options),
          plugins,
          run:
            options.tasks === false
              ? resolved.run
              : {
                  ...resolved.run,
                  tasks: {
                    ...createTasks(resolved.run?.tasks, options.tasks),
                    ...resolved.run?.tasks,
                  },
                },
          [taskConfigKey]: { config, options },
        };
      },
      { vp: create },
    );
    return factory;
  }
  return create();
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

/** Alias for projects that prefer the Vize toolchain name. */
export { withVue as withVize };
