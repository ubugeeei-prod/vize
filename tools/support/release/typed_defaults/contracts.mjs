import assert from "node:assert/strict";
import { sha256 } from "./custody.mjs";
import { scenarios } from "./scenarios.mjs";

// Frozen from the original observers, before this GNU consumer was introduced.
// Each vector includes the complete source, configuration, JSON and exit status.
export const originalObservers = {
  typed: "e88410bc963607719124f66e8edf27fa6516b45a69b18e069563938fe95c7619",
  compatibility: "ad832d72b78214d5c3556650af9b93302330744d9342084a82c755391af2eb41",
};
export const originalContracts = {
  typed: {
    processes: 56,
    sha256: "c7d3daacc838fca5ab300653d52ccf391edae96a0f8a3dad4d48547c83f43753",
  },
  first: {
    processes: 80,
    sha256: "35bdb3e2173e31cc146ea4c0c97a6dbac07ad74857503df6a3a0ca6b63d4917a",
  },
  second: {
    processes: 144,
    sha256: "6c8a5bb6beb5a78ae0d186bba7a8a5959cb69e9275e641009ee37f943bed8e3e",
  },
};

export function originalScenarios(fixtures) {
  const vectors = scenarios(fixtures);
  for (const [suite, contract] of Object.entries(originalContracts)) {
    const normalized = vectors
      .filter((entry) => entry.suite === suite)
      .map((entry) => ({
        suite,
        id: entry.entry.id,
        filename: entry.entry.filename,
        source: entry.entry.source,
        preset: entry.preset,
        config: entry.config,
        expected: fixtures.wholeJson(entry.entry, entry.findings),
        expectedStatus: entry.findings.some((finding) => finding.severity === "error") ? 1 : 0,
      }));
    assert.equal(normalized.length, contract.processes);
    assert.equal(
      sha256(JSON.stringify(normalized)),
      contract.sha256,
      `original ${suite} whole behavior vectors changed`,
    );
  }
  return vectors;
}
