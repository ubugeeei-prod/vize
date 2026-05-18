import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const checklistPath = path.join(root, "docs", "release", "v1-alpha-go-no-go.md");
const productionReadinessPath = path.join(root, "docs", "release", "production-readiness.md");

test("v1 alpha go/no-go checklist covers release gates and rollback", () => {
  const checklist = fs.readFileSync(checklistPath, "utf-8");

  for (const heading of [
    "## Owners",
    "## Pre-Tag Gate",
    "## Tag Gate",
    "## Publish Gate",
    "## Post-Publish Gate",
    "## Rollback Plan",
    "## Communication",
  ]) {
    assert.match(checklist, new RegExp(`^${heading}$`, "m"));
  }

  for (const workflow of [
    "../../.github/workflows/check.yml",
    "../../.github/workflows/benchmark.yml",
    "../../.github/workflows/e2e.yml",
    "../../.github/workflows/deploy-docs.yml",
    "../../.github/workflows/release.yml",
  ]) {
    assert.match(checklist, new RegExp(workflow.replaceAll(".", "\\.")));
  }

  for (const owner of [
    "npm owner",
    "Rust owner",
    "Editor owner",
    "Docs owner",
    "Release captain",
  ]) {
    assert.match(checklist, new RegExp(owner));
  }

  for (const requiredTerm of [
    "dist-tags",
    "crates.io",
    "VS Code marketplace",
    "GitHub release",
    "cargo yank",
    "npm deprecate",
  ]) {
    assert.match(checklist, new RegExp(requiredTerm));
  }
});

test("production-readiness checklist scopes supported and experimental surfaces", () => {
  const readiness = fs.readFileSync(productionReadinessPath, "utf-8");

  for (const heading of [
    "# Production Readiness",
    "## Current Support Scope",
    "## Required Gates",
    "## Exit Criteria For Removing Public Warnings",
    "## How To Answer The Readiness Question",
  ]) {
    assert.match(readiness, new RegExp(`^${heading}$`, "m"));
  }

  for (const requiredTerm of [
    "not yet a stable, production-ready toolchain",
    "Alpha-supported",
    "Experimental",
    "cargo audit --deny warnings",
    "real-world fixture coverage",
    "Official Vue tooling remains the compatibility baseline",
  ]) {
    assert.match(readiness, new RegExp(requiredTerm));
  }
});
