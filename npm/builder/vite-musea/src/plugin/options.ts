import type { MuseaOptions } from "../types/index.js";

const MUSEA_OPTIONS_KEY = "__vizeMuseaOptions";

type MuseaPluginOptionsCarrier = {
  [MUSEA_OPTIONS_KEY]?: MuseaOptions;
};

export function attachMuseaOptions<T extends object>(plugin: T, options: MuseaOptions): T {
  Object.defineProperty(plugin, MUSEA_OPTIONS_KEY, {
    configurable: false,
    enumerable: false,
    value: options,
    writable: false,
  });
  return plugin;
}

export function readMuseaOptions(plugin: unknown): MuseaOptions | undefined {
  if (!plugin || typeof plugin !== "object") return undefined;
  return (plugin as MuseaPluginOptionsCarrier)[MUSEA_OPTIONS_KEY];
}
