import type { ConfigEnv, UserConfig, defineConfig } from "vite-plus";
import type { UserConfigExport, VizeOptions } from "../types.ts";
import type {
  CompilerConfig,
  FormatterConfig,
  LinterConfig,
  TypeCheckerConfig,
  UserConfig as NativeConfig,
} from "../../../../cli/src/types/index.ts";

/** Inherit configuration additions from the consumer's Vite+ version. */
export type VitePlusConfig = Parameters<typeof defineConfig>[0];
export type VizeCompilerOptions = Omit<CompilerConfig, "compatibility"> &
  Omit<VizeOptions, "config">;
export interface VizeLintOptions extends LinterConfig {
  /** Also run the native typechecker before linting. @default false */
  typecheck?: boolean;
}
export interface VizePackOptions {
  /** Emit Vue and TypeScript declarations after the bundle succeeds. @default true */
  dts?: boolean;
  /** Override the native declaration output directory. Defaults to pack.outDir. */
  declarationDir?: string;
  /** Emit declaration maps pointing to authored sources. */
  declarationMap?: boolean;
  /** Enable compiler and bundle source maps. */
  sourcemap?: boolean;
  /** TypeScript project used for declaration generation. */
  tsconfig?: string;
}
type WithPack<T> = T extends (infer Item)[]
  ? WithPack<Item>[]
  : T & { vize?: VizePackOptions | false };
export type VizeSharedConfig = NativeConfig & {
  /** Alias of lint.vize; lint.vize takes precedence when both are provided. */
  lint?: VizeLintOptions | boolean;
};
export interface VueIntegrationConfig {
  extends?: VueConfig | VueConfig[];
  compiler?: VizeCompilerOptions | false;
  typecheck?: TypeCheckerConfig | boolean;
  lint?: UserConfig["lint"] & { vize?: VizeLintOptions | boolean };
  fmt?: UserConfig["fmt"] & { vize?: FormatterConfig | boolean };
  pack?: WithPack<NonNullable<UserConfig["pack"]>>;
  /** Shared native settings, including scopes, globals and editor settings. */
  vize?: VizeSharedConfig | UserConfigExport;
}
type WithVize<T> = T extends (...args: infer Args) => infer Result
  ? (...args: Args) => WithVize<Result>
  : T extends Promise<infer Result>
    ? Promise<WithVize<Result>>
    : Omit<T, "lint" | "fmt" | "pack"> & VueIntegrationConfig;
export type VueConfig = WithVize<VitePlusConfig>;
export type VueConfigObject = Omit<UserConfig, "lint" | "fmt" | "pack"> & VueIntegrationConfig;
export type VizeTask =
  | "editor:setup"
  | "check"
  | "typecheck"
  | "lint"
  | "lint:fix"
  | "fmt"
  | "fmt:check"
  | "pack"
  | "build"
  | "dev"
  | "preview"
  | "test";

export interface VizePlusOptions {
  /** Compiler plugin options; prefer the compiler config section. */
  plugin?: Omit<VizeOptions, "config"> | false;
  check?: boolean;
  lint?: boolean;
  fmt?: boolean;
  /** Disable overlapping Oxlint rules and exclude Vue from Oxfmt. @default true */
  conflicts?: boolean;
  /** Rename or disable generated tasks. Existing run.tasks take precedence. */
  tasks?: false | Partial<Record<VizeTask, string | false>>;
}
export interface VizeTaskConfig {
  config?: UserConfigExport;
  options: VizePlusOptions;
  lintTypecheck?: boolean;
}
export type VizePlusConfigFactory = (env: ConfigEnv) => Promise<UserConfig>;
export const sourceConfigKey = Symbol.for("@vizejs/vite-plugin/vite-plus/source");
export const taskConfigKey = Symbol.for("@vizejs/vite-plugin/vite-plus");
export type ConfigWithVizeTasks = UserConfig & { [taskConfigKey]?: VizeTaskConfig };
