type ConfigEnv = { mode: string; command: string; isSsrBuild?: boolean; isPreview?: boolean };
export const VITE_CONFIG_FILE_NAMES: readonly string[];
export function isViteConfigFile(filePath: string): boolean;
export function projectConfigDefaults(): { typeChecker: { jsxTypecheck: boolean } };
/** Validated canonical selection; no source or legacy callback evaluation. */
export function vueConfigOptions<T extends { vue?: unknown }>(source: T): T["vue"] | undefined;
/** Canonical arrays and each rule policy replace; explicit false wins and undefined stays absent. */
export function mergeVueConfigOptions<T>(base: T | undefined, own: T | undefined): T | undefined;
/** Project an already evaluated Vite source, resolving only its legacy native export. */
export function projectViteConfigSource(
  source: unknown,
  env?: ConfigEnv,
  configDir?: string,
): Promise<unknown>;
export function resolveViteConfigExport(
  exported: unknown,
  env?: ConfigEnv,
  configDir?: string,
): Promise<unknown>;
