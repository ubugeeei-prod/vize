import assert from "node:assert/strict";
import { test } from "node:test";
import TOML from "@iarna/toml";

import {
  corsaPackages,
  qualifyCorsaGraph,
} from "../performance/support/warm-type-backed-corsa-graph.ts";

function pair() {
  const manifest = {
    workspace: {
      members: ["crates/vize"],
      package: { version: "0.457.0", edition: "2024" },
      dependencies: {
        corsa: { package: "corsa", version: "=1.14.0" },
        corsa_lsp: { package: "corsa_lsp", version: "=1.14.0" },
        corsa_runtime: { package: "corsa_runtime", version: "=1.14.0" },
        serde: "1",
      },
    },
  };
  const lock = {
    version: 4,
    package: [
      ...corsaPackages.map((name) => ({
        name,
        version: "1.14.0",
        source: "registry+https://github.com/rust-lang/crates.io-index",
        checksum: "a".repeat(64),
        dependencies: ["corsa_runtime", "serde"],
      })),
      ...["vize", "vize_canon", "vize_patina"].map((name) => ({
        name,
        version: "0.457.0",
        dependencies: ["corsa", "corsa_lsp", "serde"],
      })),
      {
        name: "corsa_runtime",
        version: "1.14.2",
        source: "registry+https://github.com/rust-lang/crates.io-index",
        checksum: "b".repeat(64),
        dependencies: ["smallvec"],
      },
      {
        name: "serde",
        version: "1.0.0",
        source: "registry+https://github.com/rust-lang/crates.io-index",
        checksum: "c".repeat(64),
      },
    ],
  };
  const afterManifest = structuredClone(manifest);
  afterManifest.workspace.members.push(...corsaPackages.map((name) => `vendor/${name}`));
  Object.assign(
    afterManifest.workspace.dependencies,
    Object.fromEntries(
      corsaPackages.map((name) => [
        name,
        { package: `vize_${name}`, version: "=0.457.0", path: `vendor/${name}` },
      ]),
    ),
  );
  afterManifest.workspace.dependencies.corsa_runtime.version = "=1.14.2";
  const afterLock = structuredClone(lock);
  for (const record of afterLock.package) {
    if (corsaPackages.includes(record.name as (typeof corsaPackages)[number])) {
      if (record.name === "corsa") record.dependencies!.push("tempfile");
      if (record.name === "corsa_orchestrator") record.dependencies!.push("lsp-types 0.97.0");
      record.name = `vize_${record.name}`;
      record.version = "0.457.0";
      delete record.source;
      delete record.checksum;
    }
    if (record.dependencies)
      record.dependencies = record.dependencies
        .map((name) =>
          corsaPackages.includes(name as (typeof corsaPackages)[number]) ? `vize_${name}` : name,
        )
        .sort();
  }
  return { manifest, lock, afterManifest, afterLock };
}
const qualify = (value: ReturnType<typeof pair>) =>
  qualifyCorsaGraph(
    ...([value.manifest, value.afterManifest, value.lock, value.afterLock].map((value) =>
      TOML.stringify(value),
    ) as [string, string, string, string]),
  );

test("six owned identities retain complete external records and only three consumer edge migrations", () => {
  const receipt = qualify(pair());
  assert.deepEqual(receipt.providers, [...corsaPackages]);
  assert.deepEqual(receipt.consumers, ["vize", "vize_canon", "vize_patina"]);
  assert.equal(receipt.unchangedRecords, 2);
  assert.equal(receipt.runtime.version, "1.14.2");
});

test("source qualification refuses every unrelated graph, provider edge and manifest change", () => {
  const changes: Array<(value: ReturnType<typeof pair>) => void> = [
    (value) => {
      value.afterManifest.workspace.dependencies.serde = "2";
    },
    (value) => {
      value.afterManifest.workspace.members.push("vendor/unrelated");
    },
    (value) => {
      value.afterManifest.workspace.package.edition = "2027";
    },
    (value) => {
      value.afterLock.package.at(-1)!.checksum = "d".repeat(64);
    },
    (value) => {
      value.afterLock.package.find((entry) => entry.name === "corsa_runtime")!.version = "1.15.0";
    },
    (value) => {
      value.afterLock.package.find((entry) => entry.name === "vize_canon")!.version = "0.458.0";
    },
    (value) => {
      value.afterLock.package[1].dependencies!.push("unrelated");
    },
    (value) => {
      value.afterLock.package[0].source = "registry+impostor";
    },
    (value) => {
      value.afterLock.package[0].dependencies = [];
    },
    (value) => {
      value.afterLock.package.push(structuredClone(value.afterLock.package[0]));
    },
    (value) => {
      value.afterLock.package.pop();
    },
  ];
  for (const change of changes) {
    const value = pair();
    change(value);
    assert.throws(() => qualify(value));
  }
});
