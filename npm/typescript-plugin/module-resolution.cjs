"use strict";

const path = require("node:path");

function resolveExistingModuleSpecifier(containingFile, specifier, fileExists, readFile) {
  if (typeof containingFile !== "string" || typeof specifier !== "string") {
    return undefined;
  }
  if (isRelativeSpecifier(specifier)) {
    return resolveSourceCandidate(
      path.resolve(path.dirname(containingFile), specifier),
      fileExists,
    );
  }
  if (path.isAbsolute(specifier)) {
    return undefined;
  }
  return resolvePackageSpecifier(containingFile, specifier, fileExists, readFile);
}

function resolvePackageSpecifier(containingFile, specifier, fileExists, readFile) {
  const parsed = parsePackageSpecifier(specifier);
  if (!parsed) {
    return undefined;
  }
  for (const modulesDir of nodeModulesAncestors(path.dirname(containingFile))) {
    const packageRoot = path.join(modulesDir, parsed.packageName);
    const packageJson = path.join(packageRoot, "package.json");
    if (!fileExists(packageJson)) {
      continue;
    }
    if (parsed.subpath) {
      return resolveSourceCandidate(path.join(packageRoot, parsed.subpath), fileExists);
    }
    const manifest = readPackageManifest(packageJson, readFile);
    for (const field of ["types", "typings", "module", "main"]) {
      const target = typeof manifest?.[field] === "string" ? manifest[field] : undefined;
      const resolved = target
        ? resolveSourceCandidate(path.join(packageRoot, target), fileExists)
        : undefined;
      if (resolved) {
        return resolved;
      }
    }
    const resolved = resolveSourceCandidate(path.join(packageRoot, "index"), fileExists);
    if (resolved) {
      return resolved;
    }
  }
  return undefined;
}

function parsePackageSpecifier(specifier) {
  const parts = specifier.split("/");
  if (parts.length === 0 || parts[0] === "") {
    return undefined;
  }
  if (parts[0].startsWith("@")) {
    if (parts.length < 2 || parts[1] === "") {
      return undefined;
    }
    return {
      packageName: `${parts[0]}/${parts[1]}`,
      subpath: parts.slice(2).join("/"),
    };
  }
  return {
    packageName: parts[0],
    subpath: parts.slice(1).join("/"),
  };
}

function* nodeModulesAncestors(startDirectory) {
  let current = path.resolve(startDirectory);
  while (true) {
    yield path.join(current, "node_modules");
    const parent = path.dirname(current);
    if (parent === current) {
      break;
    }
    current = parent;
  }
}

function readPackageManifest(packageJson, readFile) {
  const source = typeof readFile === "function" ? readFile(packageJson) : undefined;
  if (typeof source !== "string") {
    return undefined;
  }
  try {
    return JSON.parse(source);
  } catch {
    return undefined;
  }
}

function resolveSourceCandidate(basePath, fileExists) {
  const candidates = [
    basePath,
    `${basePath}.ts`,
    `${basePath}.tsx`,
    `${basePath}.js`,
    `${basePath}.jsx`,
    path.join(basePath, "index.ts"),
    path.join(basePath, "index.tsx"),
    path.join(basePath, "index.js"),
    path.join(basePath, "index.jsx"),
  ];
  return candidates.find((candidate) => fileExists(candidate));
}

function isRelativeSpecifier(specifier) {
  return (
    typeof specifier === "string" && (specifier.startsWith("./") || specifier.startsWith("../"))
  );
}

module.exports = {
  isRelativeSpecifier,
  resolveExistingModuleSpecifier,
};
