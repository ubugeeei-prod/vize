import fs from "node:fs";
import path from "node:path";

function normalizePortablePath(value) {
  return path.posix
    .normalize(value.replaceAll("\\", "/"))
    .replace(/^([A-Za-z]):/u, (_, drive) => `${drive.toUpperCase()}:`);
}

function stripLeadingCurrentDirectory(value) {
  return value.startsWith("./") ? value.slice(2) : value;
}

function existingRealpathCandidates(filename) {
  if (!fs.existsSync(filename)) return [];
  const candidates = [];
  for (const realpath of [fs.realpathSync.native, fs.realpathSync]) {
    try {
      candidates.push(realpath(filename));
    } catch {
      // Keep the raw path candidate. This helper is only a cross-runner
      // comparison aid, not a filesystem assertion.
    }
  }
  return candidates;
}

function projectRootCandidates(projectRoot) {
  return [...new Set([projectRoot, ...existingRealpathCandidates(projectRoot)])].map((candidate) =>
    normalizePortablePath(candidate),
  );
}

function relativeInsideRoot(root, target) {
  const relative = path.posix.relative(root, target);
  if (relative === "" || (!relative.startsWith("../") && relative !== "..")) {
    return stripLeadingCurrentDirectory(relative);
  }
  return null;
}

function relativeInsideAliasedReleaseSmokeRoot(root, target) {
  const rootParts = root.split("/");
  const targetParts = target.split("/");
  const rootPartsLower = rootParts.map((part) => part.toLowerCase());
  const targetPartsLower = targetParts.map((part) => part.toLowerCase());
  const markerIndex = rootPartsLower.findIndex((part) => part.startsWith("vize-release-smoke-"));
  if (markerIndex === -1 || rootPartsLower[markerIndex + 1] !== "fresh") return null;

  const suffix = rootPartsLower.slice(markerIndex);
  const lastStart = targetParts.length - suffix.length;
  for (let index = 0; index <= lastStart; index += 1) {
    if (suffix.every((part, offset) => targetPartsLower[index + offset] === part)) {
      return stripLeadingCurrentDirectory(targetParts.slice(index + suffix.length).join("/"));
    }
  }
  return null;
}

export function normalizeReportedFile(file, projectRoot) {
  const normalized = normalizePortablePath(file);
  const roots = projectRootCandidates(projectRoot);
  const targets = [];

  if (path.posix.isAbsolute(normalized) || /^[A-Za-z]:\//u.test(normalized)) {
    targets.push(normalized);
    targets.push(
      ...existingRealpathCandidates(file).map((candidate) => normalizePortablePath(candidate)),
    );
  } else if (normalized === "." || normalized === ".." || /^[.]{1,2}\//u.test(normalized)) {
    for (const root of roots) {
      targets.push(normalizePortablePath(path.posix.join(root, normalized)));
    }
  } else {
    return stripLeadingCurrentDirectory(normalized);
  }

  for (const root of roots) {
    for (const target of targets) {
      const relative = relativeInsideRoot(root, target);
      if (relative !== null) return relative;
    }
  }

  for (const root of roots) {
    for (const target of targets) {
      const relative = relativeInsideAliasedReleaseSmokeRoot(root, target);
      if (relative !== null) return relative;
    }
  }

  return stripLeadingCurrentDirectory(normalized);
}
