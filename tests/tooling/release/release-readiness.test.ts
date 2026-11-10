import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const checklistPath = path.join(root, "docs", "release", "v1-alpha-go-no-go.md");
const evidencePath = path.join(root, "docs", "release", "davinci-v1-evidence.md");
const productionReadinessPath = path.join(root, "docs", "release", "production-readiness.md");
const supplyChainPath = path.join(root, "docs", "release", "supply-chain.md");
const vueParityMatrixPath = path.join(root, "docs", "release", "vue-parity-matrix.md");

test("v1 alpha go/no-go checklist covers release gates and rollback", () => {
  const checklist = fs.readFileSync(checklistPath, "utf-8");
  const evidence = fs.readFileSync(evidencePath, "utf-8");

  for (const heading of [
    "## Owners",
    "## Pre-Tag Gate",
    "## Release Candidate and Tag Gate",
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
    "../../.github/workflows/fuzz.yml",
    "../../.github/workflows/e2e.yml",
    "../../.github/workflows/build-docs.yml",
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
    "release-candidates/",
    "source commit",
    "prerelease: true",
    "make_latest: false",
    "dist-tags",
    "crates.io",
    "VS Code marketplace",
    "GitHub release",
    "cargo yank",
    "npm deprecate",
  ]) {
    assert.match(checklist, new RegExp(requiredTerm));
  }

  for (const retiredContract of [
    "vp run release alpha -y",
    "Atomic promotion fast-forwards",
    "Open VSX does not run automatically",
    "release-open-vsx.yml",
  ]) {
    assert.doesNotMatch(checklist, new RegExp(retiredContract));
    assert.doesNotMatch(evidence, new RegExp(retiredContract));
  }

  assert.match(evidence, /^### Release candidate and tag gate$/m);
  assert.match(evidence, /refs\/release-candidates\/<source SHA>/);
  assert.match(evidence, /Automatic Open VSX universal and platform channels/);
});

test("production-readiness checklist scopes supported and experimental surfaces", () => {
  const readiness = fs.readFileSync(productionReadinessPath, "utf-8");

  for (const heading of [
    "# Production Readiness",
    "## Current Support Scope",
    "## Required Gates",
    "## Current Audit Snapshot",
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
    "fuzz.yml",
    "seeded corpus health",
    "real-world fixture coverage",
    "line and branch coverage gates",
    "coverage:source",
    "coverage:source:branch",
    "test:check:fixtures",
    "--runtime-checks",
    "fresh-install smoke coverage",
    "linux-arm64-gnu",
    "win32-arm64-msvc",
    "linux-x64-musl",
    "parity",
    "Vue Parity Matrix",
    "Official Vue tooling remains the compatibility baseline",
  ]) {
    assert.match(readiness, new RegExp(requiredTerm));
  }
});

test("Vue parity matrix names release-blocking compiler, typecheck, runtime, and Vite gates", () => {
  const matrix = fs.readFileSync(vueParityMatrixPath, "utf-8");

  for (const heading of [
    "# Vue Parity Matrix",
    "## Baseline Versions",
    "## Compatibility Surfaces",
    "## Required Release Gates",
  ]) {
    assert.match(matrix, new RegExp(`^${heading}$`, "m"));
  }

  for (const requiredTerm of [
    "Official Vue tooling is the baseline",
    "@vue/compiler-sfc",
    "vue-tsc",
    "Alpha-supported",
    "Preview",
    "Incubating",
    "Experimental",
    "compileSfc",
    "vite build",
    "coverage:source:branch",
    "--runtime-checks",
  ]) {
    assert.match(matrix, new RegExp(requiredTerm));
  }
});

test("supply-chain verification examples are release-version neutral", () => {
  const supplyChain = fs.readFileSync(supplyChainPath, "utf-8");

  assert.match(supplyChain, /vize-<tag>-cyclonedx\.sbom\.json/);
  assert.doesNotMatch(supplyChain, /vize-v\d+\.\d+\.\d+-cyclonedx\.sbom\.json/);
});
