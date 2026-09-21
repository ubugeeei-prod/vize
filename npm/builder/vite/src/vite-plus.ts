import type { UserConfigExport } from "./types.ts";
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
export function withVize(
  config?: UserConfigExport,
  options: VizePlusOptions = {},
): VizePlusConfigFactory {
  function create(vpConfig: VitePlusConfig = {}): VizePlusConfigFactory {
    const factory: VizePlusConfigFactory = Object.assign(
      async (env: Parameters<VizePlusConfigFactory>[0]) => {
        const resolved = await (typeof vpConfig === "function" ? vpConfig(env) : vpConfig);
        const plugins = [...(resolved.plugins ?? [])];
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
