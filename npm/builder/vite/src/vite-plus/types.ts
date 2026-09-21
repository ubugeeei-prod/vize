import type { ConfigEnv, UserConfig, defineConfig } from "vite-plus";
import type { UserConfigExport, VizeOptions } from "../types.ts";

/** Inherit configuration additions from the consumer's Vite+ version. */
export type VitePlusConfig = Parameters<typeof defineConfig>[0];
export type VizeTask =
  | "check"
  | "lint"
  | "lint:fix"
  | "fmt"
  | "fmt:check"
  | "build"
  | "dev"
  | "preview"
  | "test";

export interface VizePlusOptions {
  /** Compiler plugin options, or false to keep an existing compiler. */
  plugin?: Omit<VizeOptions, "config"> | false;
  /** Run Vize's native typechecker in the check task. @default true */
  check?: boolean;
  /** Run native lint alongside Oxlint. @default true */
  lint?: boolean;
  /** Format Vue files with Vize and other files with Oxfmt. @default true */
  fmt?: boolean;
  /** Disable overlapping Oxlint rules and exclude Vue from Oxfmt. @default true */
  conflicts?: boolean;
  /** Rename or disable generated tasks. Existing run.tasks take precedence. */
  tasks?: false | Partial<Record<VizeTask, string | false>>;
}

export interface VizePlusConfigFactory {
  (env: ConfigEnv): Promise<UserConfig>;
  vp(config: VitePlusConfig): VizePlusConfigFactory;
}

export interface VizeTaskConfig {
  config?: UserConfigExport;
  options: VizePlusOptions;
}

export const taskConfigKey = Symbol.for("@vizejs/vite-plugin/vite-plus");
export type ConfigWithVizeTasks = UserConfig & { [taskConfigKey]?: VizeTaskConfig };
