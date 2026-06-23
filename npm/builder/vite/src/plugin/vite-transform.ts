import * as vite from "vite";

type TransformOutput = { code: string; map?: unknown };
type OxcOptions = { lang: "ts"; sourcemap: false; target: "esnext" };
type EsbuildOptions = { loader: "ts"; sourcemap: false; target: "esnext" };
type TransformWithOxc = (
  code: string,
  id: string,
  options: OxcOptions,
) => TransformOutput | Promise<TransformOutput>;
type TransformWithEsbuild = (
  code: string,
  id: string,
  options: EsbuildOptions,
) => TransformOutput | Promise<TransformOutput>;

interface ViteTransformApi {
  transformWithOxc?: TransformWithOxc;
  transformWithEsbuild?: TransformWithEsbuild;
}

export function createVirtualTypeScriptTransformer(viteApi: ViteTransformApi) {
  return async (code: string, id: string): Promise<TransformOutput> => {
    // This pass only strips TypeScript from Vize virtual Vue modules.
    // Syntax lowering belongs to Vite's normal build target transform.
    if (typeof viteApi.transformWithOxc === "function") {
      return viteApi.transformWithOxc(code, id, {
        lang: "ts",
        sourcemap: false,
        target: "esnext",
      });
    }
    if (typeof viteApi.transformWithEsbuild === "function") {
      return viteApi.transformWithEsbuild(code, id, {
        loader: "ts",
        sourcemap: false,
        target: "esnext",
      });
    }
    throw new Error("Installed Vite does not expose transformWithOxc or transformWithEsbuild");
  };
}

export const transformVirtualTypeScript = createVirtualTypeScriptTransformer(vite);
