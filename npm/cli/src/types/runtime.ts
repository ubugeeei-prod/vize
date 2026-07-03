import type { LanguageServerConfig, VizeConfig, VizeConfigEntry, VueVersion } from "./generated.js";

// ============================================================================
// TS-specific runtime types (cannot be expressed in Pkl)
// ============================================================================

export type MaybePromise<T> = T | Promise<T>;

export interface ConfigEnv {
  mode: string;
  command: "serve" | "build" | "check" | "lint" | "fmt";
  isSsrBuild?: boolean;
}

export type VueConfig = {
  /**
   * Vue dialect selected by the documented `vue.version` config key.
   */
  version?: VueVersion;
};

export type UserConfigEntry = VizeConfigEntry & {
  /**
   * Shared Vue dialect section accepted by the Rust config model.
   */
  vue?: VueConfig;
};

export type UserConfig = Omit<VizeConfig, "entries"> & {
  /**
   * Shared Vue dialect section accepted by the Rust config model.
   */
  vue?: VueConfig;
  /**
   * Legacy alias for `languageServer`.
   * Prefer `languageServer`.
   */
  lsp?: LanguageServerConfig;
  /**
   * Scoped config entries for monorepos and workspaces.
   */
  entries?: UserConfigEntry[];
};

export type UserConfigInput = UserConfig | UserConfigEntry[];

export type ResolvedVizeConfig = VizeConfig & {
  /**
   * Normalized flat entries. Plain object configs become one entry; array configs
   * keep their order.
   */
  entries: VizeConfigEntry[];
};

export type UserConfigExport =
  | UserConfigInput
  | ((env: ConfigEnv) => MaybePromise<UserConfigInput>);

// ============================================================================
// LoadConfigOptions
// ============================================================================

export interface LoadConfigOptions {
  /**
   * Config file search mode
   * - 'root': Search only in the specified root directory
   * - 'auto': Search from cwd upward until finding a config file
   * - 'none': Don't load config file
   * @default 'root'
   */
  mode?: "root" | "auto" | "none";

  /**
   * Custom config file path (overrides automatic search)
   */
  configFile?: string;

  /**
   * Config environment for dynamic config resolution
   */
  env?: ConfigEnv;
}
