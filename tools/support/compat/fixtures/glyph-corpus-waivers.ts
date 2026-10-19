import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";

import { collectVueInputPaths } from "./tool-matrix-inputs.ts";

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
export type GlyphProperty = "idempotence" | "parse-preservation" | "lint-agreement";
export type ViolationCategory = "semantic-diff" | "baseline-unusable" | "oracle-unavailable";

export interface KnownViolation {
  readonly property: GlyphProperty;
  readonly category: ViolationCategory;
  readonly project: string;
  readonly path: string;
  readonly rule?: string;
  readonly reason: string;
  readonly trackingIssue: number;
  readonly expiryCondition: string;
}

export interface WaiverProject {
  readonly id: string;
  readonly hydrated?: boolean;
  readonly fixtureDir: string;
  readonly vueGlobs: readonly string[];
}

export interface TrackingIssue {
  readonly number: number;
  readonly state: string;
}

const properties: ReadonlySet<unknown> = new Set<GlyphProperty>([
  "idempotence",
  "parse-preservation",
  "lint-agreement",
]);
const categories: ReadonlySet<unknown> = new Set<ViolationCategory>([
  "semantic-diff",
  "baseline-unusable",
  "oracle-unavailable",
]);

function identity(entry: KnownViolation): string {
  return [entry.property, entry.project, entry.path, entry.rule ?? ""].join("\0");
}

export function validateKnownViolationEntries(
  entries: unknown,
  projects: readonly { readonly id: string }[],
): KnownViolation[] {
  if (!Array.isArray(entries)) {
    throw new Error("glyph-corpus-known-violations.json must be an array");
  }
  const projectIds = new Set(projects.map((project) => project.id));
  const identities = new Set<string>();
  for (const value of entries as readonly unknown[]) {
    if (value == null || typeof value !== "object" || Array.isArray(value)) {
      throw new Error("known violation entries must be objects");
    }
    const entry = value as Readonly<Record<string, unknown>>;
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
      const text = entry[field];
      if (typeof text !== "string" || text.trim().length === 0) {
        throw new Error(`known violation ${field} must be a non-empty string`);
      }
    }
    if (
      typeof entry.trackingIssue !== "number" ||
      !Number.isSafeInteger(entry.trackingIssue) ||
      entry.trackingIssue < 1
    ) {
      throw new Error("known violation trackingIssue must be a positive integer");
    }
    if (entry.property === "lint-agreement") {
      if (typeof entry.rule !== "string" || entry.rule.length === 0) {
        throw new Error("known violation lint-agreement entries require a non-empty rule");
      }
    } else if (entry.rule != null) {
      throw new Error("known violation rule is only valid for lint-agreement");
    }
    const key = identity(entry as unknown as KnownViolation);
    if (identities.has(key)) {
      throw new Error(`duplicate known violation identity: ${key.replaceAll("\0", ":")}`);
    }
    identities.add(key);
  }
  return entries as KnownViolation[];
}

export function assertHydratedKnownViolationPaths(
  entries: readonly KnownViolation[],
  projects: readonly WaiverProject[],
): void {
  validateKnownViolationEntries(entries, projects);
  const projectsById = new Map(projects.map((project) => [project.id, project]));
  const registeredFiles = new Map<string, Set<string>>();
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

export function readKnownViolationLedger(
  path: string,
  projects: readonly WaiverProject[],
): KnownViolation[] {
  if (!existsSync(path)) return [];
  const parsed: unknown = JSON.parse(readFileSync(path, "utf8"));
  const entries = validateKnownViolationEntries(parsed, projects);
  assertHydratedKnownViolationPaths(entries, projects);
  return entries;
}

export interface KnownViolationConsumption {
  consume(
    project: string,
    path: string,
    rule: string | null,
    category: ViolationCategory,
  ): KnownViolation | null;
  assertAllConsumed(hydratedProjectIds: ReadonlySet<string>): void;
}

export function createKnownViolationConsumption(
  entries: readonly KnownViolation[],
): KnownViolationConsumption {
  const consumed = new Map<string, number>();
  return {
    consume(project, path, rule, category) {
      const matches = entries.filter(
        (entry) =>
          entry.project === project && entry.path === path && (entry.rule ?? null) === rule,
      );
      if (matches.length === 0) return null;
      if (matches.length > 1) throw new Error(`multiple waivers match ${project}:${path}`);
      const entry = matches[0];
      if (entry === undefined) return null;
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

export async function auditKnownViolationIssues(
  entries: readonly KnownViolation[],
  resolveIssue: (number: number) => unknown,
): Promise<TrackingIssue[]> {
  const issueNumbers = [...new Set(entries.map((entry) => entry.trackingIssue))].sort(
    (left, right) => left - right,
  );
  const evidence: TrackingIssue[] = [];
  for (const number of issueNumbers) {
    const resolved: unknown = await resolveIssue(number);
    const issue = resolved as Partial<Record<string, unknown>> | null | undefined;
    if (issue?.number !== number || typeof issue.state !== "string") {
      throw new Error(`tracking Issue #${String(number)} returned invalid evidence`);
    }
    if (issue.state.toUpperCase() !== "OPEN") {
      throw new Error(`tracking Issue #${String(number)} is ${issue.state.toUpperCase()}`);
    }
    evidence.push({ ...issue, number, state: issue.state });
  }
  return evidence;
}

export interface GlyphPropertyEvidence {
  readonly projectIds: Iterable<string>;
  readonly counters: { readonly files: number; readonly [counter: string]: unknown };
  readonly violations: readonly unknown[];
  readonly baselineUnusable?: readonly unknown[];
  readonly waivedViolations?: readonly unknown[];
  readonly waiverValidationError?: unknown;
}

export function writeGlyphCorpusPropertyEvidence(
  property: string,
  {
    projectIds,
    counters,
    violations,
    baselineUnusable = [],
    waivedViolations = [],
    waiverValidationError,
  }: GlyphPropertyEvidence,
  reportDir = process.env.FIXTURE_REPORT_DIR,
): string | null {
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
