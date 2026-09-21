import type { UserConfig } from "vite-plus";
import type { VizeCompilerOptions, VizeTaskConfig, VueConfigObject } from "./types.ts";
import { emitDeclarations } from "./declarations.ts";
import { resolveConfigExport } from "../config.ts";

type Pack = Exclude<NonNullable<UserConfig["pack"]>, unknown[]>;

export async function configurePack(
  input: VueConfigObject["pack"],
  compiler: VizeCompilerOptions | false,
  metadata: VizeTaskConfig,
): Promise<UserConfig["pack"]> {
  if (input === undefined) return undefined;
  if (Array.isArray(input)) {
    return Promise.all(
      input.map(async (entry) => (await configurePack(entry, compiler, metadata)) as Pack),
    );
  }
  const { vize: native, ...pack } = input;
  if (native === false) return pack;
  const sourcemap = native?.sourcemap ?? pack.sourcemap;
  const plugins = Array.isArray(pack.plugins)
    ? [...pack.plugins]
    : pack.plugins
      ? [pack.plugins]
      : [];
  if (compiler !== false) {
    const { default: vize } = await import("@vizejs/unplugin/rolldown");
    const shared = metadata.config
      ? await resolveConfigExport(metadata.config, { command: "build", mode: "production" })
      : undefined;
    plugins.unshift(
      vize({
        ...shared?.compiler,
        compatibility: compiler.compatibility,
        ...compiler,
        isProduction: true,
        sourceMap: native?.sourcemap ?? compiler.sourceMap ?? Boolean(sourcemap),
      }),
    );
  }
  if (!native || native.dts === false) return { ...pack, sourcemap, plugins };
  return {
    ...pack,
    sourcemap,
    plugins,
    dts: false,
    async hooks(hooks) {
      if (typeof pack.hooks === "function") await pack.hooks(hooks);
      else if (pack.hooks) hooks.addHooks(pack.hooks);
      hooks.hook("build:done", async (context) => {
        await emitDeclarations(
          {
            ...native,
            tsconfig:
              native.tsconfig ?? (typeof pack.tsconfig === "string" ? pack.tsconfig : undefined),
            declarationDir: native.declarationDir ?? context.options.outDir,
          },
          metadata,
        );
      });
    },
  };
}
