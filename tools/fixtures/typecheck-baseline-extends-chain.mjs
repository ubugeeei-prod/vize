import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";

import { stripJsonc } from "./typecheck-baseline-jsonc.mjs";

export function loadTsconfigExtendsChain(sourceConfigPath, resolveExtends) {
  const chain = [];
  appendExtendsChain(
    chain,
    new Set(),
    resolve(sourceConfigPath),
    resolveExtends,
  );
  return chain;
}

function appendExtendsChain(chain, seen, current, resolveExtends) {
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

function parseTsconfig(configPath) {
  try {
    return JSON.parse(stripJsonc(readFileSync(configPath, "utf8")));
  } catch {
    return null;
  }
}

function extendsSpecifiers(value) {
  if (typeof value === "string") return [value];
  return Array.isArray(value)
    ? value.filter((entry) => typeof entry === "string")
    : [];
}
