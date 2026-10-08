import path from "node:path";
import { createRequire } from "node:module";
import type { ResolvedVizeConfig, VizeOptions } from "../types.ts";
import { mergeCompilerOptions } from "./compiler-config.ts";

const picomatch = createRequire(import.meta.url)("picomatch") as (
  pattern: string,
  options: { dot: boolean; noext: boolean; nonegate: boolean; strictBrackets: boolean },
) => (file: string) => boolean;

type Compiler = NonNullable<ResolvedVizeConfig["compiler"]>;
type Scope = { base: string; compiler: Compiler; matches: (relative: string) => boolean };

function normalizePattern(source: string): string {
  if (!source.includes("/")) return source.replaceAll("\\", "/");
  return source.replace(/\\(.)/g, (_, char: string) =>
    "\\[]*?{}".includes(char) ? (char === "]" ? "[]]" : `[${char}]`) : `/${char}`,
  );
}

function compileSequence(patterns: string[], files: boolean): (file: string) => boolean {
  const steps = patterns.flatMap((source) => {
    const negated = source.startsWith("!");
    const pattern = normalizePattern(negated ? source.slice(1) : source).replace(/^(\.\/)+/, "");
    if (!pattern) return [];
    try {
      return [
        {
          negated,
          matches: picomatch(pattern, {
            dot: true,
            noext: true,
            nonegate: true,
            strictBrackets: true,
          }),
        },
      ];
    } catch (error) {
      console.warn(`[vize] Ignoring invalid entry glob '${source}':`, error);
      return [];
    }
  });
  return (file) => {
    let matched = files && steps.length > 0 && steps.every((step) => step.negated);
    for (const step of steps) {
      let candidate = file;
      while (candidate) {
        if (step.matches(candidate)) {
          matched = !step.negated;
          break;
        }
        const slash = candidate.lastIndexOf("/");
        candidate = slash < 0 ? "" : candidate.slice(0, slash);
      }
    }
    return matched;
  };
}

function compileScopes(config: ResolvedVizeConfig, root: string): Scope[] {
  return config.entries.flatMap((entry) => {
    if (!entry.compiler) return [];
    const files = entry.files === undefined ? () => true : compileSequence(entry.files, true);
    const ignores = compileSequence(entry.ignores ?? [], false);
    return [
      {
        base: path.resolve(root, (entry.basePath ?? "").replaceAll("\\", "/")),
        compiler: entry.compiler,
        matches: (relative: string) => files(relative) && !ignores(relative),
      },
    ];
  });
}

function resolveScopes(config: ResolvedVizeConfig, scopes: Scope[], file: string): Compiler {
  let compiler = config.compiler ?? {};
  for (const scope of scopes) {
    const relative = path.relative(scope.base, file).replaceAll("\\", "/");
    if (relative === ".." || relative.startsWith("../") || path.isAbsolute(relative)) continue;
    if (!scope.matches(relative)) continue;
    compiler = {
      ...compiler,
      ...scope.compiler,
      compatibility: { ...compiler.compatibility, ...scope.compiler.compatibility },
    };
  }
  return compiler;
}

export function compilerConfigForFile(
  config: ResolvedVizeConfig,
  root: string,
  file: string,
): Compiler {
  return resolveScopes(config, compileScopes(config, root), file);
}

export function createFileCompilerOptions(
  config: ResolvedVizeConfig | null,
  root: string,
  explicit: VizeOptions,
  defaults: Pick<VizeOptions, "whitespace"> = {},
): ((file: string) => VizeOptions) | undefined {
  if (!config) return undefined;
  const scopes = compileScopes(config, root);
  if (scopes.length === 0) return undefined;
  return (file) =>
    mergeCompilerOptions(
      explicit,
      { ...config, compiler: resolveScopes(config, scopes, file) },
      defaults,
    );
}
