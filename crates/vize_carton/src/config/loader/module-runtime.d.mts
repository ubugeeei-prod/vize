type ConfigModule = Record<string, unknown>;

export function configModuleImportPath(configURL: URL, windows?: boolean): string;
export function importConfigModule(configPath: string): Promise<ConfigModule>;
export function importConfigModule<T>(
  configPath: string,
  withoutVite: (configURL: URL) => T | Promise<T>,
): Promise<ConfigModule | T>;
