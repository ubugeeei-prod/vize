import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";

import { stripJsonc } from "./typecheck-baseline-jsonc.ts";

/** A parsed tsconfig. Fields come from fixture files, so each is checked before use. */
export interface TsconfigJson {
  readonly [key: string]: unknown;
  readonly compilerOptions?: Readonly<Record<string, unknown>>;
  readonly vueCompilerOptions?: unknown;
}

/** One config of an `extends` chain and the directory its relative paths resolve from. */
export interface TsconfigChainEntry {
  readonly config: TsconfigJson;
  readonly dir: string;
}

/** Resolves an `extends` specifier to a config path, or null when it is not followed. */
export type ExtendsResolver = (fromConfig: string, specifier: string) => string | null;

export function loadTsconfigExtendsChain(
  sourceConfigPath: string,
  resolveExtends: ExtendsResolver,
): TsconfigChainEntry[] {
  const chain: TsconfigChainEntry[] = [];
  appendExtendsChain(chain, new Set(), resolve(sourceConfigPath), resolveExtends);
  return chain;
}

function appendExtendsChain(
  chain: TsconfigChainEntry[],
  seen: Set<string>,
  current: string,
  resolveExtends: ExtendsResolver,
): void {
  if (seen.has(current)) return;
  seen.add(current);
  const config = parseTsconfig(current);
  if (config == null) return;
  chain.push({ config, dir: dirname(current) });
  const resolved = extendsSpecifiers(config.extends)
    .map((specifier) => resolveExtends(current, specifier))
    .filter((entry) => entry != null);
  for (const next of resolved.reverse()) {
    appendExtendsChain(chain, seen, next, resolveExtends);
  }
}

function parseTsconfig(configPath: string): TsconfigJson | null {
  try {
    return JSON.parse(stripJsonc(readFileSync(configPath, "utf8"))) as TsconfigJson | null;
  } catch {
    return null;
  }
}

function extendsSpecifiers(value: unknown): string[] {
  if (typeof value === "string") return [value];
  return Array.isArray(value) ? value.filter((entry) => typeof entry === "string") : [];
}
