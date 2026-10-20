import type { LanguageServerConfig, VizeConfig, VizeConfigEntry, VueVersion } from "./generated.js";

// ============================================================================
// TS-specific runtime types (cannot be expressed in Pkl)
// ============================================================================

export type MaybePromise<T> = T | Promise<T>;

export interface ConfigEnv {
  mode: string;
  command: "serve" | "build" | "check" | "lint" | "fmt";
  isSsrBuild?: boolean;
  isPreview?: boolean;
}

export type VueConfig = {
  /**
   * Vue dialect selected by the documented `vue.version` config key.
   */
  version?: VueVersion;
};

/** Native Vue settings shared by Vite, Vite+, the CLI projection and the editor. */
type VueOptionKeys =
  | "name"
  | "compiler"
  | "experimentals"
  | "dialect"
  | "vite"
  | "musea"
  | "typeChecker"
  | "linter"
  | "formatter"
  | "languageServer"
  | "globalTypes"
  | "files"
  | "ignores"
  | "basePath"
  | "lib";

/** Host Vite+ lint config produced by `oxlint-plugin-vize` flat-config helpers. */
export interface VueHostLintConfig extends Record<string, unknown> {
  jsPlugins: string[];
  plugins: string[];
  rules: Record<string, unknown>;
  settings: Record<string, unknown> & {
    vize?: {
      locale?: unknown;
      helpLevel?: unknown;
      preset?: unknown;
      typeAware?: unknown;
      rules?: unknown;
    };
  };
  ignorePatterns?: string[];
  overrides?: unknown[];
}

/** Preferred action-oriented spellings for the Vue tools. */
type VueToolActions = {
  compile?: VizeConfig["compiler"];
  typeCheck?: VizeConfig["typeChecker"];
  lint?: VueHostLintConfig;
  fmt?: VizeConfig["formatter"];
};

/** Native scopes retain the same authored options inside the Vue namespace. */
export type VueOptionsEntry = Pick<
  VizeConfigEntry,
  "name" | "basePath" | "files" | "ignores" | "compiler" | "linter"
>;

/** Section option values reuse the native types rather than a second schema. */
export type VueOptions = Pick<VizeConfig, VueOptionKeys> & {
  version?: VueVersion;
  entries?: VueOptionsEntry[];
} & VueToolActions;

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

  /** Read Vite settings during discovery. Integrations with an already loaded Vite config use false. */
  viteConfig?: boolean;

  /**
   * Config environment for dynamic config resolution
   */
  env?: ConfigEnv;
}
