type ConfigEnv = { mode: string; command: string; isSsrBuild?: boolean };
export const VITE_CONFIG_FILE_NAMES: readonly string[];
export function isViteConfigFile(filePath: string): boolean;
export function projectConfigDefaults(): { typeChecker: { jsxTypecheck: boolean } };
export function resolveViteConfigExport(exported: unknown, env?: ConfigEnv): Promise<unknown>;
