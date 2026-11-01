/** Companion plugins may own raw inputs that they turn into complete virtual SFCs. */
export type PrecompileInputOwner = (file: string, source: string) => boolean;

interface PrecompileConfig {
  plugins: readonly { name?: string; api?: unknown }[];
}

/** Use the actual environment's active plugins, including source/dist companions. */
export function precompileInputOwners(
  context: unknown,
  fallback?: PrecompileConfig,
): readonly PrecompileInputOwner[] {
  const environment = (
    context as {
      environment?: { getTopLevelConfig?: () => PrecompileConfig };
    }
  ).environment;
  const config = environment?.getTopLevelConfig?.() ?? fallback;
  return config ? snapshotPrecompileInputOwners(config) : [];
}

/** Freeze methods once per run; never infer ownership from a plugin or file name. */
export function snapshotPrecompileInputOwners(
  config: PrecompileConfig,
): readonly PrecompileInputOwner[] {
  const owners: PrecompileInputOwner[] = [];
  for (const plugin of config.plugins) {
    const api = plugin.api;
    if (api === null || typeof api !== "object") continue;
    const capability: unknown = Reflect.get(api, "vizePrecompileInputOwner");
    if (capability === undefined) continue;
    if (capability === null || typeof capability !== "object") {
      throw new Error("[vize] Invalid precompile input owner capability");
    }
    const version: unknown = Reflect.get(capability, "version");
    const forConfig: unknown = Reflect.get(capability, "forConfig");
    if (version !== 1 || typeof forConfig !== "function") {
      throw new Error("[vize] Unsupported precompile input owner capability");
    }
    const owner: unknown = Reflect.apply(forConfig, capability, [config]);
    if (owner === undefined) continue;
    if (owner === null || typeof owner !== "object") {
      throw new Error("[vize] Invalid resolved precompile input owner");
    }
    const ownsInput: unknown = Reflect.get(owner, "ownsInput");
    if (typeof ownsInput !== "function") {
      throw new Error("[vize] Invalid resolved precompile input owner");
    }
    owners.push((file, source) => {
      const result: unknown = Reflect.apply(ownsInput, owner, [file, source]);
      if (typeof result !== "boolean") {
        throw new Error("[vize] Precompile input ownership must return a boolean");
      }
      return result;
    });
  }
  return Object.freeze(owners);
}
