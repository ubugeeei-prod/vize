import path from "node:path";

import { matchesNuxtLintCheckerFile } from "./filter.ts";
import type { ResolvedVizeNuxtLintCheckerOptions } from "./options.ts";
import {
  NuxtLintCheckerWorker,
  type NuxtLintCheckerResult,
  type NuxtLintCheckerTask,
} from "./worker.ts";

const PLUGIN_NAME = "VizeNuxtLintChecker";

interface WebpackCompilation {
  errors: Error[];
  fileDependencies: Set<string>;
  warnings: Error[];
}

interface WebpackCompiler {
  modifiedFiles?: Set<string>;
  hooks: {
    watchRun: { tap(name: string, callback: (compiler: WebpackCompiler) => void): void };
    afterCompile: {
      tapPromise(name: string, callback: (compilation: WebpackCompilation) => Promise<void>): void;
    };
    watchClose: { tap(name: string, callback: () => void): void };
  };
}

export interface NuxtLintCheckerWebpackPlugin {
  apply(compiler: WebpackCompiler): void;
}

export interface NuxtLintCheckerWebpackOptions {
  configFile: string;
  options: ResolvedVizeNuxtLintCheckerOptions;
  rootDir: string;
}

export interface NuxtLintCheckerWebpackRunner {
  close(): void | Promise<void>;
  run(task: NuxtLintCheckerTask): Promise<NuxtLintCheckerResult>;
}

export interface NuxtLintCheckerWebpackDependencies {
  createRunner?: () => NuxtLintCheckerWebpackRunner;
  oxlintEntrypoint?: string;
}

function checkerError(message: string): Error {
  const error = new Error(message);
  error.name = PLUGIN_NAME;
  return error;
}

function reportResult(compilation: WebpackCompilation, result: NuxtLintCheckerResult): void {
  if (result.hasErrors) compilation.errors.push(checkerError(result.output));
  else if (result.hasWarnings) compilation.warnings.push(checkerError(result.output));
}

function task(
  config: NuxtLintCheckerWebpackOptions,
  targets: string[],
  oxlintEntrypoint: string | undefined,
): NuxtLintCheckerTask {
  const options = config.options;
  return {
    configFile: config.configFile,
    cwd: config.rootDir,
    emitError: options.emitError,
    emitWarning: options.emitWarning,
    exclude: [...options.exclude],
    fix: options.fix,
    formatter: options.formatter,
    oxlintEntrypoint,
    targets,
  };
}

function changedTargets(
  compiler: WebpackCompiler,
  config: NuxtLintCheckerWebpackOptions,
  initial: boolean,
): string[] | undefined {
  const modified = compiler.modifiedFiles;
  if (initial && config.options.lintOnStart) return [...config.options.include];
  if (!modified || modified.size === 0) return undefined;
  if (!config.options.cache) return [...config.options.include];

  const configFile = path.resolve(config.configFile);
  if ([...modified].some((file) => path.resolve(file) === configFile)) {
    return [...config.options.include];
  }
  const files = [...modified]
    .filter((file) => matchesNuxtLintCheckerFile(file, config.rootDir, config.options))
    .map((file) => path.resolve(file))
    .sort();
  return files.length > 0 ? files : undefined;
}

/**
 * Create Nuxt 2's webpack adapter.
 *
 * The worker starts at `watchRun`, concurrently with compilation; only the
 * final diagnostic handoff happens at `afterCompile`, where webpack can feed
 * the result to its existing terminal and browser overlays.
 */
export function createNuxtLintCheckerWebpackPlugin(
  config: NuxtLintCheckerWebpackOptions,
  dependencies: NuxtLintCheckerWebpackDependencies = {},
): NuxtLintCheckerWebpackPlugin {
  return {
    apply(compiler) {
      const runner = dependencies.createRunner?.() ?? new NuxtLintCheckerWorker();
      let initial = true;
      let pending: Promise<NuxtLintCheckerResult> | undefined;

      compiler.hooks.watchRun.tap(PLUGIN_NAME, (nextCompiler) => {
        const targets = changedTargets(nextCompiler, config, initial);
        initial = false;
        pending = targets
          ? runner.run(task(config, targets, dependencies.oxlintEntrypoint))
          : undefined;
      });

      compiler.hooks.afterCompile.tapPromise(PLUGIN_NAME, async (compilation) => {
        compilation.fileDependencies.add(config.configFile);
        if (!pending) return;
        const current = pending;
        pending = undefined;
        try {
          reportResult(compilation, await current);
        } catch (error) {
          compilation.errors.push(
            checkerError(
              `Nuxt lint checker failed: ${error instanceof Error ? error.message : String(error)}`,
            ),
          );
        }
      });

      compiler.hooks.watchClose.tap(PLUGIN_NAME, () => void runner.close());
    },
  };
}
