export function enableConfigCompileCache(directory?: string): unknown;
type ConfigModule = Record<string, unknown>;

/** Private loader-host clocks in milliseconds; module_total encloses the other rows. */
export function configModuleEvaluationTimings(value: unknown):
  | Array<{
      phase: string;
      elapsedMs: number;
      processId: number;
      cpuProfile?:
        | {
            status: "recorded";
            setupMs: number;
            stopMs: number;
            profile: import("node:inspector").Profiler.Profile;
          }
        | { status: "unavailable"; setupMs?: number; reason: string };
    }>
  | undefined;
export function configModuleFailureArtifacts(error: unknown): Array<[string, string]>;
export function configModuleImportPath(configURL: URL, windows?: boolean): string;
export function failedConfigModuleRequests(error: unknown): string[];
export function importConfigModule(configPath: string): Promise<ConfigModule>;
export function importConfigModule<T>(
  configPath: string,
  withoutVite: (configURL: URL) => T | Promise<T>,
): Promise<ConfigModule | T>;

export function importConfigModuleWithDependencies<
  Fallback = never,
  Value = ConfigModule | Awaited<Fallback>,
>(
  configPath: string,
  registerModuleHooks?: typeof import("node:module").registerHooks | null,
  withoutVite?: (configURL: URL) => Fallback | Promise<Fallback>,
  evaluate?: (namespace: ConfigModule | Awaited<Fallback>) => Value | Promise<Value>,
  observeDependencies?: (dependencies: string[], complete: boolean) => void,
  beforeRead?: (paths: string[]) => void | Promise<void>,
): Promise<{
  namespace: ConfigModule | Awaited<Fallback>;
  value: Awaited<Value>;
  dependencies: string[];
}>;
