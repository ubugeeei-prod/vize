#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const rootExport = "module.exports = nativeBinding;";
const namedExportPattern =
  /module\.exports\.([$A-Z_a-z][$\w]*)\s*=\s*nativeBinding\.([$A-Z_a-z][$\w]*)\s*;?(?=\s|$)/g;
const scriptPath = fileURLToPath(import.meta.url);
const packageDir = path.resolve(path.dirname(scriptPath), "..");
const defaultEntrypointPath = path.join(packageDir, "index.js");

function collectNamedExports(source) {
  const rootExportMatches = [
    ...source.matchAll(/module\.exports\s*=\s*nativeBinding\s*;?(?=\s|$)/g),
  ];
  if (rootExportMatches.length !== 1) {
    throw new Error(
      `Expected exactly one \`${rootExport}\` assignment, found ${rootExportMatches.length}.`,
    );
  }

  const rootExportMatch = rootExportMatches[0];
  const footer = source.slice((rootExportMatch.index ?? 0) + rootExportMatch[0].length);
  const names = [];
  const seen = new Set();
  let cursor = 0;

  for (const match of footer.matchAll(namedExportPattern)) {
    const unsupported = footer.slice(cursor, match.index).trim();
    if (unsupported !== "") {
      throw new Error(`Unsupported generated entrypoint footer before ${match[0]}: ${unsupported}`);
    }

    const [, publicName, bindingName] = match;
    if (publicName !== bindingName) {
      throw new Error(
        `Refusing to rewrite remapped native export ${publicName} -> ${bindingName}; preserve it explicitly.`,
      );
    }
    if (seen.has(publicName)) {
      throw new Error(`Duplicate generated native export: ${publicName}`);
    }

    seen.add(publicName);
    names.push(publicName);
    cursor = (match.index ?? 0) + match[0].length;
  }

  const trailing = footer.slice(cursor).trim();
  if (trailing !== "") {
    throw new Error(`Unsupported generated entrypoint footer: ${trailing}`);
  }
  if (names.length === 0) {
    throw new Error("The generated native entrypoint did not expose any named exports.");
  }

  return names;
}

function renderNamedExport(name) {
  const assignment = `module.exports.${name} = nativeBinding.${name};`;
  if (assignment.length <= 100) {
    return assignment;
  }
  return `module.exports.${name} =\n  nativeBinding.${name};`;
}

export function synchronizeNativeEntrypoint(source) {
  const namedExports = collectNamedExports(source);
  return `${[
    "/* eslint-disable */",
    "// @ts-nocheck",
    "// Generated from the NAPI-RS export list by scripts/sync-entrypoint.mjs.",
    "// Native target selection and package version checks live in native-binding.js.",
    'const nativeBinding = require("./native-binding");',
    "",
    rootExport,
    ...namedExports.map(renderNamedExport),
  ].join("\n")}\n`;
}

export function synchronizeNativeEntrypointFile(entrypointPath, { check = false } = {}) {
  const source = fs.readFileSync(entrypointPath, "utf8");
  const synchronized = synchronizeNativeEntrypoint(source);

  if (source === synchronized) {
    return false;
  }
  if (check) {
    throw new Error(
      `${entrypointPath} is not synchronized. Run node npm/native/scripts/sync-entrypoint.mjs.`,
    );
  }

  fs.writeFileSync(entrypointPath, synchronized);
  return true;
}

function runCli(args) {
  let check = false;
  let entrypointArgument = null;

  for (const arg of args) {
    if (arg === "--check") {
      check = true;
      continue;
    }
    if (entrypointArgument != null) {
      throw new Error(`Unexpected argument: ${arg}`);
    }
    entrypointArgument = arg;
  }

  const entrypointPath =
    entrypointArgument == null ? defaultEntrypointPath : path.resolve(entrypointArgument);
  synchronizeNativeEntrypointFile(entrypointPath, { check });
}

if (process.argv[1] != null && path.resolve(process.argv[1]) === scriptPath) {
  try {
    runCli(process.argv.slice(2));
  } catch (error) {
    console.error(error instanceof Error ? error.message : error);
    process.exitCode = 1;
  }
}
