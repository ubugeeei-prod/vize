import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";

import { collectVueInputPaths } from "./tool-matrix-inputs.mjs";

const fields = new Set([
  "property",
  "category",
  "project",
  "path",
  "rule",
  "reason",
  "trackingIssue",
  "expiryCondition",
]);
const properties = new Set(["idempotence", "parse-preservation", "lint-agreement"]);
const categories = new Set(["semantic-diff", "baseline-unusable", "oracle-unavailable"]);

function identity(entry) {
  return [entry.property, entry.project, entry.path, entry.rule ?? ""].join("\0");
}

export function validateKnownViolationEntries(entries, projects) {
  if (!Array.isArray(entries)) {
    throw new Error("glyph-corpus-known-violations.json must be an array");
  }
  const projectIds = new Set(projects.map((project) => project.id));
  const identities = new Set();
  for (const entry of entries) {
    if (entry == null || typeof entry !== "object" || Array.isArray(entry)) {
      throw new Error("known violation entries must be objects");
    }
    for (const field of Object.keys(entry)) {
      if (!fields.has(field)) throw new Error(`known violation has unknown field ${field}`);
    }
    if (!properties.has(entry.property)) {
      throw new Error(`known violation property is invalid: ${String(entry.property)}`);
    }
    if (!categories.has(entry.category)) {
      throw new Error(`known violation category is invalid: ${String(entry.category)}`);
    }
    if (typeof entry.project !== "string" || !projectIds.has(entry.project)) {
      throw new Error(`known violation project is unknown: ${String(entry.project)}`);
    }
    if (
      typeof entry.path !== "string" ||
      entry.path.length === 0 ||
      entry.path.includes("*") ||
      entry.path.includes("\\") ||
      entry.path.startsWith("/") ||
      !entry.path.endsWith(".vue") ||
      entry.path.split("/").some((segment) => segment === "" || segment === "." || segment === "..")
    ) {
      throw new Error(`known violation path must be an exact relative path: ${String(entry.path)}`);
    }
    for (const field of ["reason", "expiryCondition"]) {
      if (typeof entry[field] !== "string" || entry[field].trim().length === 0) {
        throw new Error(`known violation ${field} must be a non-empty string`);
      }
    }
    if (!Number.isSafeInteger(entry.trackingIssue) || entry.trackingIssue < 1) {
      throw new Error("known violation trackingIssue must be a positive integer");
    }
    if (entry.property === "lint-agreement") {
      if (typeof entry.rule !== "string" || entry.rule.length === 0) {
        throw new Error("known violation lint-agreement entries require a non-empty rule");
      }
    } else if (entry.rule != null) {
      throw new Error("known violation rule is only valid for lint-agreement");
    }
    const key = identity(entry);
    if (identities.has(key)) {
      throw new Error(`duplicate known violation identity: ${key.replaceAll("\0", ":")}`);
    }
    identities.add(key);
  }
  return entries;
}

export function assertHydratedKnownViolationPaths(entries, projects) {
  validateKnownViolationEntries(entries, projects);
  const projectsById = new Map(projects.map((project) => [project.id, project]));
  const registeredFiles = new Map();
  for (const entry of entries) {
    const project = projectsById.get(entry.project);
    if (!project?.hydrated) continue;
    const absolutePath = resolve(project.fixtureDir, entry.path);
    if (!existsSync(absolutePath)) {
      throw new Error(`missing waived fixture path: ${entry.project}:${entry.path}`);
    }
    let files = registeredFiles.get(entry.project);
    if (files == null) {
      files = new Set(collectVueInputPaths(project.fixtureDir, project.vueGlobs));
      registeredFiles.set(entry.project, files);
    }
    if (!files.has(entry.path)) {
      throw new Error(
        `waived fixture path is outside registered Vue inputs: ${entry.project}:${entry.path}`,
      );
    }
  }
}

export function readKnownViolationLedger(path, projects) {
  if (!existsSync(path)) return [];
  const entries = validateKnownViolationEntries(JSON.parse(readFileSync(path, "utf8")), projects);
  assertHydratedKnownViolationPaths(entries, projects);
  return entries;
}

export function createKnownViolationConsumption(entries) {
  const consumed = new Map();
  return {
    consume(project, path, rule, category) {
      const matches = entries.filter(
        (entry) =>
          entry.project === project && entry.path === path && (entry.rule ?? null) === rule,
      );
      if (matches.length === 0) return null;
      if (matches.length > 1) throw new Error(`multiple waivers match ${project}:${path}`);
      const [entry] = matches;
      if (entry.category !== category) {
        throw new Error(
          `known violation category mismatch for ${project}:${path}: ` +
            `${entry.category} != ${category}`,
        );
      }
      const key = identity(entry);
      const count = (consumed.get(key) ?? 0) + 1;
      consumed.set(key, count);
      if (count > 1) {
        throw new Error(`known violation consumed more than once: ${project}:${path}`);
      }
      return entry;
    },
    assertAllConsumed(hydratedProjectIds) {
      for (const entry of entries) {
        if (!hydratedProjectIds.has(entry.project)) continue;
        if ((consumed.get(identity(entry)) ?? 0) !== 1) {
          throw new Error(
            `stale unconsumed waiver for hydrated fixture: ${entry.project}:${entry.path}`,
          );
        }
      }
    },
  };
}

export async function auditKnownViolationIssues(entries, resolveIssue) {
  const issueNumbers = [...new Set(entries.map((entry) => entry.trackingIssue))].sort(
    (left, right) => left - right,
  );
  const evidence = [];
  for (const number of issueNumbers) {
    const issue = await resolveIssue(number);
    if (issue?.number !== number || typeof issue.state !== "string") {
      throw new Error(`tracking Issue #${String(number)} returned invalid evidence`);
    }
    if (issue.state.toUpperCase() !== "OPEN") {
      throw new Error(`tracking Issue #${String(number)} is ${issue.state.toUpperCase()}`);
    }
    evidence.push(issue);
  }
  return evidence;
}

export function writeGlyphCorpusPropertyEvidence(
  property,
  {
    projectIds,
    counters,
    violations,
    baselineUnusable = [],
    waivedViolations = [],
    waiverValidationError,
  },
  reportDir = process.env.FIXTURE_REPORT_DIR,
) {
  if (reportDir == null || reportDir === "") return null;
  if (!properties.has(property)) {
    throw new Error(`cannot write evidence for unknown glyph property: ${property}`);
  }
  const output = resolve(reportDir, `glyph-${property}.json`);
  const artifact = {
    schema: "vize.glyphCorpusPropertyEvidence",
    version: 1,
    property,
    sourceCommit: process.env.GITHUB_SHA ?? null,
    projectIds: [...projectIds].sort((left, right) => left.localeCompare(right)),
    summary: {
      cleanFileCount: counters.files,
      waivedDifferenceCount: waivedViolations.length,
      baselineUnusableCount: baselineUnusable.length,
      violationCount: violations.length,
      waiverValidationError,
    },
    waivedDifferences: waivedViolations,
    baselineUnusable,
    violations,
  };
  mkdirSync(dirname(output), { recursive: true });
  writeFileSync(output, `${JSON.stringify(artifact, null, 2)}\n`);
  return output;
}
