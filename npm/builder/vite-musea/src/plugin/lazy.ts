import type { Plugin, ResolvedConfig } from "vite";
import type { MuseaOptions } from "../types/index.js";
import { shouldApplyMuseaPlugin } from "./apply.js";
import { attachMuseaOptions } from "./options.js";

type RuntimePlugins = ReturnType<(typeof import("./index.js"))["musea"]>;

/**
 * Keep config-only Vite+ commands on a dependency-free entry path. The real
 * plugin graph is loaded only after Vite invokes a Musea hook.
 */
export function musea(options: MuseaOptions = {}): Plugin[] {
  let runtimePlugins: RuntimePlugins | undefined;
  let loading: Promise<RuntimePlugins> | undefined;

  const load = async (): Promise<RuntimePlugins> => {
    if (runtimePlugins) return runtimePlugins;
    const pending = (loading ??= import("./index.js").then(({ musea: createRuntimePlugins }) =>
      createRuntimePlugins(options),
    ));
    try {
      return (runtimePlugins = await pending);
    } finally {
      if (!runtimePlugins) loading = undefined;
    }
  };

  const callHook = async (
    pluginIndex: number,
    hookName: string,
    context: unknown,
    args: unknown[],
  ) => {
    const plugin = (await load())[pluginIndex];
    const hook = plugin && Reflect.get(plugin, hookName);
    const handler =
      typeof hook === "function"
        ? hook
        : hook && typeof hook === "object"
          ? Reflect.get(hook, "handler")
          : undefined;
    if (typeof handler !== "function") return undefined;
    return Reflect.apply(handler, context, args);
  };

  const capability = Object.freeze({
    version: 1,
    forConfig(config: ResolvedConfig) {
      const api = runtimePlugins?.[0]?.api;
      const owner = api && Reflect.get(api, "vizePrecompileInputOwner");
      const forConfig = owner && Reflect.get(owner, "forConfig");
      return typeof forConfig === "function"
        ? Reflect.apply(forConfig, owner, [config])
        : undefined;
    },
  });

  const main = attachMuseaOptions<Plugin>(
    {
      name: "vite-plugin-musea",
      enforce: "pre",
      api: { vizePrecompileInputOwner: capability },
      apply(_config, environment) {
        return shouldApplyMuseaPlugin(environment);
      },
      config(config, environment) {
        return callHook(0, "config", this, [config, environment]);
      },
      options(input) {
        return callHook(0, "options", this, [input]);
      },
      configResolved(config) {
        return callHook(0, "configResolved", this, [config]);
      },
      configureServer(server) {
        return callHook(0, "configureServer", this, [server]);
      },
      buildStart(input) {
        return callHook(0, "buildStart", this, [input]);
      },
      resolveId(source, importer, resolveOptions) {
        return callHook(0, "resolveId", this, [source, importer, resolveOptions]);
      },
      load(id) {
        return callHook(0, "load", this, [id]);
      },
      generateBundle(outputOptions, bundle, isWrite) {
        return callHook(0, "generateBundle", this, [outputOptions, bundle, isWrite]);
      },
      transform(code, id, transformOptions) {
        return callHook(0, "transform", this, [code, id, transformOptions]);
      },
      handleHotUpdate(context) {
        return callHook(0, "handleHotUpdate", this, [context]);
      },
    },
    options,
  );

  const stylePostTransform: Plugin = {
    name: "vite-plugin-musea:style-post-transform",
    transform(code, id, transformOptions) {
      return callHook(1, "transform", this, [code, id, transformOptions]);
    },
  };

  return [main, stylePostTransform];
}
